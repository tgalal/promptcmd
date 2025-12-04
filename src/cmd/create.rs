use clap::{Parser};
use std::fs;
use log::{debug};
use anyhow::{bail, Result};
use edit;

use crate::config::ConfigLocator;

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
    now: bool,

    #[arg()]
    promptname: String,
}

pub fn exec(cmd: CreateCmd) -> Result<()> {
    let promptname = cmd.promptname;
    let config_filename: String =  format!("{promptname}.prompt");
    let locator: ConfigLocator = ConfigLocator::new("aibox", "prompts.d", config_filename);

    match locator.find_config() {
        Some(path) => {
            bail!("Prompt file already exists: {}", path.display());
        },
        None => {
            let path = locator.get_user_config_path()?;

            let edited = edit::edit(TEMPL)?;

            if TEMPL != edited {
                fs::write(&path, &edited)?;
                println!("Saved {}", path.display());
            } else {
                println!("No changes, did not save");
            }
        }
    };
    Ok(())
}
