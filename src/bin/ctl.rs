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
    Edit(cmd::edit::EditCmd),
    Enable(cmd::enable::EnableCmd),
    Disable(cmd::disable::DisableCmd),
    Create(cmd::create::CreateCmd)
    // Run(run_cmd::RunCmd),
    // Read(read_cmd::ReadCmd),
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Edit(cmd) => cmd::edit::exec(cmd),
        Commands::Enable(cmd) => cmd::enable::exec(cmd),
        Commands::Disable(cmd) => cmd::disable::exec(cmd),
        Commands::Create(cmd) => cmd::create::exec(cmd),
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
