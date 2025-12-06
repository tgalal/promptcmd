use clap::{Parser};
use anyhow::Result;


#[derive(Parser)]
pub struct RunCmd {
    #[arg()]
    pub promptname: String,

    #[arg(long, short, help="Dry run" )]
    pub dryrun: bool
}


pub fn exec(_promptname: &str, _dryrun: bool) -> Result<()> {
    Ok(())
}
