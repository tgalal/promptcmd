use chrono::Utc;

use crate::stats::store::{self, LogRecord, StatsStore};

pub struct PartialLogRecord {
    pub statsstore: &'static dyn StatsStore,
    pub promptname: String,
    pub group: Option<String>,
    pub variant: Option<String>,
    pub provider: String,
    pub model: String,
    pub cache_key: Option<i64>,
}

pub struct ExecutionLogData<'a> {
     pub prompt_tokens: u32,
     pub completion_tokens: u32,
     pub result: &'a str,
     pub success: bool,
     pub time_taken: u32
}

impl PartialLogRecord {
    pub fn log(&self, execdata: ExecutionLogData) -> Result<(), store::LogError> {
        self.statsstore.log(LogRecord {
            promptname: self.promptname.clone(),
            provider: self.provider.clone(),
            model: self.model.clone(),
            variant: self.variant.clone(),
            group: self.group.clone(),
            prompt_tokens: execdata.prompt_tokens,
            completion_tokens: execdata.completion_tokens,
            result: execdata.result.to_string(),
            success: execdata.success,
            time_taken: execdata.time_taken,
            created: Utc::now(),
            cache_key: self.cache_key
        })
    }
}
