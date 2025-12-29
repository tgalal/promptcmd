use anyhow::Result;
use promptcmd::cmd;
use promptcmd::cmd::BasicTextEditor;
use promptcmd::config::{self, RUNNER_BIN_NAME};
use promptcmd::lb::weighted_lb::WeightedLoadBalancer;
use promptcmd::stats::rusqlite_store::RusqliteStore;
use std::env;
use promptcmd::installer::symlink::SymlinkInstaller;
use promptcmd::storage::promptfiles_fs::{FileSystemPromptFilesStorage};
use std::fs;
use log::debug;
use clap::{Parser, Subcommand};
use anyhow::Context;


#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    verbose: bool
}

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Edit an existing prompt file")]
    Edit(cmd::edit::EditCmd),

    #[clap(about = "Enable a prompt")]
    Enable(cmd::enable::EnableCmd),

    #[clap(about = "Disable a prompt")]
    Disable(cmd::disable::DisableCmd),

    #[clap(about = "Create a new prompt file", visible_alias = "new")]
    Create(cmd::create::CreateCmd),

    #[clap(about = "List commands and prompts", visible_alias = "ls")]
    List(cmd::list::ListCmd),

    #[clap(about = "Print promptfile contents")]
    Cat(cmd::cat::CatCmd),

    #[clap(about = "Run promptfile")]
    Run(cmd::run::RunCmd),

    #[clap(about = "Import promptfile")]
    Import(cmd::import::ImportCmd),

    #[clap(about = "Print statistics")]
    Stats(cmd::stats::StatsCmd),

    #[clap(about = "Resolve model name")]
    ResolveModel(cmd::resolve_model::ResolveModelCmd),
}

fn main() -> Result<()> {
    env_logger::init();
    config::bootstrap_directories()?;

    let mut prompts_storage = FileSystemPromptFilesStorage::new(
        config::prompt_storage_dir()?
    );

    let mut store = RusqliteStore::new(
        config::data_dir()?
    )?;

    let target_bin = env::current_exe()
        .context("Could not determine current bin")?
        .parent()
        .context("Could not determine parent of current bin")?
        .join(RUNNER_BIN_NAME);

    let mut installer = SymlinkInstaller::new(
        target_bin,
        config::prompt_install_dir()?
    );

    let editor = BasicTextEditor {};

    let appconfig = if let Some(appconfig_path) = config::appconfig_locator::path() {
        debug!("Config Path: {}",appconfig_path.display());
        config::appconfig::AppConfig::try_from(
            &fs::read_to_string(&appconfig_path)?
        )?
    } else {
        config::appconfig::AppConfig::default()
    };

    let cli = Cli::parse();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut stdout = std::io::stdout();
    let lb = WeightedLoadBalancer {};

    match cli.command {
        Commands::Edit(cmd) => cmd.exec(
            &mut handle,
            &mut stdout,
            &mut prompts_storage,
            &editor,),

        Commands::Enable(cmd) => cmd.exec(
            &prompts_storage,
            &mut installer),

        Commands::Disable(cmd) => cmd.exec(&mut installer),

        Commands::Create(cmd) => cmd.exec(
            &mut handle,
            &mut stdout,
            &mut prompts_storage,
            &mut installer,
            &editor,
            &appconfig),

        Commands::List(cmd) => cmd::list::exec(
            &prompts_storage, cmd.long, cmd.enabled, cmd.disabled, cmd.fullpath, cmd.commands, cmd.config
        ),

        Commands::Cat(cmd) => cmd.exec(
            &prompts_storage,
            &mut std::io::stdout()),

        Commands::Run(cmd) => cmd.exec(
            &mut handle,
            &mut stdout,
            &mut store,
            &prompts_storage,
            &lb
        ),

        Commands::Import(cmd) => cmd.exec(
            &mut prompts_storage,
            &mut installer,
        ),

        Commands::Stats(cmd) => cmd.exec(
            &store
        ),

        Commands::ResolveModel(cmd) => cmd.exec(
            &appconfig,
            &mut stdout
        ),
    }
}
