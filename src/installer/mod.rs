pub mod symlink;

use std::collections::HashMap;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("{0} is already installed at {1}")]
    AlreadyExists(String, String),

    #[error("IO Error")]
    IOError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum UninstallError {
    #[error("{0} is not installed")]
    NotInstalled(String),

    #[error("IO Error")]
    IOError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ListError {
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
}

pub trait DotPromptInstaller {
    fn install(&mut self, name: &str) -> Result<String, InstallError>;
    fn uninstall(&mut self, name: &str) -> Result<String, UninstallError>;
    fn is_installed(&self, name: &str) -> Option<String>;
    fn list(&self) -> Result<HashMap<String, String>, ListError>;
}
#[cfg(test)]
pub mod tests {
    use std::collections::HashSet;

    use crate::installer::{DotPromptInstaller, InstallError, UninstallError};

    #[derive(Default)]
    pub struct InMemoryInstaller {
        installed: HashSet<String>
    }

    impl DotPromptInstaller for InMemoryInstaller {
        fn install(&mut self, name: &str) -> Result<String, super::InstallError> {
            if self.installed.contains(name) {
                Err(InstallError::AlreadyExists(name.to_string(), name.to_string()))
            } else {
                self.installed.insert(name.to_string());
                Ok(name.to_string())
            }
        }

        fn uninstall(&mut self, name: &str) -> Result<String, super::UninstallError> {
            if !self.installed.contains(name) {
                Err(UninstallError::NotInstalled(name.to_string()))
            } else {
                self.installed.remove(name);
                Ok(name.to_string())
            }

        }

        fn is_installed(&self, name: &str) -> Option<String> {
            if self.installed.contains(name) {
                Some(name.to_string())
            } else {
                None
            }
        }

        fn list(&self) -> Result<std::collections::HashMap<String, String>, super::ListError> {
            Ok(self.installed.iter().map(|item| (item.to_string(), item.to_string())).collect())
        }
    }
}
