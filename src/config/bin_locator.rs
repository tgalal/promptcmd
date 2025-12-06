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
