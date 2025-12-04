use clap::{Parser};
use log::{debug};
use symlink::remove_symlink_file;
use anyhow::{Context, Result};
use dirs;


#[derive(Parser)]
pub struct DisableCmd {
    #[arg()]
    promptname: String,
}

pub fn exec(cmd: DisableCmd) -> Result<()> {
    let promptname = cmd.promptname;


    let home_dir = dirs::home_dir().context("Coud not determine home dir")?;
    let symlink_path = home_dir.join(".local/bin").join(promptname);

    debug!("symlink path: {}", symlink_path.display());

    if !symlink_path.exists() {
        return Ok(());
    }
    println!("Disabling {}", symlink_path.display());
    remove_symlink_file(symlink_path).map_err(|err| anyhow::anyhow!("{err}"))

}
