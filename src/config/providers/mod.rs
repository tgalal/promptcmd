use serde::Deserialize;
use crate::resolver::resolved;

pub const DEFAULT_MAX_TOKENS: u32 = 1000;
pub const DEFAULT_STREAM: bool = false;
pub const DEFAULT_TEMPERATURE: f32 = 0.7;
pub const DEFAULT_SYSTEM: &str = "You are useful AI assistant. Give me brief answers. Do not use special formatting like markdown.";

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProviderError {

    #[error("Missing required configuration key {name}")]
    MissingRequiredConfiguration {
        name: String,
    },

    #[error("Configuration error: {desc}")]
    ConfigurationError {
        desc: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct Providers {

    pub temperature: Option<f32>,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,
    pub default: Option<String>,
    pub system: Option<String>,

    #[serde(default)]
    pub ollama: resolved::ollama::Providers,

    #[serde(default)]
    pub openai: resolved::openai::Providers,

    #[serde(default)]
    pub anthropic: resolved::anthropic::Providers,

    // #[serde(default)]
    // pub google: resolved::google::Providers,

    // #[serde(default)]
    // pub openrouter: resolved::openrouter::Providers,
}

impl Default for Providers {
    fn default() -> Self {
        Providers {
            temperature: Some(DEFAULT_TEMPERATURE),
            stream: Some(DEFAULT_STREAM),
            max_tokens: Some(DEFAULT_MAX_TOKENS),
            system: Some(DEFAULT_SYSTEM.to_string()),
            default: None,
            ollama: resolved::ollama::Providers::default(),
            openai: resolved::openai::Providers::default(),
            anthropic: resolved::anthropic::Providers::default(),
            // google: GoogleProviders::default(),
            // openrouter: OpenRouterProviders::default(),
        }
    }
}
