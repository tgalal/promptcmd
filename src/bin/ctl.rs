use anyhow::Result;
use promptcmd::cmd;
use promptcmd::cmd::BasicTextEditor;
use promptcmd::config::{self, RUNNER_BIN_NAME};
use promptcmd::executor::Executor;
use promptcmd::lb::WeightedLoadBalancer;
use promptcmd::stats::rusqlite_store::RusqliteStore;
use promptcmd::stats::store::StatsStore;
use std::env;
use std::sync::{Arc, Mutex};
use promptcmd::installer::symlink::SymlinkInstaller;
use promptcmd::storage::promptfiles_fs::{FileSystemPromptFilesStorage};
use std::fs;
use std::io::{self, BufReader};
use log::debug;
use clap::{Parser, Subcommand};
use anyhow::Context;


#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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

    #[clap(about = "Import promptfile", alias = "i")]
    Import(cmd::import::ImportCmd),

    #[clap(about = "Print statistics")]
    Stats(cmd::stats::StatsCmd),

    #[clap(about = "Resolve model name")]
    Resolve(cmd::resolve::ResolveCmd),

    #[clap(about = "Display and edit your config.toml")]
    Config(cmd::config::ConfigCmd),
}

fn main() -> Result<()> {
    env_logger::init();
    config::bootstrap_directories()?;

    let mut prompts_storage = FileSystemPromptFilesStorage::new(
        config::prompt_storage_dir()?
    );

    let stats_store = RusqliteStore::new(
        config::base_home_dir()?
    )?;

    let runner_binary_name = RUNNER_BIN_NAME;

    #[cfg(target_os="windows")]
    let runner_binary_name = runner_binary_name.to_string() + ".exe";

    let target_bin = env::current_exe()
        .context("Could not determine current bin")?
        .parent()
        .context("Could not determine parent of current bin")?
        .join(runner_binary_name);

    let mut installer = SymlinkInstaller::new(
        target_bin,
        config::prompt_install_dir()?
    );

    let editor = BasicTextEditor {};

    let appconfig = if let Some(appconfig_path) = config::appconfig_locator::path() {
        debug!("Config Path: {}",appconfig_path.display());
        config::appconfig::AppConfig::try_from(
            fs::read_to_string(&appconfig_path)?.as_str()
        )?
    } else {
        config::appconfig::AppConfig::default()
    };

    let cli = Cli::parse();

    match cli.command {
        Commands::Edit(cmd) => cmd.exec(
            &mut BufReader::new(io::stdin()),
            &mut std::io::stdout(),
            &mut prompts_storage,
            &editor,),

        Commands::Enable(cmd) => cmd.exec(
            &prompts_storage,
            &mut installer),

        Commands::Disable(cmd) => cmd.exec(&mut installer),

        Commands::Create(cmd) => cmd.exec(
            &mut BufReader::new(io::stdin()),
            &mut std::io::stdout(),
            &mut prompts_storage,
            &mut installer,
            &editor,
            &appconfig),

        Commands::List(cmd) => cmd.exec(&prompts_storage),

        Commands::Cat(cmd) => cmd.exec(
            &prompts_storage,
            &mut std::io::stdout()),

        Commands::Run(cmd) => {
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
            let executor_arc = Arc::new(executor);
            cmd.exec(
                &mut BufReader::new(io::stdin()),
                executor_arc
            )
        },

        Commands::Import(cmd) => cmd.exec(
            &mut prompts_storage,
            &mut installer,
            &appconfig
        ),

        Commands::Stats(cmd) => cmd.exec(
            &stats_store
        ),

        Commands::Resolve(cmd) => cmd.exec(
            &appconfig,
            &mut std::io::stdout(),
        ),

        Commands::Config(cmd) => cmd.exec(
            &mut BufReader::new(io::stdin()),
            &mut std::io::stdout(),
            &editor
        ),
    }
}
