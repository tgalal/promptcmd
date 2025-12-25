use clap::{Parser};
use anyhow::{Context, Result};

use crate::{installer::DotPromptInstaller};

#[derive(Parser)]
pub struct DisableCmd {
    #[arg()]
    pub promptname: String,
}

pub fn exec(
    installer: &mut impl DotPromptInstaller,
    promptname: &str) -> Result<()> {

    if let Some(path) = installer.is_installed(promptname) {
        installer.uninstall(promptname).context("Failed to uninstalled prompt")?;
        println!("Removed {path}");
    } 

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{cmd, installer::{tests::InMemoryInstaller, DotPromptInstaller, UninstallError}, storage::{promptfiles_mem::InMemoryPromptFilesStorage}};

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

        assert!(cmd::disable::exec(&mut state.installer, "myprompt").is_ok());
        assert!(state.installer.is_installed("myprompt").is_none());
    }

    #[test]
    fn test_already_disable() {
        let mut state = setup();

        assert!(cmd::disable::exec(&mut state.installer, "myprompt").is_ok());

        let result = state.installer.uninstall("myprompt").unwrap_err();

        assert!(matches!(result, UninstallError::NotInstalled(_)));
    }
}
