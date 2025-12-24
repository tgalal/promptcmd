use clap::{Parser};
use anyhow::{Result, bail};
use crate::{storage::PromptFilesStorage};


#[derive(Parser)]
pub struct CatCmd {
    #[arg()]
    pub promptname: String,
}

pub fn exec(storage: &impl PromptFilesStorage,  promptname: &str) -> Result<()> {

    if storage.exists(promptname).is_none() {
        bail!("Could not find a prompt with the name \"{promptname}\"");
    }

    let promptdata = storage.load(promptname)?.1;
    let printable = String::from_utf8_lossy(&promptdata);
    println!("{printable}");

    Ok(())
}
