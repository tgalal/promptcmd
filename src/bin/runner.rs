use aibox::dotprompt::DotPrompt;
use aibox::dotprompt::render::Render;
use aibox::config::{appconfig, appconfig_locator, providers};
use aibox::config::appconfig::{AppConfig};
use clap::{Arg, ArgMatches, Command};
use std::{env};
use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use std::io::{self, Read};
use tokio::runtime::Runtime;
use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::{ChatMessage},
};

use aibox::config::promptfile_locator;
use log::{error, debug};

const BIN_NAME: &str = "promptbox";

fn main() -> Result<()> {
    env_logger::init();

    // Load config
    let appconfig_path = appconfig_locator::path().context("No config found")?;
    debug!("Loading configuration from {}",appconfig_path.display());
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

    let promptfile_path: PathBuf = promptfile_locator::find(&promptname)
        .context("Could not find promptfile")?;

    debug!("Loading {}", promptfile_path.display());
    let dotprompt: DotPrompt = DotPrompt::try_from(fs::read_to_string(&promptfile_path)?)?;

    debug!("Loaded {}", promptfile_path.display());

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

    let mut llm_builder= LLMBuilder::new()
        .model(&model_info.model_name)
        .max_tokens(1000) // Set maximum response length
        .temperature(0.7) // Control response randomness (0.0-1.0)
        .stream(false);  // Disable streaming responses

    llm_builder = match appconfig.providers.resolve(&model_info.provider) {
        providers::ProviderVariant::Ollama(ollamaconf) => {
            debug!("Working with the following ollama conf: {}", toml::to_string(ollamaconf).unwrap());
            llm_builder.backend(LLMBackend::Ollama)
               .base_url(&ollamaconf.endpoint)
                .max_tokens(ollamaconf.max_tokens(&appconfig.providers))
                .stream(ollamaconf.stream(&appconfig.providers))
                .temperature(ollamaconf.temperature(&appconfig.providers))
        },
        providers::ProviderVariant::OpenAi(openaiconf) => {
            debug!("Working with the following openai conf: {}", toml::to_string(openaiconf).unwrap());
            bail!("OpenAI not yet supported")

        },
        providers::ProviderVariant::Anthropic(anthropicconf) => {
            debug!("Working with the following anthropicconf conf: {}", toml::to_string(anthropicconf).unwrap());
            llm_builder.backend(LLMBackend::Anthropic)
            .api_key(&anthropicconf.api_key)
            .max_tokens(anthropicconf.max_tokens(&appconfig.providers))
            .stream(anthropicconf.stream(&appconfig.providers))
            .temperature(anthropicconf.temperature(&appconfig.providers))
        }
        providers::ProviderVariant::None => {
            bail!("No configuration found for the selected provider")
        }
    };
    println!("{}", toml::to_string(&appconfig).unwrap());

    let llm = llm_builder
        .build()
        .expect("Failed to build LLM model");

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
