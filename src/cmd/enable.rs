use clap::{Parser};
use log::{debug};
use anyhow::{bail, Result};
use log::error;

use crate::{installer::DotPromptInstaller, storage::PromptFilesStorage};

#[derive(Parser)]
pub struct EnableCmd {
    #[arg()]
    pub promptname: String,
}

impl EnableCmd {
    pub fn exec(
        &self,
        storage: &impl PromptFilesStorage,
        installer: &mut impl DotPromptInstaller,
        ) -> Result<()> {

        if let Some(path) = installer.is_installed(&self.promptname) {
            error!("Install path {} already exists", path);
            return Ok(());
        }

        if let Some(path) = storage.exists(&self.promptname) {
            debug!("Enabling {path}");

            let installed_path = installer.install(&self.promptname)?;
            println!("Installed {installed_path}");

        } else {
            bail!("Could not find an existing prompt file");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{cmd::{enable::EnableCmd}, installer::{tests::InMemoryInstaller, DotPromptInstaller}, storage::{promptfiles_mem::InMemoryPromptFilesStorage, PromptFilesStorage}};

    #[derive(Default)]
    struct TestState {
        storage: InMemoryPromptFilesStorage,
        installer: InMemoryInstaller
    }

    fn setup() -> TestState {
        TestState::default()
    }

    #[test]
    fn test_enable_success() {
        let mut state = setup();

        state.storage.store("myprompt", "promptdata").unwrap();

        let cmd = EnableCmd {
            promptname: String::from("myprompt")
        };

        assert!(cmd.exec(&state.storage, &mut state.installer).is_ok());
        assert!(state.installer.is_installed("myprompt").is_some());
    }

    #[test]
    fn test_enable_exists() {
        let mut state = setup();

        let cmd = EnableCmd {
            promptname: String::from("myprompt")
        };

        state.installer.install("myprompt").unwrap();

        assert!(cmd.exec(&state.storage, &mut state.installer).is_ok());
    }
}
