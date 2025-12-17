use llm::builder::LLMBuilder;
use serde::Deserialize;
use serde::Serialize;

use crate::config::providers::{self, ToLLMProvider};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GoogleProviders {

    #[serde(flatten)]
    pub config: GoogleConfig,
    
    #[serde(flatten)]
    pub named: std::collections::HashMap<String, GoogleConfig>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct GoogleConfig {
    temperature: Option<f32>,
    system: Option<String>,
    stream: Option<bool>,
    max_tokens: Option<u32>,
    api_key: String
}

impl GoogleConfig {
    pub fn temperature(&self, providers: &providers::Providers) -> f32 {
        self.temperature.or(
            providers.google.config.temperature.or(
                providers.temperature
            )
        ).unwrap_or(providers::DEFAULT_TEMPERATURE)
    }

    pub fn stream(&self, providers: &providers::Providers) -> bool {
        self.stream.or(
            providers.google.config.stream.or(
                providers.stream
            )
        ).unwrap_or(providers::DEFAULT_STREAM)
    }

    pub fn max_tokens(&self, providers: &providers::Providers) -> u32 {
        self.max_tokens.or(
            providers.google.config.max_tokens.or(
                providers.max_tokens
            )
        ).unwrap_or(providers::DEFAULT_MAX_TOKENS)
    }
}

impl ToLLMProvider for GoogleConfig {
    fn llm_provider(&self,
        llmbuilder: LLMBuilder,
        providers: &providers::Providers) -> Result<Box< dyn llm::LLMProvider>, providers::ProviderError> {
            let builder = llmbuilder.backend(llm::builder::LLMBackend::Google)
                .api_key(&self.api_key)
                .max_tokens(self.max_tokens(providers))
                .stream(self.stream(providers))
                .temperature(self.temperature(providers));
            
        Ok(builder.build()?)
    }
}
