use clap::{Parser};
use std::fs;
use log::{info, warn, error, debug};
use anyhow::{bail, Context, Result};
use edit;
use std::fs::File;
use std::io::Write;

use crate::config::ConfigLocator;
// use aibox::config::ConfigLocator;

#[derive(Parser)]
pub struct EditCmd {
    #[arg()]
    promptname: String,
}

pub fn exec(cmd: EditCmd) -> Result<()> {
    let promptname = cmd.promptname;
    let config_filename: String =  format!("{promptname}.prompt");
    let locator: ConfigLocator = ConfigLocator::new("aibox", "prompts.d", config_filename);

    debug!("Searching for config in:");
    for path in locator.get_search_paths() {
        debug!("  - {}", path.display());
    }

    match locator.find_config() {
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
            let paths : Vec<String>= locator
                .get_search_paths().iter().map(|path| path.display().to_string()).collect();

            bail!("Could not create an existing prompt file, searched:\n{}\nConsider creating a new one?", paths.join("\n"))
        }
    };
    Ok(())
}
