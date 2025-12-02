use anyhow::{Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct RunCmd {
    #[command(subcommand)]
    command: RunSubcommands,
}


#[derive(Parser)]
struct TranslateCmd {
    #[arg(long)]
    arg1: bool,

    #[arg(long)]
    arg2: bool,
}
#[derive(Subcommand)]
enum RunSubcommands {
    Translate(TranslateCmd),
}

pub fn exec(run_cmd: RunCmd) -> Result<()> {
    match run_cmd.command {
        RunSubcommands::Translate(t) => {
            println!("Translate! arg1={} arg2={}", t.arg1, t.arg2);
        }
        }
    Ok(())
}
