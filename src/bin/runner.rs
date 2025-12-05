use aibox::dotprompt::dotprompt::DotPrompt;
use clap::{Arg, Command};
use std::{env};
use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::fs;
use handlebars::Handlebars;
use std::collections::HashMap;
use std::io::{self, Read};
use tokio::runtime::Runtime;
use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::{ChatMessage, ChatRole},
};

use aibox::config::locator;
use aibox::dotprompt::{self, Frontmatter};
use log::{info, warn, error, debug};
use env_logger;

const BIN_NAME: &str = "promptbox";

fn main() -> Result<()> {
    env_logger::init();

    let mut args = env::args();

    let path: PathBuf = args
        .next()
        .context("Could not figure binary name")?
        .into();

    let invoked_binname = path
        .file_name()
        .context("Could not get filename")?
        .to_string_lossy()
        .to_string();


    debug!("Invoked bin is the following: {invoked_binname}");

    let mut command: Command = Command::new(&invoked_binname)
        .version("1.0");

    let promptname = if invoked_binname == BIN_NAME {
        // Not running: via symlink, first positional argument is the prompt name
        command = command.arg(Arg::new("promptname"));
        args
            .next()
            .context("Could not determine prompt name")?

    } else {
        invoked_binname
    };
    
    debug!("Prompt name: {promptname}");



    let dotprompt: DotPrompt = match locator::find_promptfile(&promptname) {
        Some(path) => {
            debug!("Loading config from {}", path.display());
            let content = fs::read_to_string(path)?;
            DotPrompt::try_from(content)?
        },
        None => bail!("No config")
    };

    let inputschema = dotprompt.input_schema();

    for (_, inputschema_element) in inputschema {

        let arg = if inputschema_element.data_type == "bool" {
            Arg::new(inputschema_element.key.clone())
            .long(inputschema_element.key.clone())
            .help(inputschema_element.description.clone())
            .required(inputschema_element.required)
            .action(clap::ArgAction::SetTrue)
        } else {
            Arg::new(inputschema_element.key.clone())
            .long(inputschema_element.key.clone())
            .help(inputschema_element.description.clone())
            .required(inputschema_element.required)

        };
        
        command = command.arg(arg);
    }

    let matches = command.get_matches();


    let inputschema = dotprompt.input_schema();
    let mut handlebar_maps: HashMap<String, String> = HashMap::new();
    for (_, ele) in inputschema {
        let value = if ele.data_type == "bool" {
            match matches.get_one::<bool>(&ele.key) {
                Some(value) => {
                    value.to_string()
                },
                None => {
                    String::from("")
                }
            }
        } else {
            match matches.get_one::<String>(&ele.key) {
                Some(value) => {
                    value.to_string()
                },
                None => {
                    String::from("")
                }
            }
        };
        handlebar_maps.insert(ele.key.to_string(), value);
    }

    if dotprompt.template_needs_stdin() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read stdin")?;
        handlebar_maps.insert(String::from("STDIN"), buffer);
    }

    let mut hbs = Handlebars::new();
    hbs.register_template_string("prompt", &dotprompt.template)
        .unwrap();

    let output = hbs.render("prompt", &handlebar_maps)
        .context("Failed to parse template")?;

    debug!("{output}");

    let model_info = dotprompt.model_info().context("Failed to parse model")?;

    let (backend, base_url) = if model_info.provider == "ollama" {
        let baseurl = std::env::var("OLLAMA_HOST").context("Provider is ollama but OLLAMA_HOST not set")?;
        (LLMBackend::Ollama, baseurl)
    } else {
        bail!("Unsupported provider: {}", model_info.provider)
    };

    // Ollama Example
        // Initialize and configure the LLM client
    let llm = LLMBuilder::new()
        .backend(backend) // Use Ollama as the LLM backend
        .base_url(base_url) // Set the Ollama server URL
        .model(&model_info.model_name) // Use the Mistral model
        .max_tokens(1000) // Set maximum response length
        .temperature(0.7) // Control response randomness (0.0-1.0)
        .stream(false) // Disable streaming responses
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
