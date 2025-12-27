use thiserror::Error;

use chrono::{DateTime, Utc};

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Error initializing store: {0}")]
    InitError(String)
}

#[derive(Debug, Error)]
pub enum LogError {
    #[error("Error storing record: {0}")]
    GeneralError(String)
}

#[derive(Debug, Error)]
    #[error("Error fetching records: {0}")]
pub enum FetchError {
    GeneralError(String)
}

pub struct LogRecord {
    pub promptname: String,
    pub provider: String,
    pub model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub result: String,
    pub success: bool,
    pub time_taken: u32,
    pub created: DateTime<Utc>
}

#[derive(Debug)]
pub struct SummaryItem {
    pub provider: String,
    pub model: String,
    pub count: i32,
    pub prompt_tokens: i32,
    pub completion_tokens: i32
}

pub trait StatsStore {
    fn log(&self, item: LogRecord) -> Result<(), LogError>;
    fn all(&self) -> Result<Vec<LogRecord>, FetchError>;
    fn summary(&self) -> Result<Vec<SummaryItem>, FetchError>;
}
