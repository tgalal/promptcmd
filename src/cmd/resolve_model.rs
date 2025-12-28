use clap::{Parser};
use anyhow::{Result, bail};
use crate::config::appconfig::{AppConfig, ModelError};


#[derive(Parser)]
pub struct ResolveModelCmd {
    #[arg()]
    pub name: String,
}

impl ResolveModelCmd {
    pub fn exec(&self, appconfig: &AppConfig, out: &mut impl std::io::Write) -> Result<()> {

        match appconfig.resolve_model_name(&self.name, true) {
            Err(ModelError::NoDefaultModelConfigured(provider_name)) => {
                writeln!(out, "{} resolves to {}, but no default_model has been configured for it", &self.name, &provider_name)?;
            },
            Err(err) => {
                bail!("{err}");
            },
            Ok(resolved_names) => {
                if resolved_names.is_empty() {
                    bail!("Failed to resolve {}", &self.name);
                } else if resolved_names.len() == 1 {
                    writeln!(out, "{}:{}", &resolved_names[0].provider, &resolved_names[0].model)?;
                } else {
                    writeln!(out, "Group:")?;
                    for item in resolved_names {
                        writeln!(out, "  {}/{}", &item.provider, &item.model)?;
                    }
                }
            }
        }

        Ok(())
    }
}
