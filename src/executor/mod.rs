use std::{collections::HashMap, io::{BufReader}, sync::{Arc, Mutex}, time::Instant};
use handlebars::HelperDef;
use llm::{builder::LLMBuilder, chat::{ChatMessage, StructuredOutputFormat}};
use log::debug;
use log::error;
use regex::RegexBuilder;
use serde_json::Value;
use thiserror::Error;
use tokio::runtime::Runtime;
use xxhash_rust::xxh3::xxh3_64;
use crate::{
    config::{
        appconfig::{
            self, GlobalProviderProperties
        },
        resolver::{
            error::ResolveError,
            ResolvedGlobalProperties,
            ResolvedPropertySource,
            Resolver}
    },
    dotprompt::{
        helpers,
        OutputFormat
    },
    executor::{
        partiallog::{ExecutionLogData, PartialLogRecord}, streaming_output::StreamingExecutionOutput, structured_streaming_output::StructuredStreamingExecutionOutput
    }
};
use crate::config::providers;
use crate::config::resolver;
use crate::dotprompt;
use crate::dotprompt::renderers;
use crate::dotprompt::renderers::Render;
use crate::lb;
use crate::stats::store;
use crate::storage;
mod partiallog;
mod streaming_output;
mod structured_streaming_output;
mod streaming_code_extractor;

pub enum ExecutionOutput {
    StreamingOutput(Box<StreamingExecutionOutput>),
    StructuredStreamingOutput(Box<StructuredStreamingExecutionOutput>),
    ImmediateOutput(String),
    DryRun,
    RenderOnly(String),
    Cached(String)
}

#[derive(Debug)]
pub struct PromptInputs {
    pub map: HashMap<String, Value>,
}

impl PromptInputs {
    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }
    pub fn insert(&mut self, k: String, v: Value) {
        self.map.insert(k, v);
    }
}

#[derive(Error, Debug)]
pub enum ExecutorErorr {
    #[error("PromptFilesStorageError: {0}")]
    PromptFilesStorageError(#[from] storage::PromptFilesStorageError),

    #[error("DotPromptParseError: {0}")]
    DotPromptParseError(#[from] dotprompt::ParseError),

    #[error("DotPromptRenderError: {0}")]
    DotPromptRenderError(#[from] renderers::RenderError),

    #[error("ResolverError: {0}")]
    ResolverError(#[from] resolver::error::ResolveError),

    #[error("ToLLMBuilderError: {0}")]
    LMBuilderError(#[from] providers::error::ToLLMBuilderError),

    #[error("LLMError: {0}")]
    LLMError(#[from] llm::error::LLMError),

    #[error("LoadBalancerError: {0}")]
    LoadBalancerError(#[from] lb::LBError),

    #[error("JSON Error: {0}")]
    JSONError(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

pub struct Executor {
    pub loadbalancer: lb::WeightedLoadBalancer,
    pub appconfig: &'static appconfig::AppConfig,
    pub statsstore: &'static dyn store::StatsStore,
    pub prompts_storage: &'static dyn storage::PromptFilesStorage,
}

pub(crate) fn extract_fenced_code(input: &str) -> Vec<String> {
    let re = RegexBuilder::new(r"```(?:\w+)?\n(.*?)```")
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    re.captures_iter(input)
        .map(|cap| cap[1].to_string())
        .collect()
}

impl Executor {

    pub fn load_dotprompt(&self, promptname: &str) -> Result<dotprompt::DotPrompt, ExecutorErorr> {
        debug!("Loading prompt name: {}", promptname);

        let (path, promptfile_content) = self.prompts_storage.load(promptname)?;

        debug!("Promptfile path: {path}");

        let dotprompt = dotprompt::DotPrompt::try_from((promptname, promptfile_content.as_str()))?;

        Ok(dotprompt)
    }
    fn cache_key(
        promptname: &str,
        provider: &str,
        model: &str,
        variant: Option<&str>,
        group: Option<&str>,
        data: &str) -> i64 {
        let full_data = format!(
            "{}|{}|{}|{}|{}|{}",
            promptname,
            provider,
            model,
            variant.unwrap_or("-"),
            group.unwrap_or("-"),
            data.replace(" ", "").replace("\n", "")
        );
        xxh3_64(full_data.as_bytes()) as i64
    }

    pub fn execute_dotprompt(
        self: Arc<Self>,
        dotprompt: &dotprompt::DotPrompt,
        overrides: Option<ResolvedGlobalProperties>,
        requested_model: Option<String>,
        inputs: PromptInputs,
        dry: bool,
        render_only: bool) -> Result<ExecutionOutput, ExecutorErorr>{

        debug!("Executing dotprompt");

        let next_exec = self.clone();
        let prompt_helper: Box<dyn HelperDef + Send + Sync> = Box::new(helpers::PromptHelper {
            executor: next_exec,
            dry, render_only
        });

        let exec_helper: Box<dyn HelperDef + Send + Sync> = Box::new(helpers::ExecHelper);
        let concat_helper: Box<dyn HelperDef + Send + Sync> = Box::new(helpers::ConcatHelper);
        let stdin_helper: Box<dyn HelperDef + Send + Sync> = Box::new(helpers::StdinHelper {
            inp: Mutex::new(BufReader::new(std::io::stdin()))
        });
        let stdin_helper2: Box<dyn HelperDef + Send + Sync> = Box::new(helpers::StdinHelper {
            inp: Mutex::new(BufReader::new(std::io::stdin()))
        });
        let ask_helper: Box<dyn HelperDef + Send + Sync> = Box::new(helpers::AskHelper {
            promptname: dotprompt.name.clone(),
            inp: Mutex::new(BufReader::new(std::io::stdin()))
        });

        let helpers_map: HashMap<&str, Box<dyn HelperDef + Send + Sync>> = HashMap::from([
            ("exec", exec_helper),
            ("prompt", prompt_helper),
            ("concat", concat_helper),
            ("stdin", stdin_helper),
            ("STDIN", stdin_helper2),
            ("ask", ask_helper),
        ]);

        let rendered_dotprompt: String = dotprompt.render(inputs, helpers_map)?;

        debug!("{rendered_dotprompt}");

        if render_only {
            return Ok(ExecutionOutput::RenderOnly(rendered_dotprompt));
        }

        let mut resolver = Resolver {
            overrides,
            fm_properties: Some(ResolvedGlobalProperties::from((
                &GlobalProviderProperties::from(&dotprompt.frontmatter),
                ResolvedPropertySource::Dotprompt(dotprompt.name.clone()),
            ))
        )};

        let resolved_config = resolver.resolve(
            self.appconfig, requested_model).map_err(|err| {
               match err {
                    ResolveError::NoNameToResolve => {
                        ExecutorErorr::Other("No model specified and no default model set in config".to_string())
                    }
                    err => ExecutorErorr::ResolverError(err)
               }
            })?;

        let (globals, group_choice, variant_name, (model_info, mut llmbuilder)) = match &resolved_config {
            resolver::ResolvedConfig::Base(base)  => {
                (&base.globals, None, None, <(providers::ModelInfo, LLMBuilder)>::try_from(base)?)
            },
            resolver::ResolvedConfig::Variant(variant)  => {
                (&variant.globals, None, Some(variant.name.clone()), <(providers::ModelInfo,
                    LLMBuilder)>::try_from(variant)?)
            },
            resolver::ResolvedConfig::Group(group) => {
                let choice = self.loadbalancer.choose(group,
                    lb::BalanceScope::Group , lb::BalanceLevel::Variant)?;
                match choice {
                    lb::Choice::Base(base) => {
                        (&base.globals, Some((group.name.clone(), choice)), None, <(providers::ModelInfo,
                            LLMBuilder)>::try_from(base)?)
                    }
                    lb::Choice::Variant(variant) => {
                        (&variant.globals, Some((group.name.clone(), choice)), Some(variant.name.clone()),
                            <(providers::ModelInfo, LLMBuilder)>::try_from(variant)?)
                    }
                }
            }
        };

        if matches!(dotprompt.frontmatter.output.format, OutputFormat::Json) {
            let output_schema: StructuredOutputFormat = serde_json::from_str(
                dotprompt.output_to_extract_structured_json("").as_str())?;
            llmbuilder = llmbuilder.schema(output_schema);
        }

        if dry {
            println!("Dry run mode");
            println!("=============");

            println!(">>> Resolved Config");
            println!("{}\n", &resolved_config);

            if let Some((_, choice)) = group_choice {
                println!(">>> LB Choice");
                println!("{}\n", choice);
            }

            println!(">>> Rendered Prompt:");
            print!("{}", &rendered_dotprompt);
            println!("<<< End Rendered Prompt");

            return Ok(ExecutionOutput::DryRun)
        }

        let cache_key = Executor::cache_key(
            &dotprompt.template,
            &model_info.provider,
            &model_info.model,
            variant_name.as_deref(),
            group_choice.as_ref().map(|(n, _)| n.as_str()),
            &rendered_dotprompt
        );

        if let Some(cache_ttl) = &globals.cache_ttl && cache_ttl.value > 0 {
            debug!("Cache requested, ttl set to {} seconds via {}", cache_ttl.value, &cache_ttl.source);
            match self.statsstore.cached(cache_key, cache_ttl.value) {
                Ok(Some(record)) => {
                    debug!("Found cached response");

                    if matches!(dotprompt.frontmatter.output.format, OutputFormat::Code) {
                        let fenced_codes = extract_fenced_code(record.result.as_str());
                        if !fenced_codes.is_empty() {
                            return Ok(ExecutionOutput::Cached(fenced_codes.join("\n").trim().to_string()));
                        }
                    }
                    return Ok(ExecutionOutput::Cached(record.result))
                },
                Ok(None) => {
                    debug!("No cache found")
                },
                Err(err) => {
                    error!("Cache error: {}", err)
                }
            }
        }

        let llm = llmbuilder.build()?;

        let messages = vec![
            ChatMessage::user()
                .content(rendered_dotprompt)
                .build(),
        ];
        let rt = Runtime::new().unwrap();

        let start_time = Instant::now();

        let partial_log_record = PartialLogRecord {
            statsstore: self.statsstore,
            promptname: dotprompt.template.clone(),
            provider: model_info.provider.clone(),
            model: model_info.model.clone(),
            variant: variant_name.clone(),
            group: group_choice.map(|(n, _)| n.clone()),
            cache_key: Some(cache_key)
        };

        if let Some(stream)= globals.stream.as_ref() && stream.value {
            debug!("stream mode");

            match model_info.provider.as_str() {
                "openai" | "google" | "openrouter" => {
                    match rt.block_on(llm.chat_stream_struct(&messages)) {
                        Ok(stream) => {
                            Ok(
                                ExecutionOutput::StructuredStreamingOutput(Box::new(StructuredStreamingExecutionOutput::new(
                                    partial_log_record,
                                    rt,
                                    stream,
                                    dotprompt.frontmatter.output.format.clone()
                                )))
                            )
                        }
                        Err(err) => {
                            panic!("{}", err);
                        }
                    }
                }
                _ => {
                    match rt.block_on(llm.chat_stream(&messages)) {
                        Ok(stream) => {
                            Ok(
                                ExecutionOutput::StreamingOutput(Box::new(StreamingExecutionOutput::new(
                                    partial_log_record,
                                    rt,
                                    stream,
                                    dotprompt.frontmatter.output.format.clone()
                                )))
                            )
                        }
                        Err(err) => {
                            panic!("{}", err);
                        }
                    }

                }
            }
        } else {
            let result = rt.block_on(llm.chat(&messages));

            let elapsed = start_time.elapsed().as_secs() as u32;

            // Send chat request and handle the response
            let (success, response_text, prompt_tokens, completion_tokens) = match result  {
                Ok(response) => {

                    let response_text = response.text().unwrap_or_default();
                    let (prompt_tokens, completion_tokens) = response.usage().map_or((0, 0),
                        |usage| (usage.prompt_tokens, usage.completion_tokens));

                    (true, response_text, prompt_tokens, completion_tokens)
                }
                Err(e) => (false, e.to_string(), 0, 0)
            };

            let log_result = partial_log_record.log(
                ExecutionLogData {
                    prompt_tokens,
                    completion_tokens,
                    result: response_text.as_str(),
                    success,
                    time_taken: elapsed,
                }
            );

            if let Err(err) = log_result {
                error!("Logging execution failed: {}", err);
            }

            if matches!(dotprompt.frontmatter.output.format, OutputFormat::Code) {
                let fenced_codes = extract_fenced_code(response_text.as_str());
                if !fenced_codes.is_empty() {
                    return Ok(ExecutionOutput::ImmediateOutput(fenced_codes.join("\n").trim().to_string()));
                }
            }
            Ok(ExecutionOutput::ImmediateOutput(response_text))
        }
    }

    pub fn execute(self: Arc<Self>, promptname: &str, overrides: Option<ResolvedGlobalProperties>,
        requested_model: Option<String>, inputs: PromptInputs, dry: bool, render_only: bool) -> Result<ExecutionOutput,
    ExecutorErorr>{
        debug!("Executing prompt name: {}", promptname);
        let dotprompt = self.load_dotprompt(promptname)?;

        self.execute_dotprompt(&dotprompt, overrides, requested_model,inputs, dry, render_only)
    }
}
