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

    let symlink_path = bin_locator::path(Some(&promptname)).context("Could not determine link path")?;

    debug!("symlink path: {}", symlink_path.display());

    if !symlink_path.exists() {
        return Ok(());
    }

    let res = remove_symlink_file(&symlink_path);

    if res.is_ok() {
        println!("Removed {}", &symlink_path.display());
    }

    res.map_err(|err| anyhow::anyhow!("{err}"))

}
