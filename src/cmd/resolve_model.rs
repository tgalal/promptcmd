use clap::{Parser};
use anyhow::{bail, Result};
use crate::{config::appconfig::AppConfig, resolver::{self, ResolvedPropertySource}};


#[derive(Parser)]
pub struct ResolveModelCmd {
    #[arg()]
    pub name: String
}

impl ResolveModelCmd {
    pub fn exec(&self, appconfig: &AppConfig, out: &mut impl std::io::Write) -> Result<()> {
        let source = Some(ResolvedPropertySource::Input(self.name.clone()));
        match resolver::resolve(appconfig, &self.name, source) {
            Ok(resolved_config) => {
                writeln!(out, "{}", resolved_config)?;
            },
            Err(err) => {
                bail!(err)
            }
        }
        Ok(())
    }
}
