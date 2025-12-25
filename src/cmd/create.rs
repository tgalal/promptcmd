use clap::{Parser};
use anyhow::{bail, Result};
use std::io::{self, Write};

use crate::cmd::{self, templates, TextEditor};
use crate::config::appconfig::AppConfig;
use crate::config::providers::{ProviderVariant};
use crate::dotprompt::ParseDotPromptError;
use crate::installer::DotPromptInstaller;
use crate::storage::PromptFilesStorage;
use crate::{dotprompt::DotPrompt};

#[derive(Parser)]
pub struct CreateCmd {
    #[arg(short, long, default_value_t=true)]
    pub now: bool,

    #[arg(short, long, default_value_t=false)]
    pub force: bool,

    #[arg()]
    pub promptname: String,
}

pub enum WriteResult {
    Validated(DotPrompt, String),
    Written(String),
    Aborted,
    Edit
}

pub fn validate_and_write(
    inp: &mut impl std::io::BufRead,
    storage: &mut impl PromptFilesStorage, appconfig: &AppConfig,
    promptname: &str, promptdata: &str, force_write: bool) -> Result<WriteResult> {

    let validation_result = match DotPrompt::try_from(promptdata) {
        Ok(dotprompt) => {
            let provider = &dotprompt.model_info()?.provider;
            if let ProviderVariant::None = appconfig.providers.resolve(provider) {
                Err(ParseDotPromptError(format!("Provider {} is unsupported", provider)))
            } else {
                Ok(dotprompt)
            }
        },
        Err(err) => {
            Err(err)
        }
    };

    match validation_result {
        Ok(dotprompt) => {
            let path = storage.store(promptname, &promptdata)?;
            Ok(WriteResult::Validated(dotprompt, path))
        }
        Err(err) => {
            println!("{}", err);
            let mut retries = 0;

            if force_write {
                let path = storage.store(promptname, &promptdata)?;
                return Ok(WriteResult::Written(path));
            }

            loop {
                print!("Save anyway? [Y]es/[N]o/[E]dit: ");
                io::stdout().flush()?;
                let mut input = String::new();
                inp.read_line(&mut input).unwrap();
                match input.trim().chars().next() {
                    Some('Y' | 'y') => {
                        let path = storage.store(promptname, &promptdata)?;
                        return Ok(WriteResult::Written(path));
                    },
                    Some('N' | 'n') => {
                        return Ok(WriteResult::Aborted);
                    },
                    Some('E' | 'e') => {
                        return Ok(WriteResult::Edit);
                    },
                    _ => {
                        println!("Invalid input");
                        retries += 1;
                        if retries > 5 {
                            return Ok(WriteResult::Aborted);
                        }
                    }
                }
            }
        }
    }
}

pub fn exec(
    inp: &mut impl std::io::BufRead,
    out: &mut impl std::io::Write,
    storage: &mut impl PromptFilesStorage,
    installer: &mut impl DotPromptInstaller,
    editor: &impl TextEditor,
    appconfig: &AppConfig,
    promptname: &str,
    enable_prompt: bool,
    force_write: bool) -> Result<()> {

    if let Some(path) = storage.exists(promptname) {
        bail!("Prompt file already exists: {path}");
    }

    let mut edited = templates::PROMPTFILE.to_string();
    loop {
        edited = editor.edit(&edited)?;

        match validate_and_write(inp, storage, appconfig, promptname, edited.as_str(), force_write)? {
            // Validated means written without errors.
            // In this case:
            // - we also enable it (if `enable`` is true)
            // - we give out user help if provider has no available configuration
            WriteResult::Validated(dotprompt, path) => {
                writeln!(out, "Saved {}", path)?;
                if enable_prompt {
                    cmd::enable::exec(storage, installer, promptname)?;
                }

                let model_info = dotprompt.model_info()?;
                match appconfig.providers.resolve(&model_info.provider) {
                    ProviderVariant::Anthropic(conf) => {
                        if conf.api_key(&appconfig.providers).is_none() {
                            writeln!(out, "{}", templates::ONBOARDING_ANTHROPIC)?;
                        }
                    },
                    ProviderVariant::OpenAi(conf) => {
                        if conf.api_key(&appconfig.providers).is_none() {
                            writeln!(out, "{}", templates::ONBOARDING_OPENAI)?;
                        }
                    },
                    ProviderVariant::Google(conf) => {
                        if conf.api_key(&appconfig.providers).is_none() {
                            writeln!(out, "{}", templates::ONBOARDING_GOOGLE)?;
                        }
                    },
                    _ => {}
                }
                break;
            }
            // Written means there were errors, but the user forced writing the file.
            // In this case we don't enable the prompt automatically, even if requested.
            WriteResult::Written(path) => {
                writeln!(out, "Saved {}", path)?;
                if enable_prompt {
                    writeln!(out, "Not enabling due to errors")?;
                }
                break;
            }
            // User aborted writing
            WriteResult::Aborted => {
                writeln!(out, "No changes, did not save")?;
                break;
            }
            // User choose to re-edit, thus will not break the loop
            WriteResult::Edit => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{cmd::{self, TextEditor}, config::appconfig::AppConfig, installer::{tests::InMemoryInstaller, DotPromptInstaller}, storage::{promptfiles_mem::InMemoryPromptFilesStorage, PromptFilesStorage}};

    const PROMPTFILE_BASIC_VALID: &str = r#"
---
model: ollama/gpt-oss:20b
input:
  schema:
    message: string, Message
output:
  format: text
---
Basic Prompt Here: {{message}}
"#;

    const PROMPTFILE_INVALID_MODEL: &str = r#"
---
model: aaaa/gpt-oss:20b
input:
  schema:
    message: string, Message
output:
  format: text
---
Basic Prompt Here: {{message}}
"#;

    // scenarios:
    // - Write success (validated)
    // - Forced Write
    // - Re-edit
    // - Abort
    
    struct TestingTextEditor {
        user_input: String
    }

    impl Default for TestingTextEditor {
        fn default() -> Self {
            Self {
                user_input: String::from("")
            }
        }
    }
    impl TestingTextEditor {
        pub fn set_user_input(&mut self, data: &str) {
            self.user_input = data.to_string().clone();
        }
    }

    impl TextEditor for TestingTextEditor {
        fn edit(&self, _: &str) -> Result<String, cmd::TextEditorError> {
            Ok(self.user_input.clone())
        }
    }

    struct TestState {
        storage: InMemoryPromptFilesStorage,
        installer: InMemoryInstaller,
        config: AppConfig,
        inp: Vec<u8>,
        out: Vec<u8>,
        editor: TestingTextEditor
    }

    fn setup(inpdata: &[u8]) -> TestState {
        TestState {
            storage: InMemoryPromptFilesStorage::default(),
            installer: InMemoryInstaller::default(),
            config: AppConfig::default(),
            inp: inpdata.to_vec(),
            out: Vec::new(),
            editor: TestingTextEditor::default()
        }
    }

    #[test]
    fn test_basic_promptfile () {
        let mut state = setup(b"");
        state.editor.set_user_input(PROMPTFILE_BASIC_VALID);

        let promptname = "translate";

        cmd::create::exec(
            &mut &state.inp[..],
            &mut std::io::stderr(),
            &mut state.storage,
            &mut state.installer,
            &state.editor,
            &state.config,
            promptname,
            true,
            false).unwrap();

        let actual_promptdata = state.storage.load(promptname).unwrap().1;

        // Provided prompt data should be stored as is
        assert_eq!(
            PROMPTFILE_BASIC_VALID, 
            actual_promptdata
        );

        // And should be enabled
        assert!(state.installer.is_installed(promptname).is_some());
    }

    #[test]
    fn test_basic_promptfile_without_enabling () {
        let mut state = setup(b"");
        state.editor.set_user_input(PROMPTFILE_BASIC_VALID);

        let promptname = "translate";

        cmd::create::exec(
            &mut &state.inp[..],
            &mut std::io::stderr(),
            &mut state.storage,
            &mut state.installer,
            &state.editor,
            &state.config,
            promptname,
            false,
            false).unwrap();

        let actual_promptdata = state.storage.load(promptname).unwrap().1;

        // Provided prompt data should be stored as is
        assert_eq!(
            PROMPTFILE_BASIC_VALID, 
            actual_promptdata
        );

        // And should not be enabled (enable is false)
        assert!(state.installer.is_installed(promptname).is_none());
    }

    #[test]
    fn test_invalid_provider_nosave() {
        let mut state = setup(b"N\n");
        state.editor.set_user_input(PROMPTFILE_INVALID_MODEL);

        let promptname = "translate";

        cmd::create::exec(
            &mut &state.inp[..],
            &mut state.out,
            &mut state.storage,
            &mut state.installer,
            &state.editor,
            &state.config,
            promptname,
            false,
            false).unwrap();

        assert!(state.storage.load(promptname).is_err());

        // And should not be enabled
        assert!(state.installer.is_installed(promptname).is_none());
    }

    #[test]
    fn test_invalid_provider_force_save_by_input() {
        let mut state = setup(b"Y\n");
        state.editor.set_user_input(PROMPTFILE_INVALID_MODEL);

        let promptname = "translate";

        cmd::create::exec(
            &mut &state.inp[..],
            &mut state.out,
            &mut state.storage,
            &mut state.installer,
            &state.editor,
            &state.config,
            promptname,
            false,
            false).unwrap();

        let actual_promptdata = state.storage.load(promptname).unwrap().1;

        assert_eq!(
            PROMPTFILE_INVALID_MODEL, 
            actual_promptdata
        );

        // And should not be enabled
        assert!(state.installer.is_installed(promptname).is_none());
    }

    #[test]
    fn test_invalid_provider_force_save_by_argument() {
        let mut state = setup(b"");
        state.editor.set_user_input(PROMPTFILE_INVALID_MODEL);

        let promptname = "translate";

        cmd::create::exec(
            &mut &state.inp[..],
            &mut state.out,
            &mut state.storage,
            &mut state.installer,
            &state.editor,
            &state.config,
            promptname,
            false,
            true).unwrap();

        let actual_promptdata = state.storage.load(promptname).unwrap().1;

        assert_eq!(
            PROMPTFILE_INVALID_MODEL, 
            actual_promptdata
        );

        // And should not be enabled
        assert!(state.installer.is_installed(promptname).is_none());
    }
}
