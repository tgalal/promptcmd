use std::time::Instant;

use llm::{chat::StreamResponse, error::LLMError};
use futures::{Stream, StreamExt};
use async_recursion::async_recursion;
use log::error;

use crate::{dotprompt::OutputFormat, executor::{partiallog::{ExecutionLogData, PartialLogRecord}, streaming_code_extractor::StreamingCodeExtractor, ExecutorErorr}};


pub struct StructuredStreamingExecutionOutput {
    pub partial_log_record: PartialLogRecord,
    pub stream: std::pin::Pin<Box<dyn Stream<Item = Result<StreamResponse, LLMError>> + Send>>,
    result_data: String,
    start_time: Instant,
    usage_prompt_tokens: u32,
    usage_completion_tokens: u32,
    output_format: OutputFormat,
    streaming_code_extractor: StreamingCodeExtractor,
    found_fenced_code: bool
}

impl StructuredStreamingExecutionOutput {
    pub fn new(
        partial_log_record: PartialLogRecord,
        stream: std::pin::Pin<Box<dyn Stream<Item = Result<StreamResponse, LLMError>> + Send>>,
        output_format: OutputFormat
    ) -> Self {

        Self {
            partial_log_record,
            stream,
            result_data: String::new(),
            start_time: Instant::now(),
            usage_prompt_tokens: 0,
            usage_completion_tokens: 0,
            output_format,
            streaming_code_extractor: StreamingCodeExtractor::new(),
            found_fenced_code: false
        }
    }

    pub async fn sync_collect(&mut self) -> Result<String, ExecutorErorr> {
        let mut result = String::new();

        while let Some(res) = self.sync_next().await {
            result.push_str(res?.as_str());
        }

        Ok(result)
    }

    #[async_recursion]
    pub async fn sync_next(&mut self) -> Option<Result<String, ExecutorErorr>> {
        match self.stream.next().await.map(|res| res.map_err(ExecutorErorr::LLMError)) {
            Some(Ok(res)) => {
                let mapped = res.choices.iter()
                    .filter_map(
                        |item| item.delta.content.as_ref()
                            .map(|c| c.to_string())
                    ).collect::<Vec<_>>()
                    .join("");
                self.result_data.push_str(mapped.as_str());

                if let Some(usage) = res.usage.as_ref() {
                    self.usage_prompt_tokens = usage.prompt_tokens;
                    self.usage_completion_tokens = usage.completion_tokens;
                }
                // If the requested output is code and we found a code fence, then the we omit
                // everything except the code within the fence.
                self.found_fenced_code = self.found_fenced_code || self.result_data.contains("```");
                if matches!(self.output_format, OutputFormat::Code) && self.found_fenced_code {
                    let mut buffer = String::new();
                    let parsing_state = self.streaming_code_extractor.feed(&mapped, &mut buffer);
                    if parsing_state {
                        Some(Ok(buffer))
                    } else {
                        self.sync_next().await
                    }
                } else {
                    Some(Ok(mapped))
                }
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
                    prompt_tokens: self.usage_prompt_tokens,
                    completion_tokens: self.usage_completion_tokens,
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
