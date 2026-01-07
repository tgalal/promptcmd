use std::{io::Write};
use crate::config::appconfig_locator;

use clap::{Parser};
use anyhow::{Result};


#[derive(Parser)]
pub struct ListSubCmd {}

impl ListSubCmd {
    pub fn exec(&self, out: &mut impl Write) -> Result<()> {
        let paths : Vec<String>= appconfig_locator::search_paths()
            .iter().map(|path| path.display().to_string()).collect();

        writeln!(out, "{}", paths.join("\n"))?;

        Ok(())
    }
}

