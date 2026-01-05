use chrono::Utc;
use clap::{Parser};
use log::{error, debug};
use clap::{Arg, ArgMatches, Command};
use std::{collections::HashMap};
use std::time::Instant;
use std::convert::TryFrom;
use anyhow::{Context, Result};
use thiserror::Error;
use llm::{
    builder::{LLMBuilder},
    chat::{ChatMessage},
};
use llm::chat::StructuredOutputFormat;
use tokio::runtime::Runtime;
use crate::config::appconfig::AppConfig;
use crate::lb::{
    BalanceLevel, BalanceScope, WeightedLoadBalancer, Choice
};
use crate::config::resolver::{self, ResolvedConfig, ResolvedPropertySource};
use crate::config::providers::ModelInfo;
use crate::dotprompt::DotPrompt;
use crate::dotprompt::render::Render;
use crate::config::{appconfig_locator,};
use crate::stats::store::{LogRecord, StatsStore};
use crate::storage::PromptFilesStorage;
use std::fs;

#[derive(Parser)]
pub struct RunCmd {
    #[arg()]
    pub promptname: String,

    #[arg(long, short, help="Dry run" )]
    pub dry: bool,

    #[arg(trailing_var_arg = true)]
    pub prompt_args: Vec<String>,
}

pub fn generate_arguments_from_dotprompt(mut command: Command, dotprompt: &DotPrompt) -> Result<Command> {
    let args: Vec<Arg> = Vec::try_from(dotprompt).context(
        "Could not generate arguments"
    )?;

    for arg in args {
       command = command.arg(arg);
    }
    Ok(command)
}

#[derive(Error, Debug)]
pub enum RunCmdError {
    #[error("'{0}' is required but not configured")]
    RequiredConfiguration(&'static str)
}

impl RunCmd {
    pub fn exec_prompt(&self,
        inp: &mut impl std::io::BufRead,
        out: &mut impl std::io::Write,
        store: &impl StatsStore,
        dotprompt: &DotPrompt, appconfig: &AppConfig,
        lb: &WeightedLoadBalancer,
        matches: &ArgMatches,) -> Result<()> {

        let mut extra_args: HashMap<String, String> = HashMap::new();

        if dotprompt.template_needs_stdin() {
            let mut buffer = String::new();
            inp.read_to_string(&mut buffer)
                .context("Failed to read stdin")?;
            extra_args.insert(String::from("STDIN"), buffer);
        }

        let output: String = dotprompt.render(matches, &extra_args)?;

        debug!("{output}");

        let requested_name = dotprompt.frontmatter.model.clone()
            .or(appconfig.providers.globals.model.clone())
            .context("No model specified and no default models set in config")?;

        let resolved_config = resolver::resolve(
            appconfig, &requested_name, Some(ResolvedPropertySource::Dotprompt(requested_name.clone())))?;

        let (group_choice, variant_name, (model_info, mut llmbuilder)) = match &resolved_config {
            ResolvedConfig::Base(base)  => {
                (None, None, <(ModelInfo, LLMBuilder)>::try_from(base)?)
            },
            ResolvedConfig::Variant(variant)  => {
                (None, Some(variant.name.clone()), <(ModelInfo, LLMBuilder)>::try_from(variant)?)
            },
            ResolvedConfig::Group(group) => {
                let choice = lb.choose(group, BalanceScope::Group , BalanceLevel::Variant)?;
                match choice {
                    Choice::Base(base) => {
                        (Some((group.name.clone(), choice)), None, <(ModelInfo, LLMBuilder)>::try_from(base)?)
                    }
                    Choice::Variant(variant) => {
                        (Some((group.name.clone(), choice)), Some(variant.name.clone()), <(ModelInfo, LLMBuilder)>::try_from(variant)?)
                    }
                }
            }
        };

        debug!("Model Provider: {}, Model Name: {}", &model_info.provider, &model_info.model);

        // let mut llmbuilder= LLMBuilder::new()
        //     .model(&model);

        if dotprompt.output_format() == "json" {
            let output_schema: StructuredOutputFormat = serde_json::from_str(
                dotprompt.output_to_extract_structured_json("").as_str())?;
            llmbuilder = llmbuilder.schema(output_schema);
        }

        if self.dry {
            println!("Dry run mode.");
            println!("=============");

            println!(">>> Resolved Config");
            println!("{}\n", &resolved_config);

            if let Some((_, choice)) = group_choice {
                println!(">>> LB Choice");
                println!("{}\n", choice);
            }

            println!(">>> Rendered Prompt");
            println!("{}", &output);

            return Ok(());
        }

        let llm = llmbuilder.build()?;

        let messages = vec![
            ChatMessage::user()
                .content(output)
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

        writeln!(out, "{response_text}")?;

        let log_result = store.log(
            LogRecord {
                promptname: self.promptname.clone(),
                provider: model_info.provider,
                model: model_info.model,
                variant: variant_name,
                group: group_choice.map(|(n, _)| n),
                prompt_tokens,
                completion_tokens,
                result: response_text,
                success,
                time_taken: elapsed,
                created: Utc::now()
            }
        );

        if let Err(err) = log_result {
            error!("Logging execution failed: {}", err);
        }

        Ok(())
    }

    pub fn exec(&self,
        inp: &mut impl std::io::BufRead,
        out: &mut impl std::io::Write,
        store: &impl StatsStore,
        prompt_storage: &impl PromptFilesStorage,
        lb: &WeightedLoadBalancer) -> Result<()> {

        let appconfig = if let Some(appconfig_path) = appconfig_locator::path() {
            debug!("Config Path: {}",appconfig_path.display());
            AppConfig::try_from(
                &fs::read_to_string(&appconfig_path)?
            )?
        } else {
            AppConfig::default()
        };

        debug!("Prompt name: {}", &self.promptname);

        prompt_storage.exists(&self.promptname).context("Could not find promptfile")?;

        let (path, promptfile_content) = prompt_storage.load(&self.promptname)?;

        debug!("Promptfile path: {path}");

        let dotprompt: DotPrompt = DotPrompt::try_from(promptfile_content.as_str())?;

        let mut command: Command = Command::new(self.promptname.to_string());

        command = generate_arguments_from_dotprompt(command, &dotprompt)?;

        let params = [vec!["--".to_string()], self.prompt_args.clone()].concat();
        let matches = command.get_matches_from(params);

        self.exec_prompt(
            inp, out,
            store, &dotprompt, &appconfig, lb, &matches)
    }
}
