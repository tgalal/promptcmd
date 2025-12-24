use std::{collections::HashMap, fs, path::PathBuf};

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

    fn resolve(&self, identifier: &str) -> PathBuf {
        let promptfilename: String =  format!("{identifier}.prompt");

        self.root_dir.join(promptfilename)
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
        let path = self.resolve(identifier);
        if path.exists() {
            return Some(path.to_string_lossy().into_owned())
        }
        return None;
    }

    fn store(&mut self, identifier: &str, dotpromptdata: &[u8]) -> Result<String, PromptFilesStorageError> {
        let filepath = self.resolve(identifier);
        fs::write(&filepath, dotpromptdata)?;

        Ok(filepath.to_string_lossy().into_owned())
    }

    fn load(&self, identifier: &str) -> Result<(String, Vec<u8>), PromptFilesStorageError> {
        let filepath = self.resolve(identifier);
        let data = fs::read(&filepath)?;

        Ok((filepath.to_string_lossy().into_owned(), data))
    }
}
