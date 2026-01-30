use std::path::PathBuf;
use std::{io::Write};
use std::io;
use std::fs;
use clap::{Parser, Subcommand};
use anyhow::{Result};

use crate::cmd::TextEditor;
use crate::config::appconfig::AppConfig;

mod edit;
mod list;

use edit::EditSubCmd;
use list::ListSubCmd;

#[derive(Parser)]
pub struct ConfigCmd {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    #[clap(alias="e", about="Edit your config.toml")]
    Edit(EditSubCmd),
    #[clap(alias="ls", about="List config.toml lookup paths")]
    List(ListSubCmd),
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
        editor: &impl TextEditor,
        config_path: Option<PathBuf>
    )-> Result<()> {

        match &self.action {
            Action::Edit(action) => action.exec(inp, out, editor, config_path),
            Action::List(action) => action.exec(out)
        }
    }
}

fn validate_and_write(
    inp: &mut impl std::io::BufRead,
    path: &PathBuf,
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
