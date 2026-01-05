use llm::builder::LLMBuilder;

use crate::create_provider;
use std::convert::From;
use std::env;
use log::debug;
use std::fmt;
use serde::Deserialize;

create_provider!("anthropic" {
    api_key: String,
});

impl TryFrom<&ResolvedProviderConfig> for LLMBuilder {
    type Error = error::ToLLMBuilderError;

    fn try_from(config: &ResolvedProviderConfig) -> std::result::Result<Self, Self::Error> {
        let mut builder = LLMBuilder::new()
            .backend(llm::builder::LLMBackend::Anthropic);

        if let Some(temperature) = config.temperature.as_ref() {
            builder = builder.temperature(temperature.value);
        }

        if let Some(system) = config.system.as_ref() {
            builder = builder.system(&system.value);
        }

        if let Some(max_tokens) = config.max_tokens.as_ref() {
            builder = builder.max_tokens(max_tokens.value);
        }

        builder = builder.model(
            config.model.as_ref().ok_or(
                error::ToLLMBuilderError::RequiredConfiguration("model")
            )?.value.clone()
        );

        builder = builder.api_key(
            config.api_key.as_ref().ok_or(
                error::ToLLMBuilderError::RequiredConfiguration("api_key")
            )?.value.clone()
        );

        Ok(builder)
    }
}
