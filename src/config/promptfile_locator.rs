use std::path::{PathBuf};

use crate::config::ConfigError;
use crate::config::{prompt_storage_dir};

/*
* Returns list of search paths for promptfiles.
* If promptfile name is specified, will join it to the search paths.
*/
pub fn search_paths(promptname: Option<&str>) -> Result<Vec<PathBuf>, ConfigError> {
    let mut paths = Vec::new();

    paths.push(prompt_storage_dir()?);


    Ok(if let Some(promptname) = promptname {
        let promptfile: String =  format!("{promptname}.prompt");
        paths.iter().map(|path| path.join(&promptfile)).collect()
    } else {
        paths
    })
}

pub fn path(promptname: &str) -> Result<PathBuf, ConfigError> {
    let paths = search_paths(Some(promptname))?;
    paths.first().ok_or(
        ConfigError::StorageDirectoryNotAvailable
    ).cloned()
}

pub fn find(promptname_or_path: &str) -> Option<PathBuf> {
    let promptpath: PathBuf = promptname_or_path.into();

    if promptpath.exists() {
        Some(promptpath)
    } else {
        let promptname = promptname_or_path;
        if let Ok(paths) = search_paths(Some(promptname)) {
            paths.into_iter()
            .find(|path| path.exists() && path.is_file())
        } else {
            None
        }
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
        assert!(!paths.unwrap().is_empty(), "Should return at least one search path");
    }

    #[test]
    fn test_search_paths_with_promptname() {
        let promptname = "test";
        let paths = search_paths(Some(promptname)).unwrap();

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
    fn test_path_returns_first_search_path() {
        let promptname = "test";
        let search_paths_result = search_paths(Some(promptname)).unwrap();
        let path_result = path(promptname).unwrap();

        assert_eq!(
            path_result,
            search_paths_result.first().cloned().unwrap(),
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

    #[test]
    fn test_promptname_gets_prompt_extension() {
        let promptname = "translate";
        let paths = search_paths(Some(promptname)).unwrap();

        for path in paths {
            let filename = path.file_name().unwrap().to_string_lossy();
            assert_eq!(
                filename, "translate.prompt",
                "Should append .prompt extension"
            );
        }
    }
}
