use clap::{Parser};
use log::{debug};
use std::{env};
use symlink::symlink_file;
use anyhow::{bail, Context, Result};
use dirs;

use crate::config::ConfigLocator;

#[derive(Parser)]
pub struct EnableCmd {
    #[arg(long, default_value_t=false)]
    pub here: bool,
    #[arg()]
    pub promptname: String,
}

pub fn exec(promptname: &String) -> Result<()> {

    let config_filename: String =  format!("{promptname}.prompt");
    let locator: ConfigLocator = ConfigLocator::new("aibox", "prompts.d", config_filename);

    let home_dir = dirs::home_dir().context("Coud not determine home dir")?;
    let symlink_path = home_dir.join(".local/bin").join(promptname);

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

    let res = match locator.find_config() {
        Some(path) => {
            println!("Enabling {}", path.display());
            symlink_file(targetbin, symlink_path)
        },
        None => {
            let paths : Vec<String>= locator
                .get_search_paths().iter().map(|path| path.display().to_string()).collect();

            bail!("Could not find an existing prompt file, searched:\n{}\nConsider creating a new one?", paths.join("\n"))
        }
    };
    res.map_err(|e| anyhow::anyhow!("Failed to create symlink: {e}"))
}
