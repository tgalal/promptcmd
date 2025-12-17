use llm::builder::LLMBuilder;
use serde::{Serialize, Deserialize};
use crate::config::providers::{self, ToLLMProvider};

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

    api_key: String
}

impl ToLLMProvider for OpenRouterConfig {
    fn llm_provider(&self,
        llmbuilder: LLMBuilder,
        _: &providers::Providers) -> Result<Box< dyn llm::LLMProvider>, providers::ProviderError> {
            let builder = llmbuilder.backend(llm::builder::LLMBackend::OpenRouter)
                .api_key(&self.api_key);
            
        Ok(builder.build()?)
    }
}
