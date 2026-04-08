use llm::builder::LLMBuilder;

use crate::create_provider;
use std::convert::From;
use std::env;
use log::debug;
use std::fmt;
use serde::Deserialize;

const DEFAULT_BASE_URL: &str = "https://api.minimax.io/v1";

create_provider!("minimax" {
    api_key: String,
    endpoint: String,
});

impl TryFrom<&ResolvedProviderConfig> for LLMBuilder {
    type Error = error::ToLLMBuilderError;

    fn try_from(config: &ResolvedProviderConfig) -> std::result::Result<Self, Self::Error> {
        let mut builder = LLMBuilder::new()
            .backend(llm::builder::LLMBackend::OpenAI);

        if let Some(temperature) = config.globals.temperature.as_ref() {
            builder = builder.temperature(temperature.value);
        }

        if let Some(system) = config.globals.system.as_ref() {
            builder = builder.system(&system.value);
        }

        if let Some(max_tokens) = config.globals.max_tokens.as_ref() {
            builder = builder.max_tokens(max_tokens.value);
        }

        let base_url = config.endpoint.as_ref()
            .map(|e| e.value.clone())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        builder = builder.base_url(&base_url);

        builder = builder.api_key(
            config.api_key.as_ref().ok_or(
                error::ToLLMBuilderError::RequiredConfiguration("minimax", "api_key")
            )?.value.clone()
        );

        builder = builder.model(
            config.globals.model.as_ref().ok_or(
                error::ToLLMBuilderError::RequiredConfiguration("minimax", "model")
            )?.value.clone()
        );

        Ok(builder)
    }
}
