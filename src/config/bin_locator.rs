use std::path::{PathBuf};
use thiserror::Error;

use crate::config::{prompt_install_dir, ConfigError};


#[derive(Error, Debug)]
pub enum BinLocatorError {
    #[error("Could not create directory")]
    CreateBinDirFailed(#[from] std::io::Error),

    #[error("Could locate bin directory")]
    LocateBinDirFailed
}

/*
* Returns list of search paths executables/symlinks.
* If bin is specified, will join it to the search paths.
*/
pub fn search_paths(bin: Option<&str>) -> Result<Vec<PathBuf>, ConfigError> {
    let mut paths = Vec::new();

    paths.push(prompt_install_dir()?);

    if let Some(bin) = bin {
        Ok(paths.iter().map(|path| path.join(bin)).collect())
    } else {
        Ok(paths)
    }
}

pub fn path(bin: Option<&str>) -> Result<PathBuf, ConfigError> {
    let paths = search_paths(bin)?;
    paths.first().ok_or(
        ConfigError::StorageDirectoryNotAvailable
    ).cloned()
}

pub fn find(bin: &str) -> Option<PathBuf> {
    if let Ok(paths) = search_paths(Some(bin)) {
        paths.into_iter()
        .find(|path| path.exists() && path.is_file())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_paths_without_bin() {
        let paths = search_paths(None);

        // Should return at least one path on most systems
        assert!(
            !paths.unwrap().is_empty(),
            "Should return executable directories"
        );
    }

    #[test]
    fn test_search_paths_with_bin() {
        let bin_name = "test-binary";
        let paths = search_paths(Some(bin_name)).unwrap();

        // If there are paths, they should all end with the bin name
        for path in paths {
            assert!(
                path.ends_with(bin_name),
                "Path {:?} should end with {}",
                path,
                bin_name
            );
        }
    }

    #[test]
    fn test_path_returns_first_search_path() {
        let bin_name = "test-binary";
        let search_paths_result = search_paths(Some(bin_name)).unwrap();
        let path_result = path(Some(bin_name)).unwrap();

        assert_eq!(
            path_result,
            search_paths_result.first().cloned().unwrap(),
            "path() should return the first search path"
        );
    }

    #[test]
    fn test_find_nonexistent_bin() {
        // Use a very unlikely binary name that shouldn't exist
        let nonexistent = "this-binary-should-not-exist-xyz123";
        let result = find(nonexistent);

        assert!(
            result.is_none(),
            "Should return None for nonexistent binary"
        );
    }

}
