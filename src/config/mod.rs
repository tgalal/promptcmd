pub mod bin_locator;
pub mod promptfile_locator;
pub mod appconfig;
pub mod appconfig_locator;
pub mod providers;

use std::path::PathBuf;

use thiserror::Error;

pub const RUNNER_BIN_NAME: &str = "promptcmd";
pub const APP_NAME: &str = "promptcmd";
pub const PROMPTS_STORAGE_DIR: &str = "prompts.d";
pub const PROMPTS_INSTALL_DIR: &str = "bin";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not determine a storage directory")]
    StorageDirectoryNotAvailable,

    #[error("Could not create storage directory")]
    CreateStorageDirectoryFailed(#[from] std::io::Error),

    #[error("Could not find base config directory")]
    BaseConfigDirNotAvailable,

    #[error("Could not find base prompt directory")]
    BasePromptDirNotAvailable,
}

pub fn base_storage_dir() -> Result<PathBuf, ConfigError>  {
    if let Some(config_dir) = dirs::config_dir() {
        Ok(config_dir.join(APP_NAME))
    } else {
        Err(ConfigError::BaseConfigDirNotAvailable)
    }
}

pub fn prompt_install_dir() -> Result<PathBuf, ConfigError> {
    Ok(base_storage_dir()?.join(PROMPTS_INSTALL_DIR))
}

pub fn prompt_storage_dir() -> Result<PathBuf, ConfigError> {
    Ok(base_storage_dir()?.join(PROMPTS_STORAGE_DIR))
}

pub fn bootstrap_directories() -> Result<(), ConfigError> {
    let prompts_install_dir = prompt_install_dir()?;
    let prompts_storage_dir = prompt_storage_dir()?;

    std::fs::create_dir_all(prompts_storage_dir)?;
    std::fs::create_dir_all(prompts_install_dir)?;

    Ok(()) 
}
