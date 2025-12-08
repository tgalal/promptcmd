use std::path::{PathBuf};

/*
* Returns list of search paths executables/symlinks.
* If bin is specified, will join it to the search paths.
*/
pub fn search_paths(bin: Option<&str>) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(exec_dir) = dirs::executable_dir() {
        paths.push(exec_dir)
    } 

    if let Some(bin) = bin {
        paths.iter().map(|path| path.join(bin)).collect()
    } else {
        paths
    }
}

pub fn path(bin: &str) -> Option<PathBuf> {
    search_paths(Some(bin)).first().cloned()
}

pub fn find(bin: &str) -> Option<PathBuf> {
    search_paths(Some(bin))
        .into_iter()
        .find(|path| path.exists() && path.is_file()
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_paths_without_bin() {
        let paths = search_paths(None);

        // Should return at least one path on most systems
        assert!(
            !paths.is_empty(),
            "Should return executable directories"
        );
    }

    #[test]
    fn test_search_paths_with_bin() {
        let bin_name = "test-binary";
        let paths = search_paths(Some(bin_name));

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
        let search_paths_result = search_paths(Some(bin_name));
        let path_result = path(bin_name);

        assert_eq!(
            path_result,
            search_paths_result.first().cloned(),
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
