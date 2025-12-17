use llm::builder::LLMBuilder;
use serde::{Serialize, Deserialize};
use crate::config::providers::{self, ToLLMProvider};

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
    endpoint: String
}

impl ToLLMProvider for OpenAIConfig {
    fn llm_provider(&self,
        llmbuilder: LLMBuilder,
        _: &providers::Providers) -> Result<Box< dyn llm::LLMProvider>, providers::ProviderError> {
            let builder = llmbuilder.backend(llm::builder::LLMBackend::OpenAI)
                .base_url(&self.endpoint);
            
        Ok(builder.build()?)
    }
}
