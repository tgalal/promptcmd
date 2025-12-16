use std::path::{PathBuf};
use std::env;

use crate::config::APP_NAME;

const PROMPTFILES_NAMESPACE: &str = "prompts.d";

/*
* Returns list of search paths for promptfiles.
* If promptfile name is specified, will join it to the search paths.
*/
pub fn search_paths(promptname: Option<&str>) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // 1. Current directory
    if let Ok(pwd) = env::current_dir() {
        paths.push(pwd);
    }

    // 2. User config directory
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir
            .join(APP_NAME)
            .join(PROMPTFILES_NAMESPACE));

        paths.push(config_dir.join(APP_NAME));
    }

    // 3. System config directory (platform-specific)
    #[cfg(target_os = "linux")]
    {
        paths.push(PathBuf::from("/etc")
            .join(APP_NAME)
            .join(PROMPTFILES_NAMESPACE));
            
        paths.push(PathBuf::from("/etc").join(APP_NAME));
    }

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Library/Application Support")
            .join(APP_NAME)
            .join(PROMPTFILES_NAMESPACE));

        paths.push(PathBuf::from("/Library/Application Support")
            .join(APP_NAME));
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(program_data) = dirs::data_dir() {
            // ProgramData is typically accessed via a different function on Windows
            // For system-wide config, you might use a hardcoded path or environment variable
            let system_config = PathBuf::from("C:\\ProgramData")
                .join(APP_NAME)
                .join(PROMPTFILES_NAMESPACE);
            paths.push(system_config);
        }
    }

    if let Some(promptname) = promptname {
        let promptfile: String =  format!("{promptname}.prompt");
        paths.iter().map(|path| path.join(&promptfile)).collect()
    } else {
        paths
    }
}

pub fn path(promptname: &str) -> Option<PathBuf> {
    search_paths(Some(promptname)).first().cloned()
}

pub fn find(promptname_or_path: &str) -> Option<PathBuf> {
    let promptpath: PathBuf = promptname_or_path.into();

    if promptpath.exists() {
        Some(promptpath)
    } else {
        let promptname = promptname_or_path;
        search_paths(Some(promptname))
            .into_iter()
            .find(|path| path.exists() && path.is_file()
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_search_paths_without_promptname() {
        let paths = search_paths(None);

        // Should return at least one path (current directory)
        assert!(!paths.is_empty(), "Should return at least one search path");
    }

    #[test]
    fn test_search_paths_with_promptname() {
        let promptname = "test";
        let paths = search_paths(Some(promptname));

        // All paths should end with "test.prompt"
        let expected_filename = format!("{}.prompt", promptname);
        for path in paths {
            assert!(
                path.ends_with(&expected_filename),
                "Path {:?} should end with {}",
                path,
                expected_filename
            );
        }
    }

    #[test]
    fn test_search_paths_includes_current_dir() {
        let paths = search_paths(None);

        // First path should be current directory
        if let Ok(pwd) = env::current_dir() {
            assert_eq!(paths[0], pwd, "First path should be current directory");
        }
    }

    #[test]
    fn test_search_paths_includes_app_name() {
        let paths = search_paths(None);

        // At least one path should contain the app name
        let has_app_name = paths.iter().any(|p| {
            p.to_string_lossy().contains(APP_NAME)
        });

        assert!(has_app_name, "Should include path with app name");
    }

    #[test]
    fn test_search_paths_includes_namespace() {
        let paths = search_paths(None);

        // At least one path should contain the prompts namespace
        let has_namespace = paths.iter().any(|p| {
            p.to_string_lossy().contains(PROMPTFILES_NAMESPACE)
        });

        assert!(has_namespace, "Should include path with prompts.d namespace");
    }

    #[test]
    fn test_path_returns_first_search_path() {
        let promptname = "test";
        let search_paths_result = search_paths(Some(promptname));
        let path_result = path(promptname);

        assert_eq!(
            path_result,
            search_paths_result.first().cloned(),
            "path() should return the first search path"
        );
    }

    #[test]
    fn test_find_with_existing_path() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("existing.prompt");

        // Create a test file
        fs::write(&test_file, "test content").unwrap();

        // Test with full path
        let result = find(test_file.to_str().unwrap());

        assert_eq!(
            result,
            Some(test_file),
            "Should find existing file by path"
        );
    }

    #[test]
    fn test_find_nonexistent_promptname() {
        // Use a very unlikely prompt name that shouldn't exist
        let nonexistent = "this-prompt-should-not-exist-xyz123";
        let result = find(nonexistent);

        assert!(
            result.is_none(),
            "Should return None for nonexistent prompt"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_system_paths_included() {
        let paths = search_paths(None);

        let has_etc_namespace = paths.iter().any(|p| {
            p.to_string_lossy().contains("/etc/aibox/prompts.d")
        });

        let has_etc_app = paths.iter().any(|p| {
            let s = p.to_string_lossy();
            s.contains("/etc/aibox") && !s.contains("prompts.d")
        });

        assert!(has_etc_namespace || has_etc_app, "Should include /etc/aibox paths on Linux");
    }

    #[test]
    fn test_promptname_gets_prompt_extension() {
        let promptname = "translate";
        let paths = search_paths(Some(promptname));

        for path in paths {
            let filename = path.file_name().unwrap().to_string_lossy();
            assert_eq!(
                filename, "translate.prompt",
                "Should append .prompt extension"
            );
        }
    }
}
