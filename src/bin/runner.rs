use aibox::dotprompt::DotPrompt;
use aibox::dotprompt::render::Render;
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

    // First figure out the current execution method
    // This could be one of:
    // 1. Direct runner binary
    // 2. symlink to runner
    // 3. shebang (TODO)

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
        .context("Could not find promptfile with name: {promptname}")?;

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
