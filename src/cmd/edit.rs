use clap::{Parser};
use std::fs;
use anyhow::{bail, Result};
use edit;

use crate::cmd::create::{validate_and_write, WriteResult};
use crate::config::appconfig::AppConfig;
use crate::config::promptfile_locator;

#[derive(Parser)]
pub struct EditCmd {
    #[arg(short, long, default_value_t=false)]
    pub force: bool,

    #[arg()]
    promptname: String,
}

pub fn exec(appconfig: &AppConfig, cmd: EditCmd) -> Result<()> {
    let promptname = cmd.promptname;

    match promptfile_locator::find(&promptname) {
        Some(path) => {
            let content = fs::read_to_string(&path)?;
            let mut edited = content.clone();
            println!("Editing {}", path.display());
            loop {
                edited = edit::edit(&edited)?;
                if content != edited {
                    match validate_and_write(appconfig, edited.as_str(), &path, cmd.force)? {
                        WriteResult::Validated(_)| WriteResult::Written=> {
                            println!("Saved {}", path.display());
                            break;
                        }
                        WriteResult::Aborted => {
                            println!("Editing aborted, no changes were saved");
                            break;
                        }
                        WriteResult::Edit => {}
                    }
                } else {
                    println!("No changes");
                    break;
                }
            }
        },
        None => {
            let paths : Vec<String>= promptfile_locator::search_paths(Some(&promptname))?
                .iter().map(|path| path.display().to_string()).collect();

            bail!("Could not find an existing prompt file, searched:\n{}\nConsider creating a new one?", paths.join("\n"))
        }
    };
    Ok(())
}
