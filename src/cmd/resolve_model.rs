use clap::{Parser};
use anyhow::{bail, Result};
use crate::{config::appconfig::AppConfig, resolver::{self, ResolvedPropertySource}};


#[derive(Parser)]
pub struct ResolveModelCmd {
    #[arg()]
    pub name: String,
    #[arg(short, long)]
    pub debug: bool,
}

impl ResolveModelCmd {
    pub fn exec(&self, appconfig: &AppConfig, out: &mut impl std::io::Write) -> Result<()> {
        let source = Some(ResolvedPropertySource::Input(self.name.clone()));
        match resolver::resolve(appconfig, &self.name, source) {
            Ok(resolved_config) => {
                if self.debug {
                    writeln!(out, "{}", resolved_config)?;
                } else {
                    writeln!(out, "{}", resolved_config)?;
                }
            },
            Err(err) => {
                bail!(err)
            }
        }
        Ok(())
    }
}
