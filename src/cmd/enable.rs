use clap::{Parser};
use log::{debug};
use std::{env};
use symlink::symlink_file;
use anyhow::{bail, Context, Result};

use crate::config::bin_locator;
use crate::config::promptfile_locator;

#[derive(Parser)]
pub struct EnableCmd {
    #[arg(long, default_value_t=false)]
    pub here: bool,
    #[arg()]
    pub promptname: String,
}

pub fn exec(promptname: &String) -> Result<()> {
    let symlink_path = bin_locator::path(promptname).context("Could not determine link path")?;

    debug!("symlink path: {}", symlink_path.display());

    if symlink_path.exists() {
        return Ok(());
    }

    let currbin_path = env::current_exe().context("Could not determine current bin")?;
    debug!("currbin path: {}", currbin_path.display());

    let targetbin = currbin_path.parent()
        .context("Could not determine parent of current bin")?
        .join("promptbox");

    if !targetbin.exists() {
        bail!("Could not locate target bin");
    }

    let res = match promptfile_locator::find(promptname) {
        Some(path) => {
            println!("Enabling {}", path.display());
            symlink_file(targetbin, symlink_path)
        },
        None => {
            let paths : Vec<String>= promptfile_locator::search_paths(Some(&promptname))
                .iter().map(|path| path.display().to_string()).collect();

            bail!("Could not find an existing prompt file, searched:\n{}\nConsider creating a new one?", paths.join("\n"))
        }
    };
    res.map_err(|e| anyhow::anyhow!("Failed to create symlink: {e}"))
}
