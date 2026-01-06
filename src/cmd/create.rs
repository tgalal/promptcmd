use clap::{Parser};
use anyhow::{bail, Result};
use llm::builder::LLMBuilder;
use std::io::{self, Write};
use log::{error, info};

use crate::cmd::enable::EnableCmd;
use crate::cmd::{templates, TextEditor, TextEditorFileType};
use crate::config::appconfig::{AppConfig};
use crate::config::appconfig_locator;
use crate::installer::DotPromptInstaller;
use crate::storage::PromptFilesStorage;
use crate::{dotprompt::DotPrompt};
use crate::config::resolver::{self, ResolvedPropertySource};
use crate::config::providers::{ModelInfo, error};


#[derive(Parser)]
pub struct CreateCmd {
    #[arg(short, long, default_value_t=false, help="Auto enable the prompt file after creation")]
    pub enable: bool,

    #[arg(short, long, default_value_t=false, help="Force save the prompt file without even with validation errors")]
    pub force: bool,

    #[arg(help="What to call the prompt being created.")]
    pub promptname: String,
}

pub enum WriteResult {
    Validated(DotPrompt, String),
    Written(String),
    Aborted,
    Edit
}

impl CreateCmd {
    pub fn exec(
        &self,
        inp: &mut impl std::io::BufRead,
        out: &mut impl std::io::Write,
        storage: &mut impl PromptFilesStorage,
        installer: &mut impl DotPromptInstaller,
        editor: &impl TextEditor,
        appconfig: &AppConfig,
        ) -> Result<()> {

        if let Some(path) = storage.exists(&self.promptname) {
            bail!("Prompt file already exists: {path}");
        }

        let enable = if appconfig.create.enable {
            info!("auto enable is set by config");
            true
        } else {
            self.enable
        };

        let force = if appconfig.create.force {
            info!("force is set by config");
            true
        } else {
            self.force
        };

        let mut edited = templates::PROMPTFILE.to_string();
        loop {
            edited = editor.edit(&edited, TextEditorFileType::Dotprompt)?;

            match validate_and_write(inp, storage, &self.promptname, edited.as_str(), force)? {
                // Validated means written without errors.
                // In this case:
                // - we also enable it (if `enable`` is true)
                // - we give out user help if provider has no available configuration
                WriteResult::Validated(dotprompt, path) => {
                    writeln!(out, "Saved {}", path)?;
                    if enable {
                        EnableCmd {
                            promptname: self.promptname.clone()
                        }.exec(storage, installer)?;
                    }

                    let model_name = dotprompt.frontmatter.model.or(
                        appconfig.providers.default.clone().or(
                            appconfig.providers.globals.model.clone()
                        )
                    );

                    if let Some(model_name) = model_name {

                        let resolved_config = match resolver::resolve(
                            appconfig, &model_name, Some(ResolvedPropertySource::Dotprompt(model_name.clone())
                        )) {
                            Ok(resolver::ResolvedConfig::Base(base))  => {
                                <(ModelInfo, LLMBuilder)>::try_from(&base)
                            },
                            Ok(resolver::ResolvedConfig::Variant(variant))  => {
                                <(ModelInfo, LLMBuilder)>::try_from(&variant)
                            },
                            Ok(_) => {
                                break;
                            },
                            Err(resolver::error::ResolveError::NotFound(name)) => {
                                writeln!(out, "Warning: No configuration could be found for '{name}'")?;
                                break;
                            },
                            Err(err) => {
                                error!("{}", err);
                                break;
                            },
                        };

                        match resolved_config {
                            Ok(_) => {},
                            Err(error::ToLLMBuilderError::RequiredConfiguration(_name)) | Err(error::ToLLMBuilderError::ModelError(error::ToModelInfoError::RequiredConfiguration(_name))) => {
                                let config_paths = appconfig_locator::search_paths()
                                    .iter().map(|path| path.to_string_lossy())
                                    .collect::<Vec<_>>()
                                    .join("\n");

                                if model_name.starts_with("anthropic") {
                                    writeln!(out, "{}{}", templates::ONBOARDING_ANTHROPIC, config_paths)?
                                } else if model_name.starts_with("openai") {
                                    writeln!(out, "{}{}", templates::ONBOARDING_OPENAI, config_paths)?
                                } else if model_name.starts_with("google") {
                                    writeln!(out, "{}{}", templates::ONBOARDING_GOOGLE, config_paths)?
                                } else if model_name.starts_with("openrouter") {
                                    writeln!(out, "{}{}", templates::ONBOARDING_OPENROUTER, config_paths)?
                                } else if model_name .starts_with("ollama") {
                                    writeln!(out, "{}{}", templates::ONBOARDING_OLLAMA, config_paths)?
                                }
                            }
                        }
                    }

                    break;
                }
                // Written means there were errors, but the user forced writing the file.
                // In this case we don't enable the prompt automatically, even if requested.
                WriteResult::Written(path) => {
                    writeln!(out, "Saved {}", path)?;
                    if enable {
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
}

pub fn validate_and_write(
    inp: &mut impl std::io::BufRead,
    storage: &mut impl PromptFilesStorage,
    promptname: &str, promptdata: &str, force_write: bool) -> Result<WriteResult> {

    let validation_result = match DotPrompt::try_from(promptdata) {
        Ok(dotprompt) => {
            Ok(dotprompt)
        },
        Err(err) => {
            Err(err.to_string())
        }
    };

    match validation_result {
        Ok(dotprompt) => {
            let path = storage.store(promptname, promptdata)?;
            Ok(WriteResult::Validated(dotprompt, path))
        }
        Err(err) => {
            println!("{}", err);
            let mut retries = 0;

            if force_write {
                let path = storage.store(promptname, promptdata)?;
                return Ok(WriteResult::Written(path));
            }

            loop {
                print!("Save anyway? [Y]es/[N]o/[E]dit: ");
                io::stdout().flush()?;
                let mut input = String::new();
                inp.read_line(&mut input).unwrap();
                match input.trim().chars().next() {
                    Some('Y' | 'y') => {
                        let path = storage.store(promptname, promptdata)?;
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


#[cfg(test)]
mod tests {
    use crate::{cmd::{self, create::CreateCmd, TextEditor, TextEditorFileType}, config::appconfig::AppConfig, installer::{tests::InMemoryInstaller, DotPromptInstaller}, storage::{promptfiles_mem::InMemoryPromptFilesStorage, PromptFilesStorage}};

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

    #[derive(Default)]
    struct TestingTextEditor {
        user_input: String
    }

    impl TestingTextEditor {
        pub fn set_user_input(&mut self, data: &str) {
            self.user_input = data.to_string().clone();
        }
    }

    impl TextEditor for TestingTextEditor {
        fn edit(&self, _: &str, _: TextEditorFileType) -> Result<String, cmd::TextEditorError> {
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

        CreateCmd {
            promptname: String::from(promptname),
            enable: true,
            force: false
        }.exec(
                &mut &state.inp[..],
                &mut std::io::stderr(),
                &mut state.storage,
                &mut state.installer,
                &state.editor,
                &state.config,
            ).unwrap();

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

        CreateCmd {
            promptname: String::from(promptname),
            enable: false,
            force: false
        }.exec(
            &mut &state.inp[..],
            &mut std::io::stderr(),
            &mut state.storage,
            &mut state.installer,
            &state.editor,
            &state.config,
        ).unwrap();

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
    fn test_invalid_provider_force_save_by_input() {
        let mut state = setup(b"Y\n");
        state.editor.set_user_input(PROMPTFILE_INVALID_MODEL);

        let promptname = "translate";

        CreateCmd {
            promptname: String::from(promptname),
            enable: false,
            force: false
        }.exec(
            &mut &state.inp[..],
            &mut state.out,
            &mut state.storage,
            &mut state.installer,
            &state.editor,
            &state.config,
        ).unwrap();

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

        CreateCmd {
            promptname: String::from(promptname),
            enable: false,
            force: true
        }.exec(
            &mut &state.inp[..],
            &mut state.out,
            &mut state.storage,
            &mut state.installer,
            &state.editor,
            &state.config,
        ).unwrap();


        let actual_promptdata = state.storage.load(promptname).unwrap().1;

        assert_eq!(
            PROMPTFILE_INVALID_MODEL,
            actual_promptdata
        );

        // And should not be enabled
        assert!(state.installer.is_installed(promptname).is_none());
    }
}
