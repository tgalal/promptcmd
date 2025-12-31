use llm::builder::LLMBuilder;
use serde::Deserialize;
use serde::Serialize;
use log::debug;

use crate::config::providers::ProviderError;
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
    default_model: Option<String>,
    api_key: Option<String>
}

impl GoogleConfig {
    pub fn api_key(&self, providers: &providers::Providers) -> Option<String> {
        if let Some(ref api_key) = self.api_key {
            Some(api_key.to_string())
        } else if let Some(ref api_key) = providers.google.config.api_key {
            Some(api_key.to_string())
        }  else {
            None
        }
    }

    pub fn system(&self, providers: &providers::Providers) -> Option<String> {
        if let Some(ref system) = self.system {
            Some(system.to_string())
        } else if let Some(ref system) = providers.google.config.system {
            Some(system.to_string())
        } else {
            providers.system.clone()
        }
    }

    pub fn default_model(&self, providers: &providers::Providers) -> Option<String> {
        if let Some(ref default_model) = self.default_model {
            Some(default_model.to_string())
        } else if let Some(ref default_model) = providers.google.config.default_model {
            Some(default_model.to_string())
        } else {
            None
        }
    }

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
            let api_key = self.api_key(providers);
            if api_key.is_none() {
                return Err(ProviderError::ConfigurationError { desc: String::from("Google provider requires api_key.") })
            }

            let mut builder = llmbuilder.backend(llm::builder::LLMBackend::Google)
                .api_key(api_key.unwrap())
                .max_tokens(self.max_tokens(providers))
                .stream(self.stream(providers))
                .temperature(self.temperature(providers));

            if let Some(system) = self.system(providers) {
                if system.len() > 70 {
                    debug!("System message: {}...", &system[..75]);
                } else {
                    debug!("System message: {}", &system);
                }
            }

        Ok(builder.build()?)
    }
}
