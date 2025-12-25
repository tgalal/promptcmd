use std::{collections::HashMap};

use crate::storage::{PromptFilesStorage, PromptFilesStorageError};


pub struct InMemoryPromptFilesStorage {
    storage: HashMap<String, String>
}

impl InMemoryPromptFilesStorage {
    pub fn new() -> Self {
        InMemoryPromptFilesStorage {
           storage: HashMap::new()
        }
    }
}


impl PromptFilesStorage for InMemoryPromptFilesStorage {
    fn list(&self) -> Result<HashMap<String, String>, PromptFilesStorageError> {
        Ok(self.storage.keys().map(|key| (key.clone(), key.clone())).collect())
    }

    fn exists(&self, identifier: &str) -> Option<String> {
        if self.storage.contains_key(identifier) {
            Some(identifier.to_string())
        } else {
            None
        }
    }

    fn store(&mut self, identifier: &str, dotprompt: &str) -> Result<String, PromptFilesStorageError> {
        let key: String = identifier.to_string();
        let value: String = dotprompt.to_string();
        self.storage.insert(key, value);

        Ok(identifier.to_string())
    }

    fn load(&self, identifier: &str) -> Result<(String, String), PromptFilesStorageError> {
        if let Some(dotprompt) = self.storage.get(identifier) {
            Ok((identifier.to_string(), dotprompt.clone()))
        } else {
            Err(PromptFilesStorageError::Other(String::from("No such identifier")))
        }
    }
}
