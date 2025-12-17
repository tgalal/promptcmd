use std::path::{PathBuf};
use std::env;

use crate::config::APP_NAME;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_paths_returns_non_empty() {
        let paths = search_paths();
        assert!(!paths.is_empty(), "Should return at least one search path");
    }

    #[test]
    fn test_search_paths_includes_current_dir() {
        let paths = search_paths();

        // First path should be current directory + config.toml
        let first_path = &paths[0];
        assert!(
            first_path.ends_with(CONFIG_NAME),
            "First path should end with config.toml"
        );
    }

    #[test]
    fn test_search_paths_includes_user_config() {
        let paths = search_paths();

        // At least one path should contain the app name
        let has_app_name = paths.iter().any(|p| {
            p.to_string_lossy().contains(APP_NAME)
        });

        assert!(has_app_name, "Should include path with app name");
    }

    #[test]
    fn test_search_paths_contains_config_name() {
        let paths = search_paths();

        // All paths should end with config.toml
        for path in paths {
            assert!(
                path.ends_with(CONFIG_NAME),
                "All paths should end with config.toml: {:?}",
                path
            );
        }
    }

    #[test]
    fn test_path_returns_first_search_path() {
        let search_paths_result = search_paths();
        let path_result = path();

        assert_eq!(
            path_result,
            search_paths_result.first().cloned(),
            "path() should return the first search path"
        );
    }

    #[test]
    fn test_path_returns_some() {
        let result = path();
        assert!(result.is_some(), "path() should return Some value");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_system_path_included() {
        let paths = search_paths();

        let has_etc_path = paths.iter().any(|p| {
            p.to_string_lossy().contains("/etc/aibox")
        });

        assert!(has_etc_path, "Should include /etc/aibox path on Linux");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_system_path_included() {
        let paths = search_paths();

        let has_library_path = paths.iter().any(|p| {
            p.to_string_lossy().contains("/Library/Application Support/aibox")
        });

        assert!(has_library_path, "Should include Library path on macOS");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_system_path_included() {
        let paths = search_paths();

        let has_program_data = paths.iter().any(|p| {
            p.to_string_lossy().contains("ProgramData")
        });

        assert!(has_program_data, "Should include ProgramData path on Windows");
    }
}

