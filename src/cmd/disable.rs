use clap::{Parser};
use anyhow::{Context, Result};

use crate::{installer::DotPromptInstaller};

#[derive(Parser)]
pub struct DisableCmd {
    #[arg()]
    promptname: String,
}

pub fn exec(
    installer: &mut impl DotPromptInstaller,
    cmd: DisableCmd) -> Result<()> {

    let promptname = cmd.promptname;

    if let Some(path) = installer.is_installed(&promptname) {
        installer.uninstall(&promptname).context("Failed to uninstalled prompt")?;
        println!("Removed {path}");
    } 

    Ok(())
}
