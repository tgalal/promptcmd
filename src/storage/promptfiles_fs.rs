use std::{collections::HashMap, fs, io, path::PathBuf};
use log::warn;

use crate::storage::{PromptFilesStorage, PromptFilesStorageError};

pub struct FileSystemPromptFilesStorage {
    root_dir: PathBuf
}

impl FileSystemPromptFilesStorage {
    pub fn new(root_dir: PathBuf) -> Self {
        FileSystemPromptFilesStorage {
            root_dir
        }
    }

    fn safe_resolve(&self, identifier: &str) -> io::Result<PathBuf> {
        let base = self.root_dir.canonicalize()?;
        let raw = base.join(format!("{identifier}.prompt"));

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

impl PromptFilesStorage for FileSystemPromptFilesStorage {

    fn list(&self) -> Result<HashMap<String, String>, PromptFilesStorageError> {
        let mut result: HashMap<String, String> = HashMap::new();

        if ! fs::exists(&self.root_dir)? {
            return Ok(result)
        }

        let dir_entries = fs::read_dir(&self.root_dir)?;

        for entry in dir_entries {
            let path = entry?.path();

            if path.is_file() &&
                let Some(e) = path.extension() &&
                e == "prompt" &&
                let Some(promptname) = path.file_stem() {

                result.insert(
                    promptname.to_string_lossy().into_owned(),
                    path.to_string_lossy().into_owned());
            }
        }

        Ok(result)
    }


    fn exists(&self, identifier: &str) -> Option<String> {
        let path = self.safe_resolve(identifier).ok();

        if let Some(path) = path && path.exists() {
            if path.exists() {
                Some(path.to_string_lossy().into_owned())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn store(&self, identifier: &str, dotpromptdata: &str) -> Result<String, PromptFilesStorageError> {
        let filepath = self.safe_resolve(identifier)?;
        fs::write(&filepath, dotpromptdata)?;

        Ok(filepath.to_string_lossy().into_owned())
    }

    fn load(&self, identifier: &str) -> Result<(String, String), PromptFilesStorageError> {
        let filepath = self.safe_resolve(identifier)?;
        if !filepath.exists() {
            return Err(PromptFilesStorageError::PromptNotFound(filepath.to_string_lossy().to_string()));
        }

        let data = fs::read_to_string(&filepath)?;

        Ok((filepath.to_string_lossy().into_owned(), data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_storage() -> (TempDir, FileSystemPromptFilesStorage) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = FileSystemPromptFilesStorage::new(temp_dir.path().to_path_buf());
        (temp_dir, storage)
    }

    #[test]
    fn test_new_creates_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemPromptFilesStorage::new(temp_dir.path().to_path_buf());
        assert_eq!(storage.root_dir, temp_dir.path());
    }

    #[test]
    fn test_list_empty_directory() {
        let (_temp_dir, storage) = setup_test_storage();
        let result = storage.list().expect("list should succeed");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_list_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent");
        let storage = FileSystemPromptFilesStorage::new(nonexistent_path);

        let result = storage.list().expect("list should succeed on nonexistent directory");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_list_with_prompt_files() {
        let (temp_dir, storage) = setup_test_storage();

        // Create test files
        fs::write(temp_dir.path().join("test1.prompt"), "content1").unwrap();
        fs::write(temp_dir.path().join("test2.prompt"), "content2").unwrap();
        fs::write(temp_dir.path().join("notaprompt.txt"), "other").unwrap();

        let result = storage.list().expect("list should succeed");

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("test1"));
        assert!(result.contains_key("test2"));
        assert!(!result.contains_key("notaprompt"));
    }

    #[test]
    fn test_list_ignores_subdirectories() {
        let (temp_dir, storage) = setup_test_storage();

        fs::write(temp_dir.path().join("file.prompt"), "content").unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        fs::write(temp_dir.path().join("subdir/nested.prompt"), "nested").unwrap();

        let result = storage.list().expect("list should succeed");

        assert_eq!(result.len(), 1);
        assert!(result.contains_key("file"));
    }

    #[test]
    fn test_store_and_load_prompt() {
        let (_temp_dir, storage) = setup_test_storage();

        let content = "This is a test prompt";
        let filepath = storage.store("mytest", content).expect("store should succeed");

        assert!(PathBuf::from(&filepath).exists());

        let (loaded_path, loaded_content) = storage.load("mytest").expect("load should succeed");
        assert_eq!(loaded_path, filepath);
        assert_eq!(loaded_content, content);
    }

    #[test]
    fn test_store_overwrites_existing() {
        let (_temp_dir, storage) = setup_test_storage();

        storage.store("test", "original").expect("first store should succeed");
        storage.store("test", "updated").expect("second store should succeed");

        let (_, content) = storage.load("test").expect("load should succeed");
        assert_eq!(content, "updated");
    }

    #[test]
    fn test_load_nonexistent_prompt() {
        let (_temp_dir, storage) = setup_test_storage();

        let result = storage.load("nonexistent");
        assert!(result.is_err());

        if let Err(PromptFilesStorageError::PromptNotFound(path)) = result {
            assert!(path.contains("nonexistent.prompt"));
        } else {
            panic!("Expected PromptNotFound error");
        }
    }

    #[test]
    fn test_exists_returns_some_for_existing_file() {
        let (_temp_dir, storage) = setup_test_storage();

        storage.store("exists_test", "content").expect("store should succeed");

        let result = storage.exists("exists_test");
        assert!(result.is_some());
        assert!(result.unwrap().contains("exists_test.prompt"));
    }

    #[test]
    fn test_exists_returns_none_for_nonexistent_file() {
        let (_temp_dir, storage) = setup_test_storage();

        let result = storage.exists("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_safe_resolve_prevents_path_traversal() {
        let (temp_dir, storage) = setup_test_storage();

        // Create a file outside the root directory
        let outside_file = temp_dir.path().parent().unwrap().join("outside.prompt");
        fs::write(&outside_file, "malicious").unwrap();

        // Try to access it via path traversal
        let result = storage.exists("../outside");
        assert!(result.is_none());

        // Try to load via path traversal
        let load_result = storage.load("../outside");
        assert!(load_result.is_err());

        // Clean up
        fs::remove_file(outside_file).ok();
    }

    #[test]
    fn test_safe_resolve_allows_valid_identifiers() {
        let (_temp_dir, storage) = setup_test_storage();

        // These should all be valid
        let valid_identifiers = vec![
            "simple",
            "with-dash",
            "with_underscore",
            "with.dot",
            "123numbers",
        ];

        for identifier in valid_identifiers {
            let result = storage.store(identifier, "test content");
            assert!(result.is_ok(), "Failed to store valid identifier: {}", identifier);

            let exists = storage.exists(identifier);
            assert!(exists.is_some(), "exists() failed for valid identifier: {}", identifier);
        }
    }

    #[test]
    fn test_store_creates_file_with_correct_extension() {
        let (temp_dir, storage) = setup_test_storage();

        storage.store("test", "content").expect("store should succeed");

        let expected_path = temp_dir.path().join("test.prompt");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_load_returns_correct_path_and_content() {
        let (_temp_dir, storage) = setup_test_storage();

        let test_content = "Line 1\nLine 2\nLine 3";
        let stored_path = storage.store("multiline", test_content).expect("store should succeed");

        let (loaded_path, loaded_content) = storage.load("multiline").expect("load should succeed");

        assert_eq!(loaded_path, stored_path);
        assert_eq!(loaded_content, test_content);
    }

    #[test]
    fn test_store_with_unicode_content() {
        let (_temp_dir, storage) = setup_test_storage();

        let unicode_content = "Hello 世界 🌍 مرحبا";
        storage.store("unicode", unicode_content).expect("store should succeed");

        let (_, loaded) = storage.load("unicode").expect("load should succeed");
        assert_eq!(loaded, unicode_content);
    }

    #[test]
    fn test_list_after_store() {
        let (_temp_dir, storage) = setup_test_storage();

        storage.store("first", "content1").unwrap();
        storage.store("second", "content2").unwrap();

        let list = storage.list().expect("list should succeed");

        assert_eq!(list.len(), 2);
        assert!(list.contains_key("first"));
        assert!(list.contains_key("second"));
    }

    #[test]
    fn test_empty_content() {
        let (_temp_dir, storage) = setup_test_storage();

        storage.store("empty", "").expect("store should succeed with empty content");

        let (_, content) = storage.load("empty").expect("load should succeed");
        assert_eq!(content, "");
    }
}
