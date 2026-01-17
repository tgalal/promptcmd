pub mod promptfiles_fs;
pub mod promptfiles_mem;
use std::collections::HashMap;

use thiserror::Error;


#[derive(Error, Debug)]
pub enum PromptFilesStorageError {
    #[error("IO Error; {0}")]
    FSError(#[from] std::io::Error),

    #[error("Prompt not found at: {0}")]
    PromptNotFound(String),

    #[error("Failed to write prompt file, reason: {0}")]
    Other(String),
}

pub trait PromptFilesStorage: Send + Sync {
    fn list(&self) -> Result<HashMap<String, String>, PromptFilesStorageError>;
    fn exists(&self, identifier: &str) -> Option<String>;
    fn store(&self, identifier: &str, dotpromptdata: &str) -> Result<String, PromptFilesStorageError>;
    fn load(&self, identifier: &str) -> Result<(String, String), PromptFilesStorageError>;
}
