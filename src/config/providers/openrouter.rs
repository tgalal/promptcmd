use llm::builder::LLMBuilder;
use serde::{Serialize, Deserialize};
use crate::config::providers::{self, ProviderError, ToLLMProvider};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OpenRouterProviders  {

    #[serde(flatten)]
    pub config: OpenRouterConfig,
    
    #[serde(flatten)]
    pub named: std::collections::HashMap<String, OpenRouterConfig>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct OpenRouterConfig {
    temperature: Option<f32>,
    system: Option<String>,
    stream: Option<bool>,
    max_tokens: Option<u32>,

    api_key: Option<String>
}

impl OpenRouterConfig {
    pub fn api_key(&self, providers: &providers::Providers) -> Option<String> {
        return if let Some(ref api_key) = self.api_key {
            Some(api_key.to_string())
        } else if let Some(ref api_key) = providers.openrouter.config.api_key {
            Some(api_key.to_string())
        }  else {
            None
        }
    }

    pub fn temperature(&self, providers: &providers::Providers) -> f32 {
        self.temperature.or(
            providers.openrouter.config.temperature.or(
                providers.temperature
            )
        ).unwrap_or(providers::DEFAULT_TEMPERATURE)
    }

    pub fn stream(&self, providers: &providers::Providers) -> bool {
        self.stream.or(
            providers.openrouter.config.stream.or(
                providers.stream
            )
        ).unwrap_or(providers::DEFAULT_STREAM)
    }

    pub fn max_tokens(&self, providers: &providers::Providers) -> u32 {
        self.max_tokens.or(
            providers.openrouter.config.max_tokens.or(
                providers.max_tokens
            )
        ).unwrap_or(providers::DEFAULT_MAX_TOKENS)
    }
}

impl ToLLMProvider for OpenRouterConfig {
    fn llm_provider(&self,
        llmbuilder: LLMBuilder,
        providers: &providers::Providers) -> Result<Box< dyn llm::LLMProvider>, providers::ProviderError> {
            let api_key = match self.api_key(providers) {
                Some(api_key) => api_key,
                None =>  {
                    return Err(ProviderError::MissingRequiredConfiguration { name: "api_key".to_string() })
                }
            };
            let builder = llmbuilder.backend(llm::builder::LLMBackend::OpenRouter)
                .max_tokens(self.max_tokens(providers))
                .stream(self.stream(providers))
                .temperature(self.temperature(providers))
                .api_key(api_key);

        Ok(builder.build()?)
    }
}
