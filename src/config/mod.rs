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
pub const ENV_PROMPTS_STORAGE_DIR: &str = "PROMPTCMD_PROMPTS_DIR";
pub const ENV_PROMPTS_INSTALL_DIR: &str = "PROMPTCMD_INSTALL_DIR";
pub const ENV_HOME_DIR: &str = "PROMPTCMD_HOME_DIR";
pub const ENV_CONFIG_DIR: &str = "PROMPTCMD_CONFIG_DIR";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not determine a storage directory")]
    StorageDirectoryNotAvailable,

    #[error("Path set in Environment {0}={1} does not exist")]
    EnvDirDoesNotExist(&'static str, String),

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
    let env_dir  = std::env::var(ENV_HOME_DIR)
        .map(PathBuf::from);

    if let Ok(env_dir) = env_dir {
        if !env_dir.exists() || !env_dir.is_dir() {
            Err(ConfigError::EnvDirDoesNotExist(ENV_HOME_DIR, env_dir.to_string_lossy().to_string()))
        } else {
            Ok(env_dir)
        }
    } else if let Some(data_dir) = dirs::home_dir() {
        let mut home_storage_dir = String::from(".");
        home_storage_dir.push_str(APP_NAME);
        Ok(data_dir.join(home_storage_dir.as_str()))
    } else {
        Err(ConfigError::StorageDirectoryNotAvailable)
    }
}

pub fn base_config_dir() -> Result<PathBuf, ConfigError>  {
    let env_dir  = std::env::var(ENV_CONFIG_DIR)
        .map(PathBuf::from);

    if let Ok(env_dir) = env_dir {
        if !env_dir.exists() || !env_dir.is_dir() {
            Err(ConfigError::EnvDirDoesNotExist(ENV_CONFIG_DIR, env_dir.to_string_lossy().to_string()))
        } else {
            Ok(env_dir)
        }
    } else if let Some(dir) = dirs::config_dir() {
        Ok(dir.join(APP_NAME))
    } else {
        Err(ConfigError::BaseConfigDirNotAvailable)
    }
}

pub fn prompt_install_dir() -> Result<PathBuf, ConfigError> {
    let env_install_dir  = std::env::var(ENV_PROMPTS_INSTALL_DIR)
        .map(PathBuf::from);

    if let Ok(env_install_dir) = env_install_dir {
        if !env_install_dir.exists() || !env_install_dir.is_dir() {
            Err(ConfigError::EnvDirDoesNotExist(ENV_PROMPTS_INSTALL_DIR, env_install_dir.to_string_lossy().to_string()))
        } else {
            Ok(env_install_dir)
        }
    } else if let Ok(base_dir) = base_home_dir() {
        Ok(base_dir.join(PROMPTS_INSTALL_DIR))
    } else {
        Err(ConfigError::StorageDirectoryNotAvailable)
    }
}

pub fn prompt_storage_dir() -> Result<PathBuf, ConfigError> {
    let env_dir  = std::env::var(ENV_PROMPTS_STORAGE_DIR)
        .map(PathBuf::from);

    if let Ok(env_dir) = env_dir {
        if !env_dir.exists() || !env_dir.is_dir() {
            Err(ConfigError::EnvDirDoesNotExist(ENV_PROMPTS_STORAGE_DIR, env_dir.to_string_lossy().to_string()))
        } else {
            Ok(env_dir)
        }
    } else if let Ok(dir) = base_config_dir() {
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

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_home_dir() {
        let actual = base_home_dir().unwrap();
        let expected = dirs::home_dir().unwrap().join(format!(".{APP_NAME}"));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_home_dir_from_env() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().join("home/");
        let expected = temp_path.clone();

        temp_env::with_vars([(ENV_HOME_DIR, Some(temp_path.to_string_lossy().to_string()))], || {
            let actual  = base_home_dir();
            // should err because path does not exist (yet)
            assert!(actual.is_err());
            std::fs::create_dir_all(temp_path).unwrap();

            let actual  = base_home_dir().unwrap();
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn test_config_dir() {
        let actual = base_config_dir().unwrap();
        let expected = dirs::config_dir().unwrap().join(APP_NAME);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_config_dir_from_env() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().join("config/");
        let expected = temp_path.clone();

        temp_env::with_vars([(ENV_CONFIG_DIR, Some(temp_path.to_string_lossy().to_string()))], || {
            let actual  = base_config_dir();
            // should err because path does not exist (yet)
            assert!(actual.is_err());
            std::fs::create_dir_all(temp_path).unwrap();

            let actual  = base_config_dir().unwrap();
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn test_prompts_dir() {
        let prompts_dir = prompt_storage_dir().unwrap();
        let expected = base_config_dir().unwrap().join(PROMPTS_STORAGE_DIR);
        assert_eq!(expected, prompts_dir);
    }

    #[test]
    fn test_prompts_dir_from_env() {

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().join("prompts/");
        let expected = temp_path.clone();

        temp_env::with_vars([(ENV_PROMPTS_STORAGE_DIR, Some(temp_path.to_string_lossy().to_string()))], || {
            let actual  = prompt_storage_dir();
            // should err because path does not exist (yet)
            assert!(actual.is_err());
            std::fs::create_dir_all(temp_path).unwrap();

            let actual  = prompt_storage_dir().unwrap();
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn test_prompts_in_alt_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().join("config/");
        let expected = temp_path.clone().join(PROMPTS_STORAGE_DIR);
        std::fs::create_dir_all(&temp_path).unwrap();

        temp_env::with_vars([(ENV_CONFIG_DIR, Some(temp_path.to_string_lossy().to_string()))], || {
            let actual  = prompt_storage_dir().unwrap();
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn test_install_dir() {
        let dir = prompt_install_dir().unwrap();
        let expected = base_home_dir().unwrap().join(PROMPTS_INSTALL_DIR);
        assert_eq!(expected, dir);
    }

    #[test]
    fn test_install_in_alt_home() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().join("home/");
        let expected = temp_path.clone().join(PROMPTS_INSTALL_DIR);
        std::fs::create_dir_all(&temp_path).unwrap();

        temp_env::with_vars([(ENV_HOME_DIR, Some(temp_path.to_string_lossy().to_string()))], || {
            let actual  = prompt_install_dir().unwrap();
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn test_install_from_env() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().join("install/");
        let expected = temp_path.clone();

        temp_env::with_vars([(ENV_PROMPTS_INSTALL_DIR, Some(temp_path.to_string_lossy().to_string()))], || {
            let actual  = prompt_install_dir();
            // should err because path does not exist (yet)
            assert!(actual.is_err());
            std::fs::create_dir_all(temp_path).unwrap();

            let actual  = prompt_install_dir().unwrap();
            assert_eq!(expected, actual);
        });
    }
}
