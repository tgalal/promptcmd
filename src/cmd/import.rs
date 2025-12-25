use std::{path::PathBuf, str::FromStr};
use crate::installer::DotPromptInstaller;
use crate::storage::PromptFilesStorage;
use crate::{cmd::enable as enable_cmd};
use crate::dotprompt::DotPrompt;

use clap::{Parser};
use anyhow::{bail, Context, Result};
use clap_stdin::FileOrStdin;
use log::{debug};


#[derive(Parser)]
pub struct ImportCmd {
    #[arg(short, long, help="Prompt name")]
    pub promptname: Option<String>,

    #[arg(short, long, help="Enable the prompt right after importing")]
    pub enable: bool,

    #[arg(short, long, help="Force overwrite if a promptfile with same name already exists")]
    pub force: bool,

    pub promptfile: FileOrStdin,
}

/**
* If promptname given, will always use
* If promptfile given but not promptname, will use filename from promptfile (if .prompt)
* If stdin, will require promptname
*/
pub fn exec(
    storage: &mut impl PromptFilesStorage,
    installer: &mut impl DotPromptInstaller,
    promptname: Option<String>,
    promptfile: FileOrStdin,
    enable: bool,
    force: bool) -> Result<()> {

    let filename = promptfile.filename();

    debug!("Filename: {filename}");

    let promptname = if let Some(promptname) = promptname {
        promptname
    } else if filename.ends_with(".prompt") {
        debug!("Determining prompt name from the given file path");
        PathBuf::from_str(filename).unwrap().file_stem().context("Error")?.to_string_lossy().to_string()
    } else {
        bail!("Could not determine prompt name. Either specify promptname or import from a .prompt file to determine name");
    };

    debug!("Prompt name: {promptname}");

    let contents = promptfile.contents()?;

    if let Some(path) = storage.exists(&promptname) {
        if force {
            println!("Overwriting existing file at {path}");
        } else {
            println!("{path} already exists, use -f/--force to overwrite");
            return Ok(())
        }
    }

    // Ensure file is actually a dotprompt and we're not importing an arbitrary file
    // DotPrompt::try_from(fs::read_to_string(&fullpath)?)?;
    DotPrompt::try_from(contents.as_str())?;

    let path = storage.store(&promptname, &contents)?;

    debug!("Imported {promptname} to {path}");

    if enable {
        debug!("Enabling {promptname}");
        enable_cmd::exec(storage, installer, &promptname)?;
    } else {
        debug!("Not enabling {promptname}");
    }
    Ok(())
}
