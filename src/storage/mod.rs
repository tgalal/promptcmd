pub mod promptfiles_fs;
pub mod promptfiles_mem;
use std::collections::HashMap;

use thiserror::Error;


#[derive(Error, Debug)]
pub enum PromptFilesStorageError {
    #[error("IO Error")]
    FSError(#[from] std::io::Error),

    #[error("Failed to write prompt file, reason: {0}")]
    Other(String),
}

pub trait PromptFilesStorage  {
    fn list(&self) -> Result<HashMap<String, String>, PromptFilesStorageError>;
    fn exists(&self, identifier: &str) -> Option<String>;
    fn store(&mut self, identifier: &str, dotpromptdata: &str) -> Result<String, PromptFilesStorageError>;
    fn load(&self, identifier: &str) -> Result<(String, String), PromptFilesStorageError>;
}
