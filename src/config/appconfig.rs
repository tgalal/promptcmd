use serde::Deserialize;
use serde::Serialize;
use toml;
use toml::de::Error as TomlError;
use thiserror::Error;

const DEFAULT_MAX_TOKENS:u32 = 1000;
const DEFAULT_STREAM:bool = false;
const DEFAULT_TEMPERATURE:f32 = 0.7;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    default_model: String,
    editor: String,
    providers: Providers,
}

#[derive(Debug, Serialize, Deserialize)]
struct Providers {

    temperature: Option<f32>,
    system: Option<String>,
    stream: Option<bool>,
    max_tokens: Option<u32>,

    #[serde(default)]
    ollama: OllamaProviders,

    #[serde(default)]
    openai: OpenAIProviders
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct OllamaProviders {
    #[serde(flatten)]
    config: OllamaConfig,
    
    #[serde(flatten)]
    named: std::collections::HashMap<String, OllamaConfig>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct OllamaConfig {
    temperature: Option<f32>,
    system: Option<String>,
    stream: Option<bool>,
    max_tokens: Option<u32>,

    #[serde(default)]
    pub endpoint: String
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct OpenAIProviders {

    #[serde(flatten)]
    config: OpenAIConfig,
    
    #[serde(flatten)]
    named: std::collections::HashMap<String, OpenAIConfig>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct OpenAIConfig {
    temperature: Option<f32>,
    system: Option<String>,
    stream: Option<bool>,
    max_tokens: Option<u32>,

    #[serde(default)]
    pub endpoint: String
}

#[derive(Error, Debug)]
pub enum AppConfigError {
    #[error("Error Reading Config file")]
    ReadConfigError(#[from] TomlError)
}

pub enum ProviderVariant<'a> {
    Ollama(&'a OllamaConfig),
    OpenAi(&'a OpenAIConfig),
    None
}

impl AppConfig {
    pub fn resolve_provider<'a>(&'a self, name: &str) -> ProviderVariant<'a> {
        // Direct, top level search
        if name == "ollama" {
            return ProviderVariant::Ollama(&self.providers.ollama.config)
        }

        // search throughout all providers
        if let Some(conf) = self.providers.ollama.named.get(name) {
            return ProviderVariant::Ollama(conf);
        }

        if let Some(conf) = self.providers.openai.named.get(name) {
            return ProviderVariant::OpenAi(conf);
        }

        ProviderVariant::None
    }
}

impl OllamaConfig {
    pub fn temperature(&self, appconfig: &AppConfig) -> f32 {
        self.temperature.or(
            appconfig.providers.ollama.config.temperature.or(
                appconfig.providers.temperature
            )
        ).unwrap_or(DEFAULT_TEMPERATURE)
    }

    pub fn stream(&self, appconfig: &AppConfig) -> bool {
        self.stream.or(
            appconfig.providers.ollama.config.stream.or(
                appconfig.providers.stream
            )
        ).unwrap_or(DEFAULT_STREAM)
    }

    pub fn max_tokens(&self, appconfig: &AppConfig) -> u32 {
        self.max_tokens.or(
            appconfig.providers.ollama.config.max_tokens.or(
                appconfig.providers.max_tokens
            )
        ).unwrap_or(DEFAULT_MAX_TOKENS)
    }
}

impl TryFrom<&String> for AppConfig {
    type Error = AppConfigError;

    fn try_from(contents: &String) -> Result<Self, Self::Error> { 
        Ok(toml::from_str::<AppConfig>(contents)?)
    }
}
