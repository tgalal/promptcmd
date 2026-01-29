use clap::{Parser};
use anyhow::{bail, Result};
use crate::config::appconfig::AppConfig;
use crate::config::resolver::Resolver;


#[derive(Parser)]
pub struct ResolveCmd {
    #[arg()]
    pub name: Option<String>
}

impl ResolveCmd {
    pub fn exec(&self, appconfig: &AppConfig, out: &mut impl std::io::Write) -> Result<()> {

        let resolver = Resolver {
            overrides: None,
            fm_properties: None
        };

        match resolver.resolve(appconfig, self.name.clone()){
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
