use promptcmd::config::providers::ToLLMProvider;
use promptcmd::dotprompt::DotPrompt;
use promptcmd::dotprompt::render::Render;
use promptcmd::config::{appconfig_locator, providers, promptfile_locator};
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

const BIN_NAME: &str = "promptbox";

fn main() -> Result<()> {
    env_logger::init();

    // Load config
    let appconfig_path = appconfig_locator::path().context("No config found")?;
    debug!("Config Path: {}",appconfig_path.display());
    let appconfig: AppConfig = AppConfig::try_from(&fs::read_to_string(&appconfig_path)?)?;

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

    let promptname = if invoked_binname == BIN_NAME {
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
        providers::ProviderVariant::OpenAi(_) => {
            bail!("OpenAI not yet supported")

        },
        providers::ProviderVariant::None => {
            bail!("No configuration found for the selected provider")
        }
    };
    let llm = provider_config.llm_provider(llmbuilder, &appconfig.providers)
        .expect("Failed to build LLMProvider");

    // Prepare conversation history with example messages
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
