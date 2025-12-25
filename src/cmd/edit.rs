use clap::{Parser};
use anyhow::{bail, Result};
use edit;

use crate::cmd::create::{validate_and_write, WriteResult};
use crate::config::appconfig::AppConfig;
use crate::storage::PromptFilesStorage;

#[derive(Parser)]
pub struct EditCmd {
    #[arg(short, long, default_value_t=false)]
    pub force: bool,

    #[arg()]
    promptname: String,
}

pub fn exec(
    inp: &mut impl std::io::BufRead, out: &mut impl std::io::Write,
    storage: &mut impl PromptFilesStorage, appconfig: &AppConfig, cmd: EditCmd) -> Result<()> {

    let promptname = cmd.promptname;

    if storage.exists(&promptname).is_some() {
        let (path, content) = storage.load(&promptname)?;
        let mut edited = content.clone();
        println!("Editing {path}");
        loop {
            edited = edit::edit(&edited)?;
            if content != edited {
                match validate_and_write(
                    inp,
                    storage, appconfig, &promptname,
                    edited.as_str(), cmd.force)? {

                    WriteResult::Validated(_, path) | WriteResult::Written(path)=> {
                        writeln!(out, "Saved {path}")?;
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
    } else {
        bail!("Could not find prompt file");
    }

    Ok(())
}
