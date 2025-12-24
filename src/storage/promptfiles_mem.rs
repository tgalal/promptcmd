use std::{collections::HashMap};

use crate::storage::{PromptFilesStorage, PromptFilesStorageError};


struct InMemoryPromptFilesStorage {
    storage: HashMap<String, Vec<u8>>
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

    fn store(&mut self, identifier: &str, dotprompt: &[u8]) -> Result<String, PromptFilesStorageError> {
        let key: String = identifier.to_string();
        let value: Vec<u8> = dotprompt.to_vec().clone();
        self.storage.insert(key, value);

        Ok(identifier.to_string())
    }

    fn load(&self, identifier: &str) -> Result<(String, Vec<u8>), PromptFilesStorageError> {
        if let Some(dotprompt) = self.storage.get(identifier) {
            Ok((identifier.to_string(), dotprompt.clone()))
        } else {
            Err(PromptFilesStorageError::Other(String::from("No such identifier")))
        }
    }
}
