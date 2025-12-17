use llm::builder::LLMBuilder;
use serde::{Serialize, Deserialize};
use crate::config::providers::{self, ProviderError, ToLLMProvider};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OpenAIProviders {

    #[serde(flatten)]
    pub config: OpenAIConfig,
    
    #[serde(flatten)]
    pub named: std::collections::HashMap<String, OpenAIConfig>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct OpenAIConfig {
    temperature: Option<f32>,
    system: Option<String>,
    stream: Option<bool>,
    max_tokens: Option<u32>,

    #[serde(default)]
    endpoint: Option<String>,

    #[serde(default)]
    api_key: Option<String>,
}

impl OpenAIConfig {
    pub fn api_key(&self, providers: &providers::Providers) -> Option<String> {
        return if let Some(ref api_key) = self.endpoint {
            Some(api_key.to_string())
        } else if let Some(ref api_key) = providers.openai.config.api_key {
            Some(api_key.to_string())
        }  else {
            None
        }
    }

    pub fn endpoint(&self, providers: &providers::Providers) -> Option<String> {
        return if let Some(ref endpoint) = self.endpoint {
            Some(endpoint.to_string())
        } else if let Some(ref endpoint) = providers.openai.config.endpoint {
            Some(endpoint.to_string())
        } else {
            None
        }
    }

    pub fn temperature(&self, providers: &providers::Providers) -> f32 {
        self.temperature.or(
            providers.openai.config.temperature.or(
                providers.temperature
            )
        ).unwrap_or(providers::DEFAULT_TEMPERATURE)
    }

    pub fn stream(&self, providers: &providers::Providers) -> bool {
        self.stream.or(
            providers.openai.config.stream.or(
                providers.stream
            )
        ).unwrap_or(providers::DEFAULT_STREAM)
    }

    pub fn max_tokens(&self, providers: &providers::Providers) -> u32 {
        self.max_tokens.or(
            providers.openai.config.max_tokens.or(
                providers.max_tokens
            )
        ).unwrap_or(providers::DEFAULT_MAX_TOKENS)
    }
}

impl ToLLMProvider for OpenAIConfig {
    fn llm_provider(&self,
        llmbuilder: LLMBuilder,
        providers: &providers::Providers) -> Result<Box< dyn llm::LLMProvider>, providers::ProviderError> {
            let mut builder = llmbuilder.backend(llm::builder::LLMBackend::OpenAI);

            let api_key = self.api_key(providers);
            let endpoint = self.endpoint(providers);

            if api_key.is_none() && endpoint.is_none() {
                return Err(ProviderError::ConfigurationError { desc: String::from("OpenAI provider requires api_key, or endpoint, or both. Neither was provided.") })
            }

            if let Some(api_key) = &api_key {
                builder = builder.api_key(api_key);
            };

            if let Some(endpoint) = &endpoint {
                builder = builder.base_url(endpoint);
                if api_key.is_none() {
                    builder = builder.api_key("none");
                }
            };

            Ok(builder.build()?)
    }
}
