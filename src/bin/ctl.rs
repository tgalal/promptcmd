use anyhow::Result;
use promptcmd::cmd;
use promptcmd::config;
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

    let cli = Cli::parse();
    match cli.command {
        Commands::Edit(cmd) => cmd::edit::exec(cmd),
        Commands::Enable(cmd) => cmd::enable::exec(&cmd.promptname),
        Commands::Disable(cmd) => cmd::disable::exec(cmd),
        Commands::Create(cmd) => cmd::create::exec(&cmd.promptname, cmd.now),
        Commands::New(cmd) => cmd::create::exec(&cmd.promptname, cmd.now),
        Commands::Ls(cmd) => cmd::list::exec(
            cmd.long, cmd.enabled, cmd.disabled, cmd.fullpath, cmd.commands
        ),
        Commands::List(cmd) => cmd::list::exec(
            cmd.long, cmd.enabled, cmd.disabled, cmd.fullpath, cmd.commands
        ),
        Commands::Cat(cmd) => cmd::cat::exec(&cmd.promptname),
        Commands::Run(cmd) => cmd::run::exec(&cmd.promptname, cmd.dryrun),
        Commands::Import(cmd) => cmd::import::exec(
            cmd.promptname,
            cmd.promptfile,
            cmd.enable,
            cmd.force,
        )
    }
}
