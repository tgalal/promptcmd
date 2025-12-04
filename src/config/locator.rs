use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use anyhow::{Context, Result};

pub struct ConfigLocator {
    app_name: String,
    config_namespace: String,
    config_filename: String,
}

impl ConfigLocator {
    pub fn new(app_name: impl Into<String>, config_namespace: impl Into<String>, config_filename: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            config_namespace: config_namespace.into(),
            config_filename: config_filename.into(),
        }
    }

    pub fn get_user_config_path(&self) -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Could not get config dir")?;

        Ok(config_dir
            .join(&self.app_name)
            .join(&self.config_namespace)
            .join(&self.config_filename))
    }

    pub fn get_prompt_search_dirs(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. Current directory
        if let Ok(pwd) = env::current_dir() {
            paths.push(pwd.join(&self.config_filename));
        }

        // 2. User config directory
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir
                .join(&self.app_name)
                .join(&self.config_namespace));
            paths.push(config_dir.join(&self.app_name));
        }

        // 3. System config directory (platform-specific)
        #[cfg(target_os = "linux")]
        {
            paths.push(PathBuf::from("/etc")
                .join(&self.app_name)
                .join(&self.config_namespace));
                
            paths.push(PathBuf::from("/etc").join(&self.app_name));
        }

        #[cfg(target_os = "macos")]
        {
            paths.push(PathBuf::from("/Library/Application Support")
                .join(&self.app_name)
                .join(&self.config_filename));
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(program_data) = dirs::data_dir() {
                // ProgramData is typically accessed via a different function on Windows
                // For system-wide config, you might use a hardcoded path or environment variable
                let system_config = PathBuf::from("C:\\ProgramData")
                    .join(&self.app_name)
                    .join(&self.config_filename);
                paths.push(system_config);
            }
        }

        paths
    }

    /// Returns all possible config paths in order of precedence
    pub fn get_search_paths(&self) -> Vec<PathBuf> {
        self.get_prompt_search_dirs()
            .iter().map(|path| path.join(&self.config_filename)).collect()
    }

    /// Finds the first existing config file
    pub fn find_config(&self) -> Option<PathBuf> {
        self.get_search_paths()
            .into_iter()
            .find(|path| path.exists() && path.is_file())
    }

    /// Reads the first existing config file
    pub fn read_config(&self) -> Result<String, std::io::Error> {
        match self.find_config() {
            Some(path) => fs::read_to_string(path),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No config file found in search paths",
            )),
        }
    }
}

