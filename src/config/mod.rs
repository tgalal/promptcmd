pub mod appconfig;
pub mod appconfig_locator;
pub mod providers;
pub mod resolver;

use std::path::PathBuf;
use log::debug;

use thiserror::Error;

pub const RUNNER_BIN_NAME: &str = "promptcmd";
pub const APP_NAME: &str = "promptcmd";
pub const PROMPTS_STORAGE_DIR: &str = "prompts";
pub const PROMPTS_INSTALL_DIR: &str = "bin";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not determine a storage directory")]
    StorageDirectoryNotAvailable,

    #[error("Could not create storage directory")]
    CreateStorageDirectoryFailed(#[from] std::io::Error),

    #[error("Could not determine config directory")]
    BaseConfigDirNotAvailable,

    #[error("Could not determine prompt directory")]
    BasePromptDirNotAvailable,

    #[error("Could not determine data directory")]
    DataDirNotAvailable,
}

pub fn base_home_dir() -> Result<PathBuf, ConfigError>  {
    if let Some(data_dir) = dirs::home_dir() {
        let mut home_storage_dir = String::from(".");
        home_storage_dir.push_str(APP_NAME);
        Ok(data_dir.join(home_storage_dir.as_str()))
    } else {
        Err(ConfigError::StorageDirectoryNotAvailable)
    }
}

pub fn base_config_dir() -> Result<PathBuf, ConfigError>  {
    if let Some(dir) = dirs::config_dir() {
        Ok(dir.join(APP_NAME))
    } else {
        Err(ConfigError::BaseConfigDirNotAvailable)
    }
}

pub fn prompt_install_dir() -> Result<PathBuf, ConfigError> {
    if let Ok(base_dir) = base_home_dir() {
        Ok(base_dir.join(PROMPTS_INSTALL_DIR))
    } else {
        Err(ConfigError::StorageDirectoryNotAvailable)
    }
}

pub fn prompt_storage_dir() -> Result<PathBuf, ConfigError> {
    if let Ok(dir) = base_config_dir() {
        Ok(dir.join(PROMPTS_STORAGE_DIR))
    }else {
        Err(ConfigError::BasePromptDirNotAvailable)
    }
}

pub fn bootstrap_directories() -> Result<(), ConfigError> {
    let prompts_install_dir = prompt_install_dir()?;
    let prompts_storage_dir = prompt_storage_dir()?;
    let config_dir = base_config_dir()?;

    debug!("Config Dir: {}", &config_dir.to_string_lossy());
    debug!("Prompts Storage Dir: {}", &prompts_storage_dir.to_string_lossy());
    debug!("Prompts Installation Dir: {}", &prompts_install_dir.to_string_lossy());

    std::fs::create_dir_all(prompts_storage_dir)?;
    std::fs::create_dir_all(prompts_install_dir)?;

    Ok(())
}
