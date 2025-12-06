use clap::{Parser};
use anyhow::{Result, bail};
use std::fs;
use crate::config::promptfile_locator;


#[derive(Parser)]
pub struct CatCmd {
    #[arg()]
    pub promptname: String,
}

pub fn exec(promptname: &str) -> Result<()> {
    if let Some(promptfile_path) = promptfile_locator::find(promptname) {
        if let Ok(promptdata) = fs::read_to_string(&promptfile_path) {
            println!("{promptdata}");
        } else {
            bail!("Could not read file {}", promptfile_path.display());
        }
    } else {
        bail!("Could not find a prompt with the name {promptname}");
    }
    Ok(())
}
