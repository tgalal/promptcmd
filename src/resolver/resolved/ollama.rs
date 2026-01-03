use llm::builder::LLMBuilder;

use crate::config::providers::ollama::OllamaConfig;
use crate::config::providers;
use crate::define_resolved_provider_config;
use crate::resolver::resolved::ToLLMBuilderError;
use crate::resolver::ResolvedProperty;
use crate::resolver::ResolvedPropertySource;
use crate::resolver::resolved::{ModelInfo, ToModelInfoError};
use std::convert::From;
use std::env;
use log::debug;
use std::fmt;
use serde::Deserialize;

define_resolved_provider_config!(OllamaConfig {
    endpoint: String,
});

impl TryFrom<&ResolvedProviderConfig> for LLMBuilder {
    type Error = ToLLMBuilderError;

    fn try_from(config: &ResolvedProviderConfig) -> std::result::Result<Self, Self::Error> {
        let mut builder = LLMBuilder::new()
            .backend(llm::builder::LLMBackend::Ollama);

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

        builder = builder.base_url(
            config.endpoint.as_ref().ok_or(
                ToLLMBuilderError::RequiredConfiguration("endpoint")
            )?.value.clone()
        );

        Ok(builder)
    }
}
