use std::time::Instant;

use llm::{error::LLMError};
use tokio::runtime::Runtime;
use futures::{Stream, StreamExt};
use log::error;

use crate::executor::{partiallog::{ExecutionLogData, PartialLogRecord}, ExecutorErorr};


pub struct StreamingExecutionOutput {
    pub partial_log_record: PartialLogRecord,
    pub sync_runtime: Runtime,
    pub stream: std::pin::Pin<Box<dyn Stream<Item = Result<String, LLMError>>>>,
    result_data: String,
    start_time: Instant
}

impl StreamingExecutionOutput {
    pub fn new(
        partial_log_record: PartialLogRecord,
        sync_runtime: Runtime,
        stream: std::pin::Pin<Box<dyn Stream<Item = Result<String, LLMError>>>>,
    ) -> Self {

        StreamingExecutionOutput {
            partial_log_record,
            sync_runtime,
            stream,
            result_data: String::new(),
            start_time: Instant::now()
        }
    }

    pub fn sync_collect(&mut self) -> Result<String, ExecutorErorr> {
        let mut result = String::new();

        while let Some(res) = self.sync_next() {
            result.push_str(res?.as_str());
        }

        Ok(result)
    }

    pub fn sync_next(&mut self) -> Option<Result<String, ExecutorErorr>> {
        let rt = &self.sync_runtime;
        match rt.block_on(self.stream.next()).map(|res| res.map_err(ExecutorErorr::LLMError)) {
            Some(Ok(res)) => {
                 self.result_data.push_str(res.as_str());
                 Some(Ok(res))
            }
            Some(Err(err)) => {
                if let Err(err) = self.partial_log_record.log(ExecutionLogData {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    result: self.result_data.as_str(),
                    success: false,
                    time_taken: 0,
                }) {
                    error!("Error logging record: {err}");
                }
                Some(Err(err))
            }
            None => {
                let time_taken = self.start_time.elapsed().as_secs() as u32;
                if let Err(err) = self.partial_log_record.log(ExecutionLogData {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    result: self.result_data.as_str(),
                    success: true,
                    time_taken
                }) {
                    error!("Error logging record: {err}");
                }
                None
            }
        }
    }
}
