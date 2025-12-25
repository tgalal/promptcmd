use llm::{builder::LLMBuilder};
use serde::{Serialize, Deserialize};
use crate::config::providers::{self, ProviderError, ToLLMProvider};

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
    endpoint: Option<String>
}

impl OllamaConfig {
    pub fn endpoint(&self, providers: &providers::Providers) -> Result<String, ProviderError> {
        if let Some(ref endpoint) = self.endpoint {
            Ok(endpoint.to_string())
        } else if let Some(ref endpoint) = providers.ollama.config.endpoint {
            Ok(endpoint.to_string())
        } else {
            Err(ProviderError::MissingRequiredConfiguration { name: String::from("endpoint") })
        }
    }

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

impl ToLLMProvider for OllamaConfig {
    fn llm_provider(&self,
        llmbuilder: LLMBuilder,
        providers: &providers::Providers) -> Result<Box< dyn llm::LLMProvider>, providers::ProviderError> {
            let endpoint = self.endpoint(providers)?;
            let builder = llmbuilder.backend(llm::builder::LLMBackend::Ollama)
               .base_url(&endpoint)
                .max_tokens(self.max_tokens(providers))
                .stream(self.stream(providers))
                .temperature(self.temperature(providers));
            
        Ok(builder.build()?)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::appconfig::AppConfig;

    #[test]
    fn test_ollama_config_inheritance() {
        let toml_content = r#"
            default_model = "claude-3-5-sonnet-20241022"
            editor = "vim"

            [providers]
            temperature = 0.7

            [providers.ollama]
            endpoint = "http://root_ollama_endpoint"
            temperature = 0.6

            [providers.ollama.custom_ollama1]
            endpoint = "http://custom_ollama1_endpoint"
            temperature = 0.4

            [providers.ollama.custom_ollama2]
            stream = false
        "#;
        let appconfig = AppConfig::try_from(&toml_content.to_string()).unwrap();
        let ollama = &appconfig.providers.ollama.config;
        let custom_ollama1 = &appconfig.providers.ollama.named.get("custom_ollama1").unwrap();
        let custom_ollama2 = &appconfig.providers.ollama.named.get("custom_ollama2").unwrap();

        assert_eq!(ollama.endpoint(&appconfig.providers).unwrap(), "http://root_ollama_endpoint");
        assert_eq!(custom_ollama1.endpoint(&appconfig.providers).unwrap(), "http://custom_ollama1_endpoint");
        assert_eq!(custom_ollama2.endpoint(&appconfig.providers).unwrap(), "http://root_ollama_endpoint");

        assert_eq!(ollama.temperature(&appconfig.providers), 0.6);
        assert_eq!(custom_ollama1.temperature(&appconfig.providers), 0.4);
        assert_eq!(custom_ollama2.temperature(&appconfig.providers), 0.6);
    }
}
