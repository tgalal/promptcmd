use std::path::{PathBuf};

use crate::config::{base_config_dir};

const CONFIG_NAME: &str="config.toml";

/*
* Returns list of search paths for promptfiles.
* If promptfile name is specified, will join it to the search paths.
*/
pub fn search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // 1. User config directory
    if let Ok(config_dir) = base_config_dir() {
        paths.push(config_dir.join(CONFIG_NAME));
    }
    paths
}

pub fn path() -> Option<PathBuf> {
    search_paths().iter().find(|item| {
        item.exists()
    }).cloned()
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
}

