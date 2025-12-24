use clap::{Parser};
use log::{debug};
use std::{env};
use symlink::symlink_file;
use anyhow::{bail, Context, Result};

use crate::config::bin_locator;
use crate::config::RUNNER_BIN_NAME;
use crate::storage::PromptFilesStorage;

#[derive(Parser)]
pub struct EnableCmd {
    #[arg(long, default_value_t=false)]
    pub here: bool,
    #[arg()]
    pub promptname: String,
}

pub fn exec(storage: &impl PromptFilesStorage, promptname: &str) -> Result<()> {

    let symlink_path = bin_locator::path(Some(promptname)).context("Could not determine link path")?;

    debug!("symlink path: {}", symlink_path.display());

    if symlink_path.exists() {
        println!("{} already exists", &symlink_path.display());
        return Ok(());
    }

    let currbin_path = env::current_exe().context("Could not determine current bin")?;
    debug!("currbin path: {}", currbin_path.display());

    let targetbin = currbin_path.parent()
        .context("Could not determine parent of current bin")?
        .join(RUNNER_BIN_NAME);

    if !targetbin.exists() {
        bail!("Could not locate target bin");
    }

    if let Some(path) = storage.exists(promptname) {
        debug!("Enabling {path}");

        let res = symlink_file(targetbin, &symlink_path)
            .map_err(|e| anyhow::anyhow!("Failed to create symlink: {e}"));

        if res.is_ok() {
            println!("Created {}", &symlink_path.display());
        } else {
            return res;
        }

    } else {
        bail!("Could not find an existing prompt file");
    }

    Ok(())
}
