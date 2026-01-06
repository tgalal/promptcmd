use std::{collections::HashMap, fs, path::PathBuf};
use symlink::remove_symlink_file;
use ::symlink::symlink_file;

use crate::installer::{DotPromptInstaller, InstallError, UninstallError};


const INSTALLER_ID: &str = "symlink";

pub struct SymlinkInstaller {
    target: PathBuf,
    install_dir: PathBuf
}

impl SymlinkInstaller {
    pub fn new(target: PathBuf, install_dir: PathBuf) -> Self {
        Self {
            target,
            install_dir: install_dir.join(INSTALLER_ID)
        }
    }

    fn resolve(&self, name: &str) -> (PathBuf, String) {

        #[cfg(target_os="windows")]
        let name = name.to_string() + ".exe";

        let install_path = self.install_dir.join(name);
        let install_path_str = install_path.to_string_lossy().into_owned();

        (install_path, install_path_str)
    }
}

impl DotPromptInstaller for SymlinkInstaller {
    fn install(&mut self, name: &str) -> Result<String, InstallError> {
        let (install_path, install_path_str) = self.resolve(name);

        if install_path.exists() {
            return Err(InstallError::AlreadyExists(name.to_string(), install_path_str.clone()));
        }

        #[cfg(unix)]
        symlink_file(&self.target, &install_path)?;

        #[cfg(target_os="windows")]
        fs::hard_link(&self.target, &install_path)?;

        Ok(install_path_str)
    }

    fn uninstall(&mut self, name: &str) -> Result<String, super::UninstallError> {
        let (install_path, install_path_str) = self.resolve(name);

        if !install_path.exists() {
            return Err(UninstallError::NotInstalled(name.to_string()));
        }

        #[cfg(unix)]
        remove_symlink_file(&install_path)?;

        #[cfg(target_os="windows")]
        fs::remove_file(&install_path)?;

        Ok(install_path_str)
    }

    fn is_installed(&self, name: &str) -> Option<String> {
        let (install_path, install_path_str) = self.resolve(name);

        if install_path.exists() {
            Some(install_path_str)
        } else {
            None
        }
    }

    fn list(&self) -> Result<std::collections::HashMap<String, String>, super::ListError> {
        let mut result: HashMap<String, String> = HashMap::new();

        if ! fs::exists(&self.install_dir)? {
            return Ok(result)
        }

        let dir_entries = fs::read_dir(&self.install_dir)?;

        for entry in dir_entries {
            let path = entry?.path();

            if path.is_file() &&
                let Ok(actual_target) = fs::read_link(&path) &&
                actual_target == self.target &&
                let Some(promptname) = path.file_stem() {
                    result.insert(
                        promptname.to_string_lossy().into_owned(),
                        path.to_string_lossy().into_owned());
                }
        }

        Ok(result)
    }
}

