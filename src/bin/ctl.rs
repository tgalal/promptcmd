use anyhow::Result;
use promptcmd::cmd;
use promptcmd::cmd::BasicTextEditor;
use promptcmd::config::appconfig::AppConfig;
use promptcmd::config::{self, appconfig_locator, RUNNER_BIN_NAME};
use promptcmd::executor::Executor;
use promptcmd::lb::WeightedLoadBalancer;
use promptcmd::stats::rusqlite_store::RusqliteStore;
use std::env;
use std::sync::{Arc, OnceLock};
use promptcmd::installer::symlink::SymlinkInstaller;
use promptcmd::storage::promptfiles_fs::{FileSystemPromptFilesStorage};
use std::fs;
use std::io::{self, BufReader};
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

    #[clap(about = "Render prompts without API calls")]
    Render(cmd::render::RenderCmd),
}

static PROMPTS_STORAGE: OnceLock<FileSystemPromptFilesStorage> = OnceLock::new();
static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();
static STATS_STORE: OnceLock<RusqliteStore> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    config::bootstrap_directories()?;



    let prompt_storage_path = config::prompt_storage_dir()?;
    let base_home_dir = config::base_home_dir()?;
    let prompts_storage = PROMPTS_STORAGE.get_or_init(||
        FileSystemPromptFilesStorage::new(prompt_storage_path)
    );
    let statsstore = STATS_STORE.get_or_init(||
        match RusqliteStore::new(base_home_dir) {
            Ok(store) => store,
            Err(err) => panic!("{}", err)
        }
    );

    let appconfig = if let Some(appconfig_path) = appconfig_locator::path() {
        let appconfig_data = fs::read_to_string(&appconfig_path)?;

        APP_CONFIG.get_or_init(||
            match AppConfig::try_from(appconfig_data.as_str()) {
                Ok(appconfig) => appconfig,
                Err(err) => panic!("Failed to initialize: {}", err)
            }
        )
    } else {
        APP_CONFIG.get_or_init(AppConfig::default)
    };

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

    let cli = Cli::parse();

    match cli.command {
        Commands::Edit(cmd) => cmd.exec(
                &mut BufReader::new(io::stdin()),
                &mut std::io::stdout(),
                prompts_storage,
                &editor,),
        Commands::Enable(cmd) => cmd.exec(
                prompts_storage,
                &mut installer),
        Commands::Disable(cmd) => cmd.exec(&mut installer),
        Commands::Create(cmd) => cmd.exec(
                &mut BufReader::new(io::stdin()),
                &mut std::io::stdout(),
                prompts_storage,
                &mut installer,
                &editor,
                appconfig),
        Commands::List(cmd) => cmd.exec(prompts_storage),
        Commands::Cat(cmd) => cmd.exec(
                prompts_storage,
                &mut std::io::stdout()),
        Commands::Run(cmd) => {
                let lb = WeightedLoadBalancer {
                    stats: statsstore
                };
                let executor = Executor {
                    loadbalancer: lb,
                    appconfig,
                    statsstore,
                    prompts_storage
                };
                let executor_arc = Arc::new(executor);
                cmd.exec(
                    executor_arc
                ).await
            },
        Commands::Import(cmd) => cmd.exec(
                prompts_storage,
                &mut installer,
                appconfig
            ),
        Commands::Stats(cmd) => cmd.exec(
                statsstore
            ),
        Commands::Resolve(cmd) => cmd.exec(
                appconfig,
                &mut std::io::stdout(),
            ),
        Commands::Config(cmd) => cmd.exec(
                &mut BufReader::new(io::stdin()),
                &mut std::io::stdout(),
                &editor
            ),
        Commands::Render(cmd) => cmd.exec(
                prompts_storage,
                &mut std::io::stdout(),
                &editor
            ),
    }
}
