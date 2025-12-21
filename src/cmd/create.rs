use clap::{Parser};
use std::fs;
use anyhow::{bail, Context, Result};
use edit;
use std::path::PathBuf;
use std::io::{self, Write};

use crate::{cmd::enable, config::promptfile_locator, dotprompt::DotPrompt};

const TEMPL: &str = r#"---
model: MODEL
input:
  schema:
output:
  format: text
---
You are a useful assistant

{{STDIN}}
"#;

#[derive(Parser)]
pub struct CreateCmd {
    #[arg(short, long, default_value_t=true)]
    pub now: bool,

    #[arg()]
    pub promptname: String,
}

pub enum WriteResult {
    Validated,
    Written,
    Aborted,
    Edit
}

pub fn validate_and_write(promptdata: &str, path: &PathBuf) -> Result<WriteResult>{
    match DotPrompt::try_from(promptdata) {
        Ok(_) => {
            fs::write(path, promptdata)?;
            Ok(WriteResult::Validated)
        }
        Err(err) => {
            println!("{}", err);
            loop {
                print!("Save anyway? [Y]es/[N]o/[E]dit: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                match input.trim().chars().next() {
                    Some('Y' | 'y') => {
                        fs::write(path, promptdata)?;
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
                    }
                }
            }
        }
    }
}

pub fn exec(promptname: &str, enable_prompt: bool) -> Result<()> {

    match promptfile_locator::find(promptname) {
        Some(path) => {
            bail!("Prompt file already exists: {}", path.display());
        },
        None => {
            let path = promptfile_locator::path(promptname).context(
                "Could not locate promptfile"
            )?;

            let mut edited = TEMPL.to_string();
            loop {
                edited = edit::edit(edited)?;
                if TEMPL != edited {
                    match validate_and_write(edited.as_str(), &path)? {
                        WriteResult::Validated => {
                            println!("Saved {}", path.display());
                            if enable_prompt {
                                return enable::exec(promptname);
                            }
                            break;
                        }

                        WriteResult::Written => {
                            println!("Saved {}", path.display());
                            if enable_prompt {
                                println!("Not enabling due to errors");
                            }
                            break;
                        }

                        WriteResult::Aborted => {
                            println!("No changes, did not save");
                            break;
                        }

                        WriteResult::Edit => {}
                    }
                } else {
                    println!("No changes, did not save");
                    break;
                }
            }
        }
    };
    Ok(())
}
