use clap::{Parser};
use std::fs;
use log::{debug};
use anyhow::{bail, Context, Result};
use edit;

use crate::{cmd::enable, config::promptfile_locator};

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

pub fn exec(promptname: &String, enable_prompt: bool) -> Result<()> {

    match promptfile_locator::find(promptname) {
        Some(path) => {
            bail!("Prompt file already exists: {}", path.display());
        },
        None => {
            let path = promptfile_locator::find(promptname).context(
                "Could not locate promptfile"
            )?;

            let edited = edit::edit(TEMPL)?;

            if TEMPL != edited {
                fs::write(&path, &edited)?;
                println!("Saved {}", path.display());
                if enable_prompt {
                    return enable::exec(promptname);
                }
            } else {
                println!("No changes, did not save");
            }
        }
    };
    Ok(())
}
