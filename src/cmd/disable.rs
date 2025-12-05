use clap::{Parser};
use log::{debug};
use symlink::remove_symlink_file;
use anyhow::{Context, Result};

use crate::config::bin_locator;

#[derive(Parser)]
pub struct DisableCmd {
    #[arg()]
    promptname: String,
}

pub fn exec(cmd: DisableCmd) -> Result<()> {
    let promptname = cmd.promptname;


    let symlink_path = bin_locator::path(&promptname).context("Could not determine link path")?;

    debug!("symlink path: {}", symlink_path.display());

    if !symlink_path.exists() {
        return Ok(());
    }
    println!("Disabling {}", symlink_path.display());
    remove_symlink_file(symlink_path).map_err(|err| anyhow::anyhow!("{err}"))

}
