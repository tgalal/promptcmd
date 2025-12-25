use clap::{Parser};
use log::{debug};
use anyhow::{bail, Result};

use crate::{installer::DotPromptInstaller, storage::PromptFilesStorage};

#[derive(Parser)]
pub struct EnableCmd {
    #[arg(long, default_value_t=false)]
    pub here: bool,
    #[arg()]
    pub promptname: String,
}

pub fn exec(storage: &impl PromptFilesStorage, installer: &mut impl DotPromptInstaller,
    promptname: &str) -> Result<()> {

    if let Some(path) = installer.is_installed(promptname) {
        println!("{} is already installed at {}", promptname, &path);
        return Ok(());
    }

    if let Some(path) = storage.exists(promptname) {
        debug!("Enabling {path}");

        let installed_path = installer.install(promptname)?;
        println!("Created {installed_path}");

    } else {
        bail!("Could not find an existing prompt file");
    }

    Ok(())
}
