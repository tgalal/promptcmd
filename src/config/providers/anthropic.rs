use serde::Deserialize;
use serde::Serialize;

use crate::config::providers;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AnthropicProviders {

    #[serde(flatten)]
    pub config: AnthropicConfig,
    
    #[serde(flatten)]
    pub named: std::collections::HashMap<String, AnthropicConfig>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct AnthropicConfig {
    temperature: Option<f32>,
    system: Option<String>,
    stream: Option<bool>,
    max_tokens: Option<u32>,

    pub api_key: String
}


impl AnthropicConfig {
    pub fn temperature(&self, providers: &providers::Providers) -> f32 {
        self.temperature.or(
            providers.anthropic.config.temperature.or(
                providers.temperature
            )
        ).unwrap_or(providers::DEFAULT_TEMPERATURE)
    }

    pub fn stream(&self, providers: &providers::Providers) -> bool {
        self.stream.or(
            providers.anthropic.config.stream.or(
                providers.stream
            )
        ).unwrap_or(providers::DEFAULT_STREAM)
    }

    pub fn max_tokens(&self, providers: &providers::Providers) -> u32 {
        self.max_tokens.or(
            providers.anthropic.config.max_tokens.or(
                providers.max_tokens
            )
        ).unwrap_or(providers::DEFAULT_MAX_TOKENS)
    }
}
