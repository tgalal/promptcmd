use serde::Deserialize;
use serde::Serialize;
use toml;
use toml::de::Error as TomlError;
use thiserror::Error;
use crate::config::providers;

// const DEFAULT_MAX_TOKENS:u32 = 1000;
// const DEFAULT_STREAM:bool = false;
// const DEFAULT_TEMPERATURE:f32 = 0.7;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    default_model: String,
    editor: String,
    pub providers: providers::Providers,
}

// #[derive(Debug, Serialize, Deserialize)]
// struct Providers {
// 
//     temperature: Option<f32>,
//     system: Option<String>,
//     stream: Option<bool>,
//     max_tokens: Option<u32>,
// 
//     #[serde(default)]
//     ollama: OllamaProviders,
// 
//     #[serde(default)]
//     openai: OpenAIProviders,
//  
//     #[serde(default)]
//     anthropic: AnthropicProviders
// }




#[derive(Error, Debug)]
pub enum AppConfigError {
    #[error("Error Reading Config file")]
    ReadConfigError(#[from] TomlError)
}

// pub enum ProviderVariant<'a> {
//     Ollama(&'a providers::ollama::OllamaConfig),
//     OpenAi(&'a providers::openai::OpenAIConfig),
//     Anthropic(&'a providers::anthropic::AnthropicConfig),
//     None
// }
// 
// impl AppConfig {
//     pub fn resolve_provider<'a>(&'a self, name: &str) -> ProviderVariant<'a> {
//         // Direct, top level search
//         if name == "ollama" {
//             return ProviderVariant::Ollama(&self.providers.ollama.config)
//         } else if name == "anthropic" {
//             return ProviderVariant::Anthropic(&self.providers.anthropic.config)
//         }
// 
//         // search throughout all providers
//         if let Some(conf) = self.providers.ollama.named.get(name) {
//             return ProviderVariant::Ollama(conf);
//         }
// 
//         if let Some(conf) = self.providers.openai.named.get(name) {
//             return ProviderVariant::OpenAi(conf);
//         }
// 
//         if let Some(conf) = self.providers.anthropic.named.get(name) {
//             return ProviderVariant::Anthropic(conf);
//         }
// 
//         ProviderVariant::None
//     }
// }

// impl OllamaConfig {
//     pub fn temperature(&self, appconfig: &AppConfig) -> f32 {
//         self.temperature.or(
//             appconfig.providers.ollama.config.temperature.or(
//                 appconfig.providers.temperature
//             )
//         ).unwrap_or(DEFAULT_TEMPERATURE)
//     }
// 
//     pub fn stream(&self, appconfig: &AppConfig) -> bool {
//         self.stream.or(
//             appconfig.providers.ollama.config.stream.or(
//                 appconfig.providers.stream
//             )
//         ).unwrap_or(DEFAULT_STREAM)
//     }
// 
//     pub fn max_tokens(&self, appconfig: &AppConfig) -> u32 {
//         self.max_tokens.or(
//             appconfig.providers.ollama.config.max_tokens.or(
//                 appconfig.providers.max_tokens
//             )
//         ).unwrap_or(DEFAULT_MAX_TOKENS)
//     }
// }

impl TryFrom<&String> for AppConfig {
    type Error = AppConfigError;

    fn try_from(contents: &String) -> Result<Self, Self::Error> { 
        Ok(toml::from_str::<AppConfig>(contents)?)
    }
}
