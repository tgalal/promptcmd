use clap::{Parser};
use std::fs;
use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::io::{self, Write};

use crate::cmd::{self, templates};
use crate::config::appconfig::AppConfig;
use crate::config::providers::{ProviderVariant};
use crate::dotprompt::ParseDotPromptError;
use crate::{config::promptfile_locator, dotprompt::DotPrompt};

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
    Validated(DotPrompt),
    Written,
    Aborted,
    Edit
}

pub fn validate_and_write(appconfig: &AppConfig, promptdata: &str, path: &PathBuf, force_write: bool) -> Result<WriteResult> {

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
            fs::write(path, promptdata)?;
            Ok(WriteResult::Validated(dotprompt))
        }
        Err(err) => {
            println!("{}", err);

            if force_write {
                fs::write(path, promptdata)?;
                return Ok(WriteResult::Written);
            }

            loop {
                print!("Save anyway? [Y]es/[N]o/[E]dit: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                match input.trim().chars().next() {
                    Some('Y' | 'y') => {
                        fs::write(path, promptdata)?;
                        return Ok(WriteResult::Written);
                    },
                    Some('N' | 'n') => {
                        return Ok(WriteResult::Aborted);
                    },
                    Some('E' | 'e') => {
                        return Ok(WriteResult::Edit);
                    },
                    _ => {
                        println!("Invalid input");
                    }
                }
            }
        }
    }
}

pub fn exec(appconfig: &AppConfig, promptname: &str, enable_prompt: bool, force_write: bool) -> Result<()> {

    match promptfile_locator::find(promptname) {
        Some(path) => {
            bail!("Prompt file already exists: {}", path.display());
        },
        None => {
            let path = promptfile_locator::path(promptname).context(
                "Could not locate promptfile"
            )?;

            let mut edited = templates::PROMPTFILE.to_string();
            loop {
                edited = edit::edit(edited)?;

                match validate_and_write(appconfig, edited.as_str(), &path, force_write)? {
                    // Validated means written without errors.
                    // In this case:
                    // - we also enable it (if `enable`` is true)
                    // - we give out user help if provider has no available configuration
                    WriteResult::Validated(dotprompt) => {
                        println!("Saved {}", path.display());
                        if enable_prompt {
                            cmd::enable::exec(promptname)?;
                        }

                        let model_info = dotprompt.model_info()?;
                        match appconfig.providers.resolve(&model_info.provider) {
                            ProviderVariant::Anthropic(conf) => {
                                if conf.api_key(&appconfig.providers).is_none() {
                                    println!("{}", templates::ONBOARDING_ANTHROPIC);
                                }
                            },
                            ProviderVariant::OpenAi(conf) => {
                                if conf.api_key(&appconfig.providers).is_none() {
                                    println!("{}", templates::ONBOARDING_OPENAI);
                                }
                            },
                            ProviderVariant::Google(conf) => {
                                if conf.api_key(&appconfig.providers).is_none() {
                                    println!("{}", templates::ONBOARDING_GOOGLE);
                                }
                            },
                            _ => {}
                        }
                        break;
                    }
                    // Written means there were errors, but the user forced writing the file.
                    // In this case we don't enable the prompt automatically, even if requested.
                    WriteResult::Written => {
                        println!("Saved {}", path.display());
                        if enable_prompt {
                            println!("Not enabling due to errors");
                        }
                        break;
                    }
                    // User aborted writing
                    WriteResult::Aborted => {
                        println!("No changes, did not save");
                        break;
                    }
                    // User choose to re-edit, thus will not break the loop
                    WriteResult::Edit => {}
                }
            }
        }
    };
    Ok(())
}
