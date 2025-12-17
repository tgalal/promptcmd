pub mod anthropic;
pub mod ollama;
pub mod openai;
pub mod google;

use llm::{builder::LLMBuilder, error::LLMError, LLMProvider};
use serde::{Serialize, Deserialize};

const DEFAULT_MAX_TOKENS:u32 = 1000;
const DEFAULT_STREAM:bool = false;
const DEFAULT_TEMPERATURE:f32 = 0.7;

use thiserror::Error;

use crate::config::providers::{anthropic::{AnthropicProviders}, google::{GoogleProviders},
    ollama::{OllamaProviders}, openai::{OpenAIProviders}};

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Error Creating LLM Backend")]
    CreateLLMClientError(#[from] LLMError),

    #[error("Missing required configuration key {name}")]
    MissingRequiredConfiguration {
        name: String,
    },
}

pub trait ToLLMProvider {
    fn llm_provider(&self, llmbuilder: LLMBuilder, providers: &Providers) -> Result<Box< dyn LLMProvider>, ProviderError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Providers {

    pub temperature: Option<f32>,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,

    #[serde(default)]
    pub ollama: ollama::OllamaProviders,

    #[serde(default)]
    pub openai: openai::OpenAIProviders,
 
    #[serde(default)]
    pub anthropic: anthropic::AnthropicProviders,
 
    #[serde(default)]
    pub google: google::GoogleProviders
}

pub enum ProviderVariant<'a> {
    Ollama(&'a ollama::OllamaConfig),
    OpenAi(&'a openai::OpenAIConfig),
    Anthropic(&'a anthropic::AnthropicConfig),
    Google(&'a google::GoogleConfig),
    None
}

impl Default for Providers {
    fn default() -> Self {
        Providers {
            temperature: Some(DEFAULT_TEMPERATURE),
            stream: Some(DEFAULT_STREAM),
            max_tokens: Some(DEFAULT_MAX_TOKENS),
            ollama: OllamaProviders::default(),
            openai: OpenAIProviders::default(),
            anthropic: AnthropicProviders::default(),
            google: GoogleProviders::default()
        }
    }
}

impl Providers {

    pub fn resolve<'a>(&'a self, name: &str) -> ProviderVariant<'a> {
        // Direct, top level search
        if name == "ollama" {
            return ProviderVariant::Ollama(&self.ollama.config)
        } else if name == "anthropic" {
            return ProviderVariant::Anthropic(&self.anthropic.config)
        } else if name == "google" {
            return ProviderVariant::Google(&self.google.config)
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

        if let Some(conf) = self.google.named.get(name) {
            return ProviderVariant::Google(conf);
        }

        ProviderVariant::None
    }
}
