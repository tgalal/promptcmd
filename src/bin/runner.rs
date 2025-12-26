use promptcmd::config::{self, appconfig_locator};
use promptcmd::config::appconfig::{AppConfig};
use promptcmd::dotprompt::DotPrompt;
use promptcmd::cmd::run::{self, RunCmd};
use promptcmd::stats::rusqlite_store::RusqliteStore;
use promptcmd::storage::promptfiles_fs::{FileSystemPromptFilesStorage};
use clap::{Arg, Command};
use promptcmd::storage::PromptFilesStorage;
use std::{env};
use anyhow::{Context, Result, bail};
use std::path::PathBuf;
use std::fs;
use log::{ debug};


fn main() -> Result<()> {
    env_logger::init();

    let prompts_storage = FileSystemPromptFilesStorage::new(
        config::prompt_storage_dir()?
    );

    let mut store = RusqliteStore::new(
        config::data_dir()?
    )?;

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


    if let Some(path) = prompts_storage.exists(&promptname) {
        debug!("Promptfile path: {path}");

        let (_, promptdata) = prompts_storage.load(&promptname)?;

        let dotprompt: DotPrompt = DotPrompt::try_from(promptdata.as_str())?;

        command = run::generate_arguments_from_dotprompt(command, &dotprompt)?;

        let matches = command.get_matches();

        let runcmd = RunCmd {
            promptname,
            dryrun: false,
            prompt_args: Vec::new()
        };

        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        let mut stdout = std::io::stdout();

        runcmd.exec_prompt(
            &mut handle,
            &mut stdout,
            &mut store,
            &dotprompt,
            &appconfig,
            &matches)
    } else {
        bail!("Could not find prompt file")
    }
}
