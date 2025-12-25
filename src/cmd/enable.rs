use clap::{Parser};
use log::{debug};
use anyhow::{bail, Result};

use crate::{installer::DotPromptInstaller, storage::PromptFilesStorage};

#[derive(Parser)]
pub struct EnableCmd {
    #[arg(long, default_value_t=false)]
    pub here: bool,
    #[arg()]
    pub promptname: String,
}

pub fn exec(
    storage: &impl PromptFilesStorage,
    installer: &mut impl DotPromptInstaller,
    promptname: &str) -> Result<()> {

    if let Some(path) = installer.is_installed(promptname) {
        println!("{} is already installed at {}", promptname, &path);
        return Ok(());
    }

    if let Some(path) = storage.exists(promptname) {
        debug!("Enabling {path}");

        let installed_path = installer.install(promptname)?;
        println!("Created {installed_path}");

    } else {
        bail!("Could not find an existing prompt file");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{cmd, installer::{tests::InMemoryInstaller, DotPromptInstaller}, storage::{promptfiles_mem::InMemoryPromptFilesStorage, PromptFilesStorage}};

    struct TestState {
        storage: InMemoryPromptFilesStorage,
        installer: InMemoryInstaller
    }

    impl Default for TestState {
        fn default() -> Self {
            Self {
                storage: InMemoryPromptFilesStorage::new(),
                installer: InMemoryInstaller::default()
            }
        }
    }

    fn setup() -> TestState {
        TestState::default()
    }

    #[test]
    fn test_enable_success() {
        let mut state = setup();

        state.storage.store("myprompt", "promptdata").unwrap();

        assert!(cmd::enable::exec(&state.storage, &mut state.installer, "myprompt").is_ok());
        assert!(state.installer.is_installed("myprompt").is_some());
    }

    #[test]
    fn test_enable_exists() {
        let mut state = setup();

        state.installer.install("myprompt").unwrap();

        assert!(cmd::enable::exec(&state.storage, &mut state.installer, "myprompt").is_ok());
    }
}
