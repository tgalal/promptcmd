use std::path::{PathBuf};
use std::env;

const APP_NAME: &str = "aibox";
const CONFIG_NAME: &str="config.toml";

/*
* Returns list of search paths for promptfiles.
* If promptfile name is specified, will join it to the search paths.
*/
pub fn search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // 1. Current directory
    if let Ok(pwd) = env::current_dir() {
        paths.push(pwd.join(CONFIG_NAME));
    }

    // 2. User config directory
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join(APP_NAME).join(CONFIG_NAME));
    }

    // 3. System config directory (platform-specific)
    #[cfg(target_os = "linux")]
    {
        paths.push(PathBuf::from("/etc").join(APP_NAME).join(CONFIG_NAME));
    }

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Library/Application Support")
            .join(APP_NAME).join(CONFIG_NAME));
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(program_data) = dirs::data_dir() {
            // ProgramData is typically accessed via a different function on Windows
            // For system-wide config, you might use a hardcoded path or environment variable
            let system_config = PathBuf::from("C:\\ProgramData")
                .join(APP_NAME).join(CONFIG_NAME);
            paths.push(system_config);
        }
    }

    paths
}

pub fn path() -> Option<PathBuf> {
    search_paths().first().cloned()
}

