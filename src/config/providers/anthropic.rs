use llm::builder::LLMBuilder;
use serde::Deserialize;
use serde::Serialize;

use crate::config::providers::ProviderError;
use crate::config::providers::{self, ToLLMProvider};

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
    default_model: Option<String>,

    api_key: Option<String>
}


impl AnthropicConfig {
    pub fn api_key(&self, providers: &providers::Providers) -> Option<String> {
        if let Some(ref api_key) = self.api_key {
            Some(api_key.to_string())
        } else if let Some(ref api_key) = providers.anthropic.config.api_key {
            Some(api_key.to_string())
        }  else {
            None
        }
    }

    pub fn default_model(&self, providers: &providers::Providers) -> Option<String> {
        if let Some(ref default_model) = self.default_model {
            Some(default_model.to_string())
        } else if let Some(ref default_model) = providers.anthropic.config.default_model {
            Some(default_model.to_string())
        } else {
            None
        }
    }

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

impl ToLLMProvider for AnthropicConfig {
    fn llm_provider(&self,
        llmbuilder: LLMBuilder,
        providers: &providers::Providers) -> Result<Box< dyn llm::LLMProvider>, providers::ProviderError> {

            let api_key = self.api_key(providers);
            if api_key.is_none() {
                return Err(ProviderError::ConfigurationError { desc: String::from("Anthropic provider requires api_key.") })
            }

            let builder = llmbuilder.backend(llm::builder::LLMBackend::Anthropic)
                .api_key(api_key.unwrap())
                .max_tokens(self.max_tokens(providers))
                .stream(self.stream(providers))
                .temperature(self.temperature(providers));

        Ok(builder.build()?)
    }
}
