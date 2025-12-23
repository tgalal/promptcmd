use clap::{Parser};
use log::{error, debug};
use clap::{Arg, ArgMatches, Command};
use std::io::{self, Read};
use std::collections::HashMap;
use anyhow::{bail, Context, Result};
use llm::{
    builder::{LLMBuilder},
    chat::{ChatMessage},
};
use llm::chat::StructuredOutputFormat;
use std::path::PathBuf;
use tokio::runtime::Runtime;
use crate::config::appconfig::{AppConfig};
use crate::config::{promptfile_locator, providers};
use crate::dotprompt::DotPrompt;
use crate::dotprompt::render::Render;
use crate::config::providers::ToLLMProvider;
use crate::config::{appconfig_locator,};
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

/*
* This function locates the given promptfile, and parses it into Dotprompt.
* It then generates a command line interface matching the input schema from the Dotprompt,
* then performs  the matching.
*/
pub fn exec_prompt(dotprompt: &DotPrompt, appconfig: &AppConfig, matches: &ArgMatches) -> Result<()> {

    // command = generate_arguments_from_dotprompt(command, &dotprompt)?;

    // let matches: ArgMatches = command.get_matches();

    let mut extra_args: HashMap<String, String> = HashMap::new();

    if dotprompt.template_needs_stdin() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read stdin")?;
        extra_args.insert(String::from("STDIN"), buffer);
    }
    
    let output: String = dotprompt.render(matches, &extra_args)?;

    debug!("{output}");

    let model_info = dotprompt.model_info().context("Failed to parse model")?;

    debug!("Model Provider: {}, Model Name: {}", &model_info.provider, &model_info.model_name);

    let mut llmbuilder= LLMBuilder::new()
        .model(&model_info.model_name);

    if dotprompt.output_format() == "json" {
        let output_schema: StructuredOutputFormat = serde_json::from_str(
            dotprompt.output_to_extract_structured_json("").as_str())?;
        llmbuilder = llmbuilder.schema(output_schema);
    }

    let provider_config: &dyn ToLLMProvider =  match appconfig.providers.resolve(&model_info.provider) {
        providers::ProviderVariant::Ollama(conf) => conf,
        providers::ProviderVariant::Anthropic(conf) => conf,
        providers::ProviderVariant::Google(conf) => conf,
        providers::ProviderVariant::OpenAi(conf) => conf,
        providers::ProviderVariant::OpenRouter(conf) => conf,
        providers::ProviderVariant::None => {
            bail!("No configuration found for the selected provider: {}", model_info.provider);
        }
    };

    let llm = provider_config.llm_provider(llmbuilder, &appconfig.providers)?;

    let messages = vec![
        ChatMessage::user()
            .content(output)
            .build(),
    ];
    let rt = Runtime::new().unwrap();
    let result = rt.block_on(llm.chat(&messages));
        // Send chat request and handle the response
    match result  {
        Ok(text) => println!("{text}"),
        Err(e) => error!("Chat error: {e}"),
    }
    
    Ok(())
}

pub fn exec(promptname: &str, _: bool, rest: Vec<String>) -> Result<()> {
    let appconfig = if let Some(appconfig_path) = appconfig_locator::path() {
        debug!("Config Path: {}",appconfig_path.display());
        AppConfig::try_from(
            &fs::read_to_string(&appconfig_path)?
        )?
    } else {
        AppConfig::default()
    };

    debug!("Prompt name: {promptname}");

    let promptfile_path: PathBuf = promptfile_locator::find(promptname)
        .context("Could not find promptfile")?;

    debug!("Promptfile path: {}", promptfile_path.display());

    let dotprompt: DotPrompt = DotPrompt::try_from(fs::read_to_string(&promptfile_path)?.as_str())?;

    let mut command: Command = Command::new(promptname.to_string());

    command = generate_arguments_from_dotprompt(command, &dotprompt)?;
    let params = [vec!["--".to_string()], rest].concat();
    let matches = command.get_matches_from(params);

    exec_prompt(&dotprompt, &appconfig, &matches)
}
