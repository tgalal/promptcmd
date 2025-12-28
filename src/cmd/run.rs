use chrono::Utc;
use clap::{Parser};
use log::{error, debug};
use clap::{Arg, ArgMatches, Command};
use std::collections::HashMap;
use std::time::Instant;
use anyhow::{bail, Context, Result};
use llm::{
    builder::{LLMBuilder},
    chat::{ChatMessage},
};
use llm::chat::StructuredOutputFormat;
use tokio::runtime::Runtime;
use crate::config::appconfig::{AppConfig};
use crate::config::{providers};
use crate::dotprompt::DotPrompt;
use crate::dotprompt::render::Render;
use crate::config::providers::ToLLMProvider;
use crate::config::{appconfig_locator,};
use crate::stats::store::{LogRecord, StatsStore};
use crate::storage::PromptFilesStorage;
use std::fs;

#[derive(Parser)]
pub struct RunCmd {
    #[arg()]
    pub promptname: String,

    #[arg(long, short, help="Dry run" )]
    pub dryrun: bool,

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

impl RunCmd {
    /*
    * This function locates the given promptfile, and parses it into Dotprompt.
    * It then generates a command line interface matching the input schema from the Dotprompt,
    * then performs  the matching.
    */
    pub fn exec_prompt(&self,
        inp: &mut impl std::io::BufRead,
        out: &mut impl std::io::Write,
        store: &mut impl StatsStore,
        dotprompt: &DotPrompt, appconfig: &AppConfig, matches: &ArgMatches) -> Result<()> {

        let mut extra_args: HashMap<String, String> = HashMap::new();

        if dotprompt.template_needs_stdin() {
            let mut buffer = String::new();
            inp.read_to_string(&mut buffer)
                .context("Failed to read stdin")?;
            extra_args.insert(String::from("STDIN"), buffer);
        }

        let output: String = dotprompt.render(matches, &extra_args)?;

        debug!("{output}");

        let resolved_model_name = appconfig.resolve_model_name(&dotprompt.frontmatter.model, true)?;

        let (provider, model) = (&resolved_model_name[0].provider, &resolved_model_name[0].model);

        debug!("Model Provider: {}, Model Name: {}", provider, model);

        let mut llmbuilder= LLMBuilder::new()
            .model(model);

        if dotprompt.output_format() == "json" {
            let output_schema: StructuredOutputFormat = serde_json::from_str(
                dotprompt.output_to_extract_structured_json("").as_str())?;
            llmbuilder = llmbuilder.schema(output_schema);
        }

        let provider_config: &dyn ToLLMProvider =  match appconfig.providers.resolve(provider) {
            providers::ProviderVariant::Ollama(conf) => conf,
            providers::ProviderVariant::Anthropic(conf) => conf,
            providers::ProviderVariant::Google(conf) => conf,
            providers::ProviderVariant::OpenAi(conf) => conf,
            providers::ProviderVariant::OpenRouter(conf) => conf,
            providers::ProviderVariant::None => {
                bail!("No configuration found for the selected provider: {}", provider);
            }
        };

        let llm = provider_config.llm_provider(llmbuilder, &appconfig.providers)?;

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
                provider: provider.to_string(),
                model: model.to_string(),
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
        store: &mut impl StatsStore,
        prompt_storage: &impl PromptFilesStorage) -> Result<()> {

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
            store, &dotprompt, &appconfig, &matches)
    }
}
