use std::{path::PathBuf, str::FromStr};
use crate::{cmd::enable::EnableCmd, config::appconfig::AppConfig};
use crate::installer::DotPromptInstaller;
use crate::storage::PromptFilesStorage;
use crate::dotprompt::DotPrompt;

use clap::{Parser};
use anyhow::{bail, Context, Result};
use clap_stdin::FileOrStdin;
use log::{debug, info};


#[derive(Parser)]
pub struct ImportCmd {
    #[arg(short, long, help="Prompt name. Required if PROMPTFILE is stdin or a filename not ending with .prompt")]
    pub promptname: Option<String>,

    #[arg(short, long, help="Enable the prompt right after importing")]
    pub enable: bool,

    #[arg(short, long, help="Force overwrite if a promptfile with same name already exists")]
    pub force: bool,

    #[arg(help="Filepath or stdin")]
    pub promptfile: FileOrStdin,
}

impl ImportCmd {
    /**
    * If promptname given, will always use
    * If promptfile given but not promptname, will use filename from promptfile (if .prompt)
    * If stdin, will require promptname
    */
    pub fn exec(
        self,
        storage: &mut impl PromptFilesStorage,
        installer: &mut impl DotPromptInstaller,
        appconfig: &AppConfig,
        ) -> Result<()> {

        let enable = if appconfig.import.enable {
            info!("auto enable is set by config");
            true
        } else {
            self.enable
        };

        let force = if appconfig.import.force {
            info!("force is set by config");
            true
        } else {
            self.force
        };

        let filename = self.promptfile.filename();

        debug!("Filename: {filename}");

        let promptname = if let Some(ref promptname) = self.promptname {
            promptname.to_string()
        } else if filename.ends_with(".prompt") {
            debug!("Determining prompt name from the given file path");
            PathBuf::from_str(
                filename
            ).context("Error creating path")?
            .file_stem()
            .context("Error determining promptname from file path")?
            .to_string_lossy().to_string()
        } else {
            bail!("Could not determine prompt name. Either specify promptname or import from a .prompt file to determine name");
        };

        debug!("Prompt name: {promptname}");

        let contents = self.promptfile.contents()?;

        if let Some(path) = storage.exists(&promptname) {
            if force {
                println!("Overwriting existing file at {path}");
            } else {
                bail!("{path} already exists, use -f/--force to overwrite");
            }
        }

        // Ensure file is actually a dotprompt and we're not importing an arbitrary file
        // DotPrompt::try_from(fs::read_to_string(&fullpath)?)?;
        DotPrompt::try_from(contents.as_str())?;

        let path = storage.store(&promptname, &contents)?;

        debug!("Imported {promptname} to {path}");

        if enable {
            debug!("Enabling {promptname}");
            EnableCmd {
                promptname
            }.exec(storage, installer)?;
        } else {
            debug!("Not enabling {promptname}");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::config::appconfig::AppConfig;
    use crate::{cmd::import::ImportCmd, installer::tests::InMemoryInstaller, storage::promptfiles_mem::InMemoryPromptFilesStorage};
    use crate::storage::PromptFilesStorage;
    use crate::installer::DotPromptInstaller;
    use clap_stdin::FileOrStdin;
    use tempfile::NamedTempFile;

    const PROMPTFILE_BASIC_VALID_1: &str = r#"---
model: ollama/gpt-oss:20b
input:
  schema:
    message: string, Message
output:
  format: text
---
Basic Prompt Here: {{message}}
"#;

    struct TestState {
        storage: InMemoryPromptFilesStorage,
        installer: InMemoryInstaller,
        config: AppConfig,
        promptfile: NamedTempFile
    }

    fn setup(content: &str, use_correct_ext: bool) -> TestState {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = if use_correct_ext {
            NamedTempFile::with_suffix(".prompt").unwrap()
        } else {
            NamedTempFile::new().unwrap()
        };

        write!(temp, "{content}").unwrap();

        TestState {
            storage: InMemoryPromptFilesStorage::default(),
            installer: InMemoryInstaller::default(),
            config: AppConfig::default(),
            promptfile: temp

        }
    }

    #[test]
    fn test_import_by_path() {
        let mut state = setup(PROMPTFILE_BASIC_VALID_1, false);

        let promptname = "myprompt";
        let cmd = ImportCmd {
            promptname: Some(promptname.to_string()),
            enable: true,
            force: false,
            promptfile: FileOrStdin::from_str(state.promptfile.path().to_str().unwrap()).unwrap()
        };

        cmd.exec(&mut state.storage, &mut state.installer, &state.config).unwrap();

        let imported_promptdata = state.storage.load(promptname).unwrap().1;

        // Provided prompt data should be stored as is
        assert_eq!(
            PROMPTFILE_BASIC_VALID_1.trim(),
            imported_promptdata
        );

        // And should be enabled
        assert!(state.installer.is_installed(promptname).is_some());
    }

    #[test]
    fn test_import_by_path_without_enabling() {
        let mut state = setup(PROMPTFILE_BASIC_VALID_1, false);

        let promptname = "myprompt";
        let cmd = ImportCmd {
            promptname: Some(promptname.to_string()),
            enable: false,
            force: false,
            promptfile: FileOrStdin::from_str(state.promptfile.path().to_str().unwrap()).unwrap()
        };

        cmd.exec(&mut state.storage, &mut state.installer, &state.config).unwrap();

        let imported_promptdata = state.storage.load(promptname).unwrap().1;

        // Provided prompt data should be stored as is
        assert_eq!(
            PROMPTFILE_BASIC_VALID_1.trim(),
            imported_promptdata
        );

        // And should not be enabled
        assert!(state.installer.is_installed(promptname).is_none());
    }

    #[test]
    fn test_import_by_path_without_name_but_correct_extension() {
        let mut state = setup(PROMPTFILE_BASIC_VALID_1, true);
        let promptname = state.promptfile.path().file_stem().unwrap().to_str().unwrap();

        let cmd = ImportCmd {
            promptname: None,
            enable: true,
            force: false,
            promptfile: FileOrStdin::from_str(state.promptfile.path().to_str().unwrap()).unwrap()
        };

        cmd.exec(&mut state.storage, &mut state.installer, &state.config).unwrap();

        let imported_promptdata = state.storage.load(promptname).unwrap().1;

        // Provided prompt data should be stored as is
        assert_eq!(
            PROMPTFILE_BASIC_VALID_1.trim(),
            imported_promptdata
        );

        // And should be enabled
        assert!(state.installer.is_installed(promptname).is_some());
    }

    #[test]
    fn test_import_by_path_without_name() {
        let mut state = setup(PROMPTFILE_BASIC_VALID_1, false);

        let promptname = "myprompt";
        let cmd = ImportCmd {
            promptname: None,
            enable: true,
            force: false,
            promptfile: FileOrStdin::from_str(state.promptfile.path().to_str().unwrap()).unwrap()
        };

        // should fail because no name is given, and file path is not .prompt
        cmd.exec(&mut state.storage, &mut state.installer, &state.config).unwrap_err();

        state.storage.load(promptname).unwrap_err();

        // And should be not enabled
        assert!(state.installer.is_installed(promptname).is_none());
    }

    #[test]
    fn test_import_invalid_should_fail() {
        let mut state = setup("---invalid data", false);

        let promptname = "myprompt";
        let cmd = ImportCmd {
            promptname: Some(promptname.to_string()),
            enable: true,
            force: false,
            promptfile: FileOrStdin::from_str(state.promptfile.path().to_str().unwrap()).unwrap()
        };

        cmd.exec(&mut state.storage, &mut state.installer, &state.config).unwrap_err();

        state.storage.load(promptname).unwrap_err();

        // And should be not enabled
        assert!(state.installer.is_installed(promptname).is_none());
    }

    #[test]
    fn test_import_invalid_should_fail_even_with_force() {
        let mut state = setup("---invalid data", false);

        let promptname = "myprompt";
        let cmd = ImportCmd {
            promptname: Some(promptname.to_string()),
            enable: true,
            force: true,
            promptfile: FileOrStdin::from_str(state.promptfile.path().to_str().unwrap()).unwrap()
        };

        cmd.exec(&mut state.storage, &mut state.installer, &state.config).unwrap_err();

        state.storage.load(promptname).unwrap_err();

        // And should be not enabled
        assert!(state.installer.is_installed(promptname).is_none());
    }

    #[test]
    fn test_import_existing_should_fail() {
        let mut state = setup(PROMPTFILE_BASIC_VALID_1, false);
        state.storage.store("myprompt", "promptdata").unwrap();

        let promptname = "myprompt";
        let cmd = ImportCmd {
            promptname: Some(promptname.to_string()),
            enable: true,
            force: false,
            promptfile: FileOrStdin::from_str(state.promptfile.path().to_str().unwrap()).unwrap()
        };

        cmd.exec(&mut state.storage, &mut state.installer, &state.config).unwrap_err();

        // Existing prompt should not have been changed
        let actual_promptdata = state.storage.load(promptname).unwrap().1;
        assert_eq!(
            "promptdata",
            actual_promptdata
        );

        // And should not install
        assert!(state.installer.is_installed(promptname).is_none());
    }

    #[test]
    fn test_import_existing_should_ok_if_force() {
        let mut state = setup(PROMPTFILE_BASIC_VALID_1, false);
        state.storage.store("myprompt", "promptdata").unwrap();

        let promptname = "myprompt";
        let cmd = ImportCmd {
            promptname: Some(promptname.to_string()),
            enable: true,
            force: true,
            promptfile: FileOrStdin::from_str(state.promptfile.path().to_str().unwrap()).unwrap()
        };

        cmd.exec(&mut state.storage, &mut state.installer, &state.config).ok();

        // Existing prompt should not have been changed
        let actual_promptdata = state.storage.load(promptname).unwrap().1;
        assert_eq!(
            PROMPTFILE_BASIC_VALID_1.trim(),
            actual_promptdata
        );

        // And should be enabled
        assert!(state.installer.is_installed(promptname).is_some());
    }
}
