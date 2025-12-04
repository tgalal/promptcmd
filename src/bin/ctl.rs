use anyhow::Result;
use aibox::cmd::edit;
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
    Edit(edit::EditCmd)
    // Run(run_cmd::RunCmd),
    // Read(read_cmd::ReadCmd),
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Edit(cmd) => edit::exec(cmd)
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
