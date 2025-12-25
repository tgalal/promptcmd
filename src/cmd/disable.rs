use clap::{Parser};
use anyhow::{Context, Result};

use crate::{installer::DotPromptInstaller};

#[derive(Parser)]
pub struct DisableCmd {
    #[arg()]
    pub promptname: String,
}

impl DisableCmd {
    pub fn exec(&self, installer: &mut impl DotPromptInstaller) -> Result<()> {

        if let Some(path) = installer.is_installed(&self.promptname) {
            installer.uninstall(&self.promptname).context("Failed to uninstalled prompt")?;
            println!("Removed {path}");
        } 

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{cmd::{disable::DisableCmd}, installer::{tests::InMemoryInstaller, DotPromptInstaller, UninstallError}, storage::promptfiles_mem::InMemoryPromptFilesStorage};

    #[derive(Default)]
    struct TestState {
        storage: InMemoryPromptFilesStorage,
        installer: InMemoryInstaller
    }

    fn setup() -> TestState {
        TestState::default()
    }

    #[test]
    fn test_disable_success() {
        let mut state = setup();

        state.installer.install("myprompt").unwrap();

        let cmd = DisableCmd {
            promptname: String::from("myprompt")
        };

        assert!(cmd.exec(&mut state.installer).is_ok());
        assert!(state.installer.is_installed("myprompt").is_none());
    }

    #[test]
    fn test_already_disable() {
        let mut state = setup();

        let cmd = DisableCmd {
            promptname: String::from("myprompt")
        };

        assert!(cmd.exec(&mut state.installer).is_ok());

        let result = state.installer.uninstall("myprompt").unwrap_err();

        assert!(matches!(result, UninstallError::NotInstalled(_)));
    }
}
