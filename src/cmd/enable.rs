use std::path::{Path, PathBuf};

use clap::{Parser};
use log::{debug};
use anyhow::{bail, Result};
use log::error;
use std::env;

use crate::{installer::DotPromptInstaller, storage::PromptFilesStorage};

#[derive(Parser)]
pub struct EnableCmd {
    #[arg()]
    pub promptname: String,
}

fn is_in_path(dir: &str) -> bool {
    let dir_path = Path::new(dir);

    if let Some(path_var) = env::var_os("PATH") {
        for path in env::split_paths(&path_var) {
            if path == dir_path {
                return true;
            }
        }
    }
    false
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

            if let Some(path) = PathBuf::from(&installed_path).parent() && !is_in_path(&path.to_string_lossy()) {
                let path = path.to_string_lossy();
                let warning_message = format!(
r#"Warning: The install directory:

{path}

is not in your PATH environment variable. You can temporarily update your PATH for this session:

export PATH=$PATH:"{path}"

which will make `{}` available right away. Alternatively you can run one of:

promptctl run {} --
"{installed_path}"

For simplicity, consider updating your shell's PATH to persistently run prompts without requiring their full path,

"#,
                    &self.promptname, &self.promptname);
                let rendered_wm = textwrap::wrap(&warning_message, 80).join("\n");
                println!();
                println!("{}", rendered_wm);
            }
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
