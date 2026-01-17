use std::{collections::HashMap, sync::Mutex};

use crate::storage::{PromptFilesStorage, PromptFilesStorageError};


#[derive(Default)]
pub struct InMemoryPromptFilesStorage {
    storage: Mutex<HashMap<String, String>>
}

impl PromptFilesStorage for InMemoryPromptFilesStorage {
    fn list(&self) -> Result<HashMap<String, String>, PromptFilesStorageError> {
        let storage=  self.storage.lock().unwrap();
        Ok(storage.keys().map(|key| (key.clone(), key.clone())).collect())
    }

    fn exists(&self, identifier: &str) -> Option<String> {
        let storage=  self.storage.lock().unwrap();
        if storage.contains_key(identifier) {
            Some(identifier.to_string())
        } else {
            None
        }
    }

    fn store(&self, identifier: &str, dotprompt: &str) -> Result<String, PromptFilesStorageError> {
        let key: String = identifier.to_string();
        let value: String = dotprompt.to_string();
        let mut storage=  self.storage.lock().unwrap();
        storage.insert(key, value);

        Ok(identifier.to_string())
    }

    fn load(&self, identifier: &str) -> Result<(String, String), PromptFilesStorageError> {
        let storage=  self.storage.lock().unwrap();
        if let Some(dotprompt) = storage.get(identifier) {
            Ok((identifier.to_string(), dotprompt.clone()))
        } else {
            Err(PromptFilesStorageError::Other(String::from("No such identifier")))
        }
    }
}
