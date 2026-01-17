use llm::builder::LLMBuilder;

use crate::create_provider;
use std::convert::From;
use std::env;
use log::debug;
use std::fmt;
use serde::Deserialize;

create_provider!("openrouter" {
    api_key: String,
});

impl TryFrom<&ResolvedProviderConfig> for LLMBuilder {
    type Error = error::ToLLMBuilderError;

    fn try_from(config: &ResolvedProviderConfig) -> std::result::Result<Self, Self::Error> {
        let mut builder = LLMBuilder::new()
            .backend(llm::builder::LLMBackend::OpenRouter);

        if let Some(temperature) = config.globals.temperature.as_ref() {
            builder = builder.temperature(temperature.value);
        }

        if let Some(system) = config.globals.system.as_ref() {
            builder = builder.system(&system.value);
        }

        if let Some(max_tokens) = config.globals.max_tokens.as_ref() {
            builder = builder.max_tokens(max_tokens.value);
        }

        builder = builder.api_key(
            config.api_key.as_ref().ok_or(
                error::ToLLMBuilderError::RequiredConfiguration("openrouter", "api_key")
            )?.value.clone()
        );

        builder = builder.model(
            config.globals.model.as_ref().ok_or(
                error::ToLLMBuilderError::RequiredConfiguration("openrouter", "model")
            )?.value.clone()
        );

        Ok(builder)
    }
}
