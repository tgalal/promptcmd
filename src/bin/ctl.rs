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
    /// Edit a resource
    #[clap(about = "Edit an existing prompt file")]
    Edit(cmd::edit::EditCmd),

    /// Enable a resource
    #[clap(about = "Enable a prompt")]
    Enable(cmd::enable::EnableCmd),

    /// Disable a resource
    #[clap(about = "Disable a prompt")]
    Disable(cmd::disable::DisableCmd),

    /// Create a new resource
    #[clap(about = "Create a new prompt file")]
    Create(cmd::create::CreateCmd),
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Edit(cmd) => cmd::edit::exec(cmd),
        Commands::Enable(cmd) => cmd::enable::exec(&cmd.promptname),
        Commands::Disable(cmd) => cmd::disable::exec(cmd),
        Commands::Create(cmd) => cmd::create::exec(&cmd.promptname, cmd.now),
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
