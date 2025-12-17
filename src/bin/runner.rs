use promptcmd::config::providers::ToLLMProvider;
use promptcmd::dotprompt::DotPrompt;
use promptcmd::dotprompt::render::Render;
use promptcmd::config::{self, appconfig_locator, promptfile_locator, providers};
use promptcmd::config::appconfig::{AppConfig};
use clap::{Arg, ArgMatches, Command};
use llm::chat::StructuredOutputFormat;
use std::{env};
use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use std::io::{self, Read};
use tokio::runtime::Runtime;
use llm::{
    builder::{LLMBuilder},
    chat::{ChatMessage},
};

use log::{error, debug};

fn main() -> Result<()> {
    env_logger::init();


    let appconfig = if let Some(appconfig_path) = appconfig_locator::path() {
        debug!("Config Path: {}",appconfig_path.display());
        AppConfig::try_from(
            &fs::read_to_string(&appconfig_path)?
        )?
    } else {
        AppConfig::default()
    };

    // Find the executable name directly from args.
    let mut args = env::args();

    let path: PathBuf = args
        .next()
        .context("Could not figure binary name")?
        .into();
 
    let invoked_binname: String = path
        .file_name()
        .context("Could not get filename")?
        .to_string_lossy()
        .into();

    debug!("Executable name: {invoked_binname}");

    let mut command: Command = Command::new(&invoked_binname);

    let promptname = if invoked_binname == config::RUNNER_BIN_NAME {
        // Not running: via symlink, first positional argument is the prompt name
        command = command.arg(Arg::new("promptname"));
        args
            .next()
            .context("Could not determine prompt name")?

    } else {
        // if the executable name differs from BIN_NAME, then this might be symlink
        // TODO: check!
        invoked_binname
    };
    debug!("Prompt name: {promptname}");

    let promptfile_path: PathBuf = promptfile_locator::find(promptname.as_str())
        .context("Could not find promptfile")?;

    debug!("Promptfile path: {}", promptfile_path.display());
    let dotprompt: DotPrompt = DotPrompt::try_from(fs::read_to_string(&promptfile_path)?.as_str())?;

    let args: Vec<Arg> = Vec::try_from(&dotprompt).context(
        "Could not generate arguments"
    )?;

    for arg in args {
       command = command.arg(arg);
    }

    let matches:ArgMatches = command.get_matches();

    let mut extra_args: HashMap<String, String> = HashMap::new();

    if dotprompt.template_needs_stdin() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read stdin")?;
        extra_args.insert(String::from("STDIN"), buffer);
    }
    
    let output: String = dotprompt.render(&matches, &extra_args)?;

    debug!("{output}");

    let model_info = dotprompt.model_info().context("Failed to parse model")?;

    debug!("Model Provider: {}, Model Name: {}", &model_info.provider, &model_info.model_name);

    let mut llmbuilder= LLMBuilder::new()
        .model(&model_info.model_name);

    if dotprompt.output_format() == "json" {
        let output_schema: StructuredOutputFormat = serde_json::from_str(
            dotprompt.output_to_extract_structured_json(promptname.as_str()).as_str())?;
        llmbuilder = llmbuilder.schema(output_schema);
    }

    let provider_config: &dyn ToLLMProvider=  match appconfig.providers.resolve(&model_info.provider) {
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
