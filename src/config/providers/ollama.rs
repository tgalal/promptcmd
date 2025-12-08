use serde::{Serialize, Deserialize};
use crate::config::providers;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OllamaProviders {
    #[serde(flatten)]
    pub config: OllamaConfig,
    
    #[serde(flatten)]
    pub named: std::collections::HashMap<String, OllamaConfig>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct OllamaConfig {
    temperature: Option<f32>,
    system: Option<String>,
    stream: Option<bool>,
    max_tokens: Option<u32>,

    #[serde(default)]
    pub endpoint: String
}

impl OllamaConfig {
    pub fn temperature(&self, providers: &providers::Providers) -> f32 {
        self.temperature.or(
            providers.ollama.config.temperature.or(
                providers.temperature
            )
        ).unwrap_or(providers::DEFAULT_TEMPERATURE)
    }

    pub fn stream(&self, providers: &providers::Providers) -> bool {
        self.stream.or(
            providers.ollama.config.stream.or(
                providers.stream
            )
        ).unwrap_or(providers::DEFAULT_STREAM)
    }

    pub fn max_tokens(&self, providers: &providers::Providers) -> u32 {
        self.max_tokens.or(
            providers.ollama.config.max_tokens.or(
                providers.max_tokens
            )
        ).unwrap_or(providers::DEFAULT_MAX_TOKENS)
    }
}
