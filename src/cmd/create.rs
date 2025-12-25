use clap::{Parser};
use anyhow::{bail, Result};
use std::io::{self, Write};

use crate::cmd::{self, templates, TextEditor};
use crate::config::appconfig::AppConfig;
use crate::config::providers::{ProviderVariant};
use crate::dotprompt::ParseDotPromptError;
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
    editor: &impl TextEditor, appconfig: &AppConfig, promptname: &str,
    enable_prompt: bool, force_write: bool) -> Result<()> {

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
                    cmd::enable::exec(storage, promptname)?;
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
    use crate::{cmd::{self, NoOpTextEditor, TextEditor}, config::appconfig::AppConfig, storage::{promptfiles_mem::InMemoryPromptFilesStorage, PromptFilesStorage}};

    const PROMPTFILE_1: &str = r#"
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

    // scenarios:
    // - Write success (validated)
    // - Forced Write
    // - Re-edit
    // - Abort
    
    struct TestingTextEditor {
        user_input: String
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

    #[test]
    fn test_basic_promptfile () {
        let mut storage = InMemoryPromptFilesStorage::new();
        let mut output: Vec<u8> = Vec::new();
        let input = b"";            
        let appconfig = AppConfig::default();
        let editor: TestingTextEditor = TestingTextEditor {
            user_input: PROMPTFILE_1.to_string()
        };

        let promptname = "translate";

        cmd::create::exec(
            &mut &input[..],
            &mut std::io::stderr(), &mut storage, &editor, &appconfig, promptname, false, false).unwrap();

        let actual_promptdata = storage.load(promptname).unwrap().1;

        assert_eq!(
            PROMPTFILE_1, 
            actual_promptdata
        );
    }
}
