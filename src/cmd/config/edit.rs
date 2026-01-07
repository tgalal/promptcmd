use clap::{Parser};
use super::{WriteResult, validate_and_write};
use crate::config::appconfig_locator;
use crate::cmd::{templates, TextEditor, TextEditorFileType};
use std::{io::Write, path::Path};
use std::fs;

use anyhow::{Result, Context};

#[derive(Parser)]
pub struct EditSubCmd {
    #[arg(short, long, help="Force save the config file without even with validation errors")]
    pub force: bool,
}


impl EditSubCmd {
    pub fn exec(&self,
        inp: &mut impl std::io::BufRead,
        out: &mut impl Write,
        editor: &impl TextEditor) -> Result<()> {

        let paths = appconfig_locator::search_paths();

        let path: &Path = paths.iter().find(|path| {
            path.exists()
        }).or(paths.first())
            .context("Could not determine a config path")?;

        let content = if path.exists() {
            println!("Editing {}", path.to_string_lossy());
            fs::read_to_string(path)?
        } else {
            println!("Creating {}", path.to_string_lossy());
            String::from(templates::CONFIG_TEMPLATE)
        };

        let mut edited = content.clone();

        loop {
            edited = editor.edit(&edited, TextEditorFileType::Toml)?;
            if content != edited {
                match validate_and_write(
                    inp,
                    path,
                    edited.as_str(), self.force)? {

                    WriteResult::Validated | WriteResult::Written => {
                        writeln!(out, "Saved {}", path.to_string_lossy())?;
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

        Ok(())
    }
}
