use clap::{Arg, Command};
use promptcmd::config::{self, appconfig_locator};
use promptcmd::config::appconfig::{AppConfig};
use promptcmd::cmd::run;
use promptcmd::dotprompt::renderers::argmatches::DotPromptArgMatches;
use promptcmd::dotprompt::DotPrompt;
use promptcmd::executor::{Executor, PromptInputs};
use promptcmd::lb::WeightedLoadBalancer;
use promptcmd::stats::rusqlite_store::RusqliteStore;
use promptcmd::stats::store::StatsStore;
use promptcmd::storage::promptfiles_fs::{FileSystemPromptFilesStorage};
use promptcmd::storage::PromptFilesStorage;
use std::sync::{Arc, Mutex};
use std::{env};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::io::{self, Read};
use std::fs;
use log::debug;


fn main() -> Result<()> {
    env_logger::init();

    let prompts_storage = FileSystemPromptFilesStorage::new(
        config::prompt_storage_dir()?
    );

    let stats_store = RusqliteStore::new(
        config::base_storage_dir()?
    )?;

    let appconfig = if let Some(appconfig_path) = appconfig_locator::path() {
        debug!("Config Path: {}",appconfig_path.display());
        AppConfig::try_from(
            fs::read_to_string(&appconfig_path)?.as_str()
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
        .file_stem()
        .context("Could not get filename")?
        .to_string_lossy()
        .into();

    debug!("Executable name: {invoked_binname}");

    let mut command: Command = Command::new(&invoked_binname);
    let promptname = if invoked_binname == config::RUNNER_BIN_NAME {
        // Not running: via symlink, first positional argument is the prompt name or path
        command = command.arg(Arg::new("promptname"));
        args
            .next()
            .context("Could not determine prompt name")?

    } else {
        // if the executable name differs from BIN_NAME, then this might be symlink
        // TODO: check!
        invoked_binname
    };
    /////
    debug!("Prompt name: {promptname}");
    let (_, promptdata) = prompts_storage.load(&promptname)?;
    let dotprompt: DotPrompt = DotPrompt::try_from(promptdata.as_str())?;

    command = run::generate_arguments_from_dotprompt(command, &dotprompt)?;
    command = command.arg(Arg::new("dry")
        .long("dry")
        .help("Dry run")
        .action(clap::ArgAction::SetTrue)
        .required(false));

    let matches = command.get_matches();

    let arc_statsstore: Arc<Mutex<dyn StatsStore + Send>> = Arc::new(Mutex::new(stats_store));
    let lb = WeightedLoadBalancer {
        stats: Arc::clone(&arc_statsstore)
    };
    let arc_prompts_storage = Arc::new(Mutex::new(prompts_storage));
    let arc_appconfig = Arc::new(appconfig);
    let executor = Executor {
        loadbalancer: lb,
        appconfig: arc_appconfig,
        statsstore: arc_statsstore,
        prompts_storage: arc_prompts_storage
    };

    let arc_executor = Arc::new(executor);


    let stdin = if dotprompt.template_needs_stdin() {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)
            .context("Failed to read stdin")?;
        Some(buffer)
    } else {
        None
    };

    let dry = *matches.get_one::<bool>("dry").unwrap_or(&false);

    let argmatches = DotPromptArgMatches {
        matches,
        stdin,
        dotprompt: &dotprompt
    };

    let inputs: PromptInputs = argmatches.try_into()?;


    let result = arc_executor.execute_dotprompt(&dotprompt, inputs, dry)?;
    println!("{}", result);

    Ok(())
}
