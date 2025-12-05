use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use anyhow::{Context, Result};

const prompts_namespace: &str = "prompts.d";
const app_name: &str = "aibox";


pub fn get_user_promptfile_path(promptname: &str) -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Could not get config dir")?;

    let config_filename: String =  format!("{promptname}.prompt");

    Ok(config_dir
        .join(app_name)
        .join(prompts_namespace)
        .join(config_filename))
}

pub fn get_prompt_search_dirs() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // 1. Current directory
    if let Ok(pwd) = env::current_dir() {
        paths.push(pwd);
    }

    // 2. User config directory
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir
            .join(app_name)
            .join(prompts_namespace));

        paths.push(config_dir.join(app_name));
    }

    // 3. System config directory (platform-specific)
    #[cfg(target_os = "linux")]
    {
        paths.push(PathBuf::from("/etc")
            .join(app_name)
            .join(prompts_namespace));
            
        paths.push(PathBuf::from("/etc").join(app_name));
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
pub fn get_promptfile_paths(promptname: &str) -> Vec<PathBuf> {
    let config_filename: String =  format!("{promptname}.prompt");

    get_prompt_search_dirs()
        .iter().map(|path| path.join(config_filename.as_str())).collect()
}

/// Finds the first existing config file
pub fn find_promptfile(promptname: &str) -> Option<PathBuf> {
    get_promptfile_paths(promptname)
        .into_iter()
        .find(|path| path.exists() && path.is_file())
}

/// Reads the first existing config file
pub fn read_promptfile(promptname: &str) -> Result<String, std::io::Error> {
    match find_promptfile(promptname) {
        Some(path) => fs::read_to_string(path),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No config file found in search paths",
        )),
    }
}
