use std::path::{PathBuf};
use std::env;

const PROMPTFILES_NAMESPACE: &str = "prompts.d";
const APP_NAME: &str = "aibox";

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
