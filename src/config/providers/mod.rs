pub mod anthropic;
pub mod ollama;
pub mod openai;

use serde::{Serialize, Deserialize};

const DEFAULT_MAX_TOKENS:u32 = 1000;
const DEFAULT_STREAM:bool = false;
const DEFAULT_TEMPERATURE:f32 = 0.7;


#[derive(Debug, Serialize, Deserialize)]
pub struct Providers {

    temperature: Option<f32>,
    system: Option<String>,
    stream: Option<bool>,
    max_tokens: Option<u32>,

    #[serde(default)]
    ollama: ollama::OllamaProviders,

    #[serde(default)]
    openai: openai::OpenAIProviders,
 
    #[serde(default)]
    anthropic: anthropic::AnthropicProviders
}

pub enum ProviderVariant<'a> {
    Ollama(&'a ollama::OllamaConfig),
    OpenAi(&'a openai::OpenAIConfig),
    Anthropic(&'a anthropic::AnthropicConfig),
    None
}

impl Providers {
    pub fn resolve<'a>(&'a self, name: &str) -> ProviderVariant<'a> {
        // Direct, top level search
        if name == "ollama" {
            return ProviderVariant::Ollama(&self.ollama.config)
        } else if name == "anthropic" {
            return ProviderVariant::Anthropic(&self.anthropic.config)
        }

        // search throughout all providers
        if let Some(conf) = self.ollama.named.get(name) {
            return ProviderVariant::Ollama(conf);
        }

        if let Some(conf) = self.openai.named.get(name) {
            return ProviderVariant::OpenAi(conf);
        }

        if let Some(conf) = self.anthropic.named.get(name) {
            return ProviderVariant::Anthropic(conf);
        }

        ProviderVariant::None
    }
}
