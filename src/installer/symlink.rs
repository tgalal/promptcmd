use std::{collections::HashMap, fs, io, path::PathBuf};
use symlink::remove_symlink_file;
use ::symlink::symlink_file;
use log::warn;

use crate::installer::{DotPromptInstaller, InstallError, UninstallError};

pub struct SymlinkInstaller {
    target: PathBuf,
    install_dir: PathBuf
}

impl SymlinkInstaller {
    pub fn new(target: PathBuf, install_dir: PathBuf) -> Self {
        Self {
            target,
            install_dir
        }
    }

    fn safe_resolve(&self, name: &str) -> io::Result<PathBuf> {
        #[cfg(target_os="windows")]
        let name = name.to_string() + ".exe";

        let base = self.install_dir.canonicalize()?;
        let raw = base.join(name);

        // Canonicalize the parent (which must exist), then re-append the filename
        let parent = raw.parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no parent"))?
            .canonicalize()?;
        let file_name = raw.file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no file name"))?;

        let resolved = parent.join(file_name);

        if !resolved.starts_with(&base) {
            warn!("Path traversal detected: {}", resolved.to_string_lossy());
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "path traversal detected",
            ));
        }
        Ok(resolved)
    }

}

impl DotPromptInstaller for SymlinkInstaller {
    fn install(&mut self, name: &str) -> Result<String, InstallError> {
        fs::create_dir_all(&self.install_dir)?;

        let install_path = self.safe_resolve(name)?;

        if install_path.exists() {
            return Err(InstallError::AlreadyExists(name.to_string(), install_path.to_string_lossy().to_string()));
        }

        symlink_file(&self.target, &install_path)?;

        Ok(install_path.to_string_lossy().to_string())
    }

    fn uninstall(&mut self, name: &str) -> Result<String, super::UninstallError> {
        let install_path = self.safe_resolve(name)?;

        if !install_path.exists() {
            return Err(UninstallError::NotInstalled(name.to_string()));
        }

        remove_symlink_file(&install_path)?;

        Ok(install_path.to_string_lossy().to_string())
    }

    fn is_installed(&self, name: &str) -> Option<String> {
        let install_path = self.safe_resolve(name).ok();

        if let Some(install_path) = install_path && install_path.exists() {
            Some(install_path.to_string_lossy().to_string())
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
                let Some(promptname) = path.file_name() {

                    let promptname = promptname.to_string_lossy().to_string();

                    #[cfg(target_os="windows")]
                    let promptname: String = if let Some(exe_stripped) = promptname.strip_suffix(".exe") {
                        exe_stripped.to_string()
                    } else {
                        promptname
                    };

                    result.insert(
                        promptname,
                        path.to_string_lossy().into_owned());
                }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;

    fn setup_test_env() -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("target_binary");
        let install_dir = temp_dir.path().join("install");

        // Create the target file
        File::create(&target).unwrap();

        (temp_dir, target, install_dir)
    }

    #[test]
    fn test_new_creates_installer() {
        let (temp_dir, target, install_dir) = setup_test_env();

        let installer = SymlinkInstaller::new(target.clone(), install_dir.clone());

        assert_eq!(installer.target, target);
        assert_eq!(installer.install_dir, install_dir);

        drop(temp_dir);
    }

    #[test]
    fn test_safe_resolve_normal_path() {
        let (temp_dir, target, install_dir) = setup_test_env();
        fs::create_dir_all(&install_dir).unwrap();

        let installer = SymlinkInstaller::new(target, install_dir.clone());
        let resolved = installer.safe_resolve("mycommand").unwrap();

        #[cfg(target_os = "windows")]
        assert!(resolved.ends_with("mycommand.exe"));

        #[cfg(not(target_os = "windows"))]
        assert!(resolved.ends_with("mycommand"));

        assert!(resolved.starts_with(&install_dir.canonicalize().unwrap()));

        drop(temp_dir);
    }

    #[test]
    fn test_safe_resolve_prevents_path_traversal() {
        let (temp_dir, target, install_dir) = setup_test_env();
        fs::create_dir_all(&install_dir).unwrap();

        let installer = SymlinkInstaller::new(target, install_dir);

        // Attempt path traversal with ..
        let result = installer.safe_resolve("../../../etc/passwd");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied);

        drop(temp_dir);
    }

    #[test]
    fn test_safe_resolve_with_dot_dot_in_name() {
        let (temp_dir, target, install_dir) = setup_test_env();
        fs::create_dir_all(&install_dir).unwrap();

        let installer = SymlinkInstaller::new(target, install_dir);

        // Test various path traversal attempts
        let traversal_attempts = vec!["../escape", "../../dangerous", "../../../etc/passwd"];

        for attempt in traversal_attempts {
            let result = installer.safe_resolve(attempt);
            assert!(result.is_err(), "Should reject path traversal: {}", attempt);
        }

        drop(temp_dir);
    }

    #[test]
    fn test_install_success() {
        let (temp_dir, target, install_dir) = setup_test_env();

        let mut installer = SymlinkInstaller::new(target.clone(), install_dir.clone());
        let result = installer.install("testcmd");

        assert!(result.is_ok());
        let install_path = result.unwrap();

        #[cfg(target_os = "windows")]
        assert!(install_path.ends_with("testcmd.exe"));

        #[cfg(not(target_os = "windows"))]
        assert!(install_path.ends_with("testcmd"));

        // Verify symlink was created
        let path = PathBuf::from(&install_path);
        assert!(path.exists());
        assert!(fs::read_link(&path).is_ok());

        drop(temp_dir);
    }

    #[test]
    fn test_install_already_exists() {
        let (temp_dir, target, install_dir) = setup_test_env();

        let mut installer = SymlinkInstaller::new(target.clone(), install_dir.clone());

        // First install should succeed
        installer.install("duplicate").unwrap();

        // Second install should fail
        let result = installer.install("duplicate");
        assert!(result.is_err());

        match result {
            Err(InstallError::AlreadyExists(name, _)) => {
                assert_eq!(name, "duplicate");
            }
            _ => panic!("Expected AlreadyExists error"),
        }

        drop(temp_dir);
    }

    #[test]
    fn test_install_prevents_path_traversal() {
        let (temp_dir, target, install_dir) = setup_test_env();

        let mut installer = SymlinkInstaller::new(target, install_dir);

        let result = installer.install("../evil");
        assert!(result.is_err());

        match result {
            Err(InstallError::IOError(e)) => {
                assert_eq!(e.kind(), io::ErrorKind::PermissionDenied);
            }
            _ => panic!("Expected IOError with PermissionDenied"),
        }

        drop(temp_dir);
    }

    #[test]
    fn test_uninstall_success() {
        let (temp_dir, target, install_dir) = setup_test_env();

        let mut installer = SymlinkInstaller::new(target.clone(), install_dir.clone());

        // Install first
        installer.install("removeme").unwrap();

        // Now uninstall
        let result = installer.uninstall("removeme");
        assert!(result.is_ok());

        // Verify symlink was removed
        let install_path = result.unwrap();
        let path = PathBuf::from(&install_path);
        assert!(!path.exists());

        drop(temp_dir);
    }

    #[test]
    fn test_uninstall_not_installed() {
        let (temp_dir, target, install_dir) = setup_test_env();
        fs::create_dir_all(&install_dir).unwrap();

        let mut installer = SymlinkInstaller::new(target, install_dir);

        let result = installer.uninstall("nonexistent");
        assert!(result.is_err());

        match result {
            Err(UninstallError::NotInstalled(name)) => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected NotInstalled error"),
        }

        drop(temp_dir);
    }

    #[test]
    fn test_uninstall_prevents_path_traversal() {
        let (temp_dir, target, install_dir) = setup_test_env();
        fs::create_dir_all(&install_dir).unwrap();

        let mut installer = SymlinkInstaller::new(target, install_dir);

        let result = installer.uninstall("../etc/passwd");
        assert!(result.is_err());

        // Path traversal should be rejected with an IOError
        match result {
            Err(UninstallError::IOError(_)) => {
                // Success - path traversal was prevented
            }
            _ => panic!("Expected IOError for path traversal attempt"),
        }

        drop(temp_dir);
    }

    #[test]
    fn test_is_installed_when_installed() {
        let (temp_dir, target, install_dir) = setup_test_env();

        let mut installer = SymlinkInstaller::new(target.clone(), install_dir.clone());
        installer.install("installed").unwrap();

        let result = installer.is_installed("installed");
        assert!(result.is_some());

        drop(temp_dir);
    }

    #[test]
    fn test_is_installed_when_not_installed() {
        let (temp_dir, target, install_dir) = setup_test_env();

        let installer = SymlinkInstaller::new(target, install_dir);

        let result = installer.is_installed("notinstalled");
        assert!(result.is_none());

        drop(temp_dir);
    }

    #[test]
    fn test_is_installed_with_path_traversal() {
        let (temp_dir, target, install_dir) = setup_test_env();

        let installer = SymlinkInstaller::new(target, install_dir);

        // Path traversal attempts should return None
        let result = installer.is_installed("../etc/passwd");
        assert!(result.is_none());

        drop(temp_dir);
    }

    #[test]
    fn test_list_empty() {
        let (temp_dir, target, install_dir) = setup_test_env();

        let installer = SymlinkInstaller::new(target, install_dir);

        let result = installer.list();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);

        drop(temp_dir);
    }

    #[test]
    fn test_special_name() {
        let (temp_dir, target, install_dir) = setup_test_env();

        let mut installer = SymlinkInstaller::new(target.clone(), install_dir.clone());

        // Install multiple commands
        installer.install("READFORME.md").unwrap();

        let result = installer.list();
        assert!(result.is_ok());

        let list = result.unwrap();
        assert_eq!(list.len(), 1);
        assert!(list.contains_key("READFORME.md"));

        drop(temp_dir);
    }

    #[test]
    fn test_list_with_installed_commands() {
        let (temp_dir, target, install_dir) = setup_test_env();

        let mut installer = SymlinkInstaller::new(target.clone(), install_dir.clone());

        // Install multiple commands
        installer.install("cmd1").unwrap();
        installer.install("cmd2").unwrap();
        installer.install("cmd3").unwrap();

        let result = installer.list();
        assert!(result.is_ok());

        let list = result.unwrap();
        assert_eq!(list.len(), 3);
        assert!(list.contains_key("cmd1"));
        assert!(list.contains_key("cmd2"));
        assert!(list.contains_key("cmd3"));

        drop(temp_dir);
    }

    #[test]
    fn test_list_ignores_non_symlinks() {
        let (temp_dir, target, install_dir) = setup_test_env();
        fs::create_dir_all(&install_dir).unwrap();

        let mut installer = SymlinkInstaller::new(target.clone(), install_dir.clone());

        // Install a legitimate symlink
        installer.install("legit").unwrap();

        // Create a regular file in the install dir
        File::create(install_dir.join("regularfile")).unwrap();

        let result = installer.list();
        assert!(result.is_ok());

        let list = result.unwrap();
        // Should only contain the symlink, not the regular file
        assert_eq!(list.len(), 1);
        assert!(list.contains_key("legit"));
        assert!(!list.contains_key("regularfile"));

        drop(temp_dir);
    }

    #[test]
    fn test_list_ignores_symlinks_to_different_targets() {
        let (temp_dir, target, install_dir) = setup_test_env();
        fs::create_dir_all(&install_dir).unwrap();

        // Create another target
        let other_target = temp_dir.path().join("other_target");
        File::create(&other_target).unwrap();

        let mut installer = SymlinkInstaller::new(target.clone(), install_dir.clone());

        // Install a legitimate symlink
        installer.install("ours").unwrap();

        // Create a symlink to a different target
        let other_link = install_dir.join("others");
        symlink_file(&other_target, &other_link).unwrap();

        let result = installer.list();
        assert!(result.is_ok());

        let list = result.unwrap();
        // Should only contain symlinks pointing to our target
        assert_eq!(list.len(), 1);
        assert!(list.contains_key("ours"));
        assert!(!list.contains_key("others"));

        drop(temp_dir);
    }
}

