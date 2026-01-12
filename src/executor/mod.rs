use std::{collections::HashMap, sync::{Arc, Mutex}, time::Instant};

use chrono::Utc;
use handlebars::HelperDef;
use llm::{builder::LLMBuilder, chat::{ChatMessage, StructuredOutputFormat}};
use log::debug;
use log::error;
use serde_json::Value;
use thiserror::Error;
use tokio::runtime::Runtime;
use crate::{config::appconfig, dotprompt::{helpers, OutputFormat}};
use crate::config::providers;
use crate::config::resolver;
use crate::dotprompt;
use crate::dotprompt::renderers;
use crate::dotprompt::renderers::Render;
use crate::lb;
use crate::stats::store;
use crate::storage;


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
    pub appconfig: Arc<appconfig::AppConfig>,
    pub statsstore: Arc<Mutex<dyn store::StatsStore + Send>>,
    pub prompts_storage: Arc<Mutex<dyn storage::PromptFilesStorage + Send>>,
}


impl Executor {

    pub fn load_dotprompt(&self, promptname: &str) -> Result<dotprompt::DotPrompt, ExecutorErorr> {
        debug!("Loading prompt name: {}", promptname);

        let (path, promptfile_content) = self.prompts_storage.lock().unwrap().load(promptname)?;

        debug!("Promptfile path: {path}");

        let dotprompt = dotprompt::DotPrompt::try_from((promptname, promptfile_content.as_str()))?;

        Ok(dotprompt)
    }

    pub fn execute_dotprompt(
        self: Arc<Self>,
        dotprompt: &dotprompt::DotPrompt,
        inputs: PromptInputs,
        dry: bool) -> Result<String, ExecutorErorr>{

        debug!("Executing dotprompt");

        let appconfig = self.appconfig.as_ref();

        let prompt_helper: Box<dyn HelperDef + Send + Sync> = Box::new(helpers::PromptHelper {
            executor: self.clone(),
            dry
        });

        let exec_helper: Box<dyn HelperDef + Send + Sync> = Box::new(helpers::ExecHelper);
        let concat_helper: Box<dyn HelperDef + Send + Sync> = Box::new(helpers::ConcatHelper);

        let helpers_map: HashMap<&str, Box<dyn HelperDef + Send + Sync>> = HashMap::from([
            ("exec", exec_helper),
            ("prompt", prompt_helper),
            ("concat", concat_helper)
        ]);


        let rendered_dotprompt: String = dotprompt.render(inputs, helpers_map)?;

         debug!("{rendered_dotprompt}");

        let requested_name = dotprompt.frontmatter.model.clone()
            .or(appconfig.providers.default.clone())
            .or(appconfig.providers.globals.model.clone())
            .ok_or(ExecutorErorr::Other("No model specified and no default model set in config".to_string()))?;

        let resolved_config = resolver::resolve(
            self.appconfig.as_ref(), &requested_name,
            Some(resolver::ResolvedPropertySource::Dotprompt(requested_name.clone())))?;

        let (group_choice, variant_name, (model_info, mut llmbuilder)) = match &resolved_config {
            resolver::ResolvedConfig::Base(base)  => {
                (None, None, <(providers::ModelInfo, LLMBuilder)>::try_from(base)?)
            },
            resolver::ResolvedConfig::Variant(variant)  => {
                (None, Some(variant.name.clone()), <(providers::ModelInfo,
                    LLMBuilder)>::try_from(variant)?)
            },
            resolver::ResolvedConfig::Group(group) => {
                let choice = self.loadbalancer.choose(group,
                    lb::BalanceScope::Group , lb::BalanceLevel::Variant)?;
                match choice {
                    lb::Choice::Base(base) => {
                        (Some((group.name.clone(), choice)), None, <(providers::ModelInfo,
                            LLMBuilder)>::try_from(base)?)
                    }
                    lb::Choice::Variant(variant) => {
                        (Some((group.name.clone(), choice)), Some(variant.name.clone()),
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

            return Ok("[no response due to dry run]".to_string());
        }

        let llm = llmbuilder.build()?;

        let messages = vec![
            ChatMessage::user()
                .content(rendered_dotprompt)
                .build(),
        ];
        let rt = Runtime::new().unwrap();

        let start_time = Instant::now();

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

        let log_result = self.statsstore.lock().unwrap().log(
            store::LogRecord {
                promptname: dotprompt.template.clone(),
                provider: model_info.provider,
                model: model_info.model,
                variant: variant_name,
                group: group_choice.map(|(n, _)| n),
                prompt_tokens,
                completion_tokens,
                result: response_text.clone(),
                success,
                time_taken: elapsed,
                created: Utc::now()
            }
        );

        if let Err(err) = log_result {
            error!("Logging execution failed: {}", err);
        }

        Ok(response_text)
    }

    pub fn execute(self: Arc<Self>, promptname: &str, inputs: PromptInputs, dry: bool) -> Result<String, ExecutorErorr>{
        debug!("Executing prompt name: {}", promptname);
        let dotprompt = self.load_dotprompt(promptname)?;

        self.execute_dotprompt(&dotprompt, inputs, dry)
    }
}
