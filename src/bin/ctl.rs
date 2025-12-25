use anyhow::Result;
use promptcmd::cmd;
use promptcmd::cmd::BasicTextEditor;
use promptcmd::config;
use promptcmd::storage::promptfiles_fs::{FileSystemPromptFilesStorage};
use std::fs;
use log::debug;
use clap::{Parser, Subcommand};


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

    #[clap(about = "Create a new prompt file")]
    Create(cmd::create::CreateCmd),
 
    #[clap(about = "Create a new prompt file")]
    New(cmd::create::CreateCmd),

    #[clap(about = "List commands and prompts")]
    Ls(cmd::list::ListCmd),

    #[clap(about = "List commands and prompts")]
    List(cmd::list::ListCmd),
 
    #[clap(about = "Print promptfile contents")]
    Cat(cmd::cat::CatCmd),
 
    #[clap(about = "Run promptfile")]
    Run(cmd::run::RunCmd),

    #[clap(about = "Import promptfile")]
    Import(cmd::import::ImportCmd),
}

fn main() -> Result<()> {
    env_logger::init();
    config::bootstrap_directories()?;

    let mut prompts_storage = FileSystemPromptFilesStorage::new(
        config::prompt_storage_dir()?
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

    match cli.command {
        Commands::Edit(cmd) => cmd::edit::exec(
            &mut handle, &mut stdout,
            &mut prompts_storage, &appconfig, cmd),

        Commands::Enable(cmd) => cmd::enable::exec(&prompts_storage, &cmd.promptname),
        Commands::Disable(cmd) => cmd::disable::exec(cmd),

        Commands::Create(cmd) => cmd::create::exec(
            &mut handle, &mut stdout,
            &mut prompts_storage, &editor,
            &appconfig, &cmd.promptname, cmd.now, cmd.force),

        Commands::New(cmd) => cmd::create::exec(
            &mut handle, &mut stdout,
            &mut prompts_storage, &editor,
            &appconfig, &cmd.promptname, cmd.now, cmd.force),

        Commands::Ls(cmd) => cmd::list::exec(
            &prompts_storage,   cmd.long, cmd.enabled, cmd.disabled, cmd.fullpath, cmd.commands, cmd.config,
        ),

        Commands::List(cmd) => cmd::list::exec(
            &prompts_storage, cmd.long, cmd.enabled, cmd.disabled, cmd.fullpath, cmd.commands, cmd.config
        ),

        Commands::Cat(cmd) => cmd::cat::exec(&prompts_storage, &cmd.promptname, &mut std::io::stdout()),
        Commands::Run(cmd) => cmd::run::exec(&prompts_storage, &cmd.promptname, cmd.dryrun, cmd.prompt_args),

        Commands::Import(cmd) => cmd::import::exec(
            &mut prompts_storage,
            cmd.promptname,
            cmd.promptfile,
            cmd.enable,
            cmd.force,
        )
    }
}
