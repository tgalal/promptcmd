use clap::{Parser};
use std::fs;
use log::{debug};
use anyhow::{bail, Result};
use edit;

use crate::config::promptfile_locator;

#[derive(Parser)]
pub struct EditCmd {
    #[arg()]
    promptname: String,
}

pub fn exec(cmd: EditCmd) -> Result<()> {
    let promptname = cmd.promptname;


    match promptfile_locator::find(&promptname) {
        Some(path) => {
            println!("Editing {}", path.display());
            let content = fs::read_to_string(&path)?;
            let edited = edit::edit(&content)?;
            if content != edited {
                fs::write(&path, &edited)?;
                println!("Saved {}", path.display());
            } else {
                println!("No changes");
            }
        },
        None => {
            let paths : Vec<String>= promptfile_locator::search_paths(Some(&promptname))
                .iter().map(|path| path.display().to_string()).collect();

            bail!("Could not create an existing prompt file, searched:\n{}\nConsider creating a new one?", paths.join("\n"))
        }
    };
    Ok(())
}
