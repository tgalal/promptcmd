use llm::builder::LLMBuilder;

use crate::define_resolved_provider_config;
use crate::resolver::resolved;
use crate::resolver::resolved::ToLLMBuilderError;
use crate::resolver::resolved::{ModelInfo, ToModelInfoError};
use crate::resolver::ResolvedProperty;
use crate::resolver::ResolvedPropertySource;
use std::convert::From;
use std::env;
use log::debug;
use std::fmt;
use serde::Deserialize;

define_resolved_provider_config!("anthropic" {
    api_key: String,
});

impl TryFrom<&ResolvedProviderConfig> for LLMBuilder {
    type Error = ToLLMBuilderError;

    fn try_from(config: &ResolvedProviderConfig) -> std::result::Result<Self, Self::Error> {
        let mut builder = LLMBuilder::new()
            .backend(llm::builder::LLMBackend::Anthropic);

        if let Some(temperature) = config.temperature.as_ref() {
            builder = builder.temperature(temperature.value);
        }

        if let Some(system) = config.system.as_ref() {
            builder = builder.system(&system.value);
        }

        builder = builder.model(
            config.model.as_ref().ok_or(
                ToLLMBuilderError::RequiredConfiguration("model")
            )?.value.clone()
        );

        builder = builder.api_key(
            config.api_key.as_ref().ok_or(
                ToLLMBuilderError::RequiredConfiguration("api_key")
            )?.value.clone()
        );

        // if config.api_key.is_none() {
        //     return Err(ToLLMBuilderError::RequiredConfiguration("api_key"))
        // }

        Ok(builder)
    }
}
