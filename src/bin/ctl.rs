use anyhow::Result;
use aibox::cmd;
use clap::{Parser, Subcommand};
use log::{info, warn, error, debug};
use env_logger;


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

    #[clap(about = "List commands and prompts")]
    Ls(cmd::list::ListCmd),
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Edit(cmd) => cmd::edit::exec(cmd),
        Commands::Enable(cmd) => cmd::enable::exec(&cmd.promptname),
        Commands::Disable(cmd) => cmd::disable::exec(cmd),
        Commands::Create(cmd) => cmd::create::exec(&cmd.promptname, cmd.now),
        Commands::Ls(cmd) => cmd::list::exec(
            cmd.long, cmd.enabled, cmd.disabled, cmd.fullpath, cmd.commands
        ),
    }
    // match cli.command {
    //     Commands::Run(cmd) => {
    //         run_cmd::exec(cmd)
    //     },
    //     Commands::Read(cmd) => {
    //         read_cmd::exec(cmd)
    //     },
    // }
}
