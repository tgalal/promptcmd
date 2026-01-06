use std::{io::Write, path::Path};
use std::io;
use std::fs;

use clap::{Parser};
use anyhow::{Context, Result};

use crate::cmd::{templates, TextEditor, TextEditorFileType};
use crate::config::appconfig::AppConfig;
use crate::config::appconfig_locator;


#[derive(Parser)]
pub struct ConfigCmd {
    #[arg(short, long, help="Edit your config.toml")]
    pub edit: bool,
    #[arg(short, long, help="Force save the config file without even with validation errors")]
    pub force: bool,
    #[arg(short, long, alias="ls", help="List config.toml lookup paths")]
    pub list: bool,
}

enum WriteResult {
    Validated,
    Written,
    Aborted,
    Edit
}

impl ConfigCmd {
    pub fn exec(&self,
        inp: &mut impl std::io::BufRead,
        out: &mut impl Write,
        editor: &impl TextEditor
    )-> Result<()> {

        if self.list {
            self.exec_ls(out)
        } else if self.edit {
            self.exec_edit(inp, out, editor)
        } else {
            Ok(())
        }
    }

    fn exec_ls(&self, out: &mut impl Write) -> Result<()> {
        let paths : Vec<String>= appconfig_locator::search_paths()
            .iter().map(|path| path.display().to_string()).collect();

        writeln!(out, "{}", paths.join("\n"))?;

        Ok(())
    }

    fn exec_edit(&self,
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

fn validate_and_write(
    inp: &mut impl std::io::BufRead,
    path: &Path,
    contents: &str,
    force_write: bool) -> Result<WriteResult> {

    let validation_result = match AppConfig::try_from(contents) {
        Ok(appconfig) => {
            Ok(appconfig)
        },
        Err(err) => {
            Err(err.to_string())
        }
    };

    match validation_result {
        Ok(_) => {
            fs::write(path, contents)?;
            Ok(WriteResult::Validated)
        }
        Err(err) => {
            println!("{}", err);
            let mut retries = 0;

            if force_write {
                fs::write(path, contents)?;
                return Ok(WriteResult::Written);
            }

            loop {
                print!("Save anyway? [Y]es/[N]o/[E]dit: ");
                io::stdout().flush()?;
                let mut input = String::new();
                inp.read_line(&mut input).unwrap();
                match input.trim().chars().next() {
                    Some('Y' | 'y') => {
                        fs::write(path, contents)?;
                        return Ok(WriteResult::Written);
                    },
                    Some('N' | 'n') => {
                        return Ok(WriteResult::Aborted);
                    },
                    Some('E' | 'e') => {
                        return Ok(WriteResult::Edit);
                    },
                    _ => {
                        println!("Invalid input");
                        retries += 1;
                        if retries > 5 {
                            return Ok(WriteResult::Aborted);
                        }
                    }
                }
            }
        }
    }
}
