use std::collections::HashMap;

use serde::Deserialize;
use toml;
use toml::de::Error as TomlError;
use thiserror::Error;

use crate::config::providers;
use crate::config::resolver;


#[derive(Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub create: Create,
    #[serde(default)]
    pub import: Import,
    #[serde(default)]
    pub providers: Providers,
    #[serde(default)]
    pub groups: HashMap<String, GroupConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Create {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct Import {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct GlobalProviderProperties {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub model: Option<String>,
    pub system: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Providers {
    pub default: Option<String>,
    #[serde(flatten)]
    pub globals: GlobalProviderProperties,

    #[serde(default)]
    pub ollama: providers::ollama::Providers,
    #[serde(default)]
    pub openai: providers::openai::Providers,
    #[serde(default)]
    pub anthropic: providers::anthropic::Providers,
    #[serde(default)]
    pub google: providers::google::Providers,
    #[serde(default)]
    pub openrouter: providers::openrouter::Providers,
}

#[derive(Debug, Deserialize, Default)]
pub struct GroupConfig {
    pub providers: Vec<GroupProviderConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GroupProviderConfig{
    Short(String),
    Long(LongGroupProviderConfig)
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct LongGroupProviderConfig {
    pub name: String,
    pub weight: Option<u32>,
}

impl GroupProviderConfig {
    pub fn to_long(&self) -> LongGroupProviderConfig {
        match self {
            Self::Short(name) => LongGroupProviderConfig {
                name: name.to_string(),
                weight: Some(1),
            },
            Self::Long(config) => config.clone()
        }
    }
}


#[derive(Error, Debug)]
pub enum AppConfigError {
    #[error("Error Reading Config file")]
    ReadConfigError(#[from] TomlError)
}

impl TryFrom<&str> for AppConfig {
    type Error = AppConfigError;

    fn try_from(contents: &str) -> Result<Self, Self::Error> {
        Ok(toml::from_str::<AppConfig>(contents)?)
    }
}

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Error parsing model string:{0}")]
    ParseNameError(String),
    #[error("Could not resolve model or group: {0}")]
    ResolveFailed(#[from] resolver::error::ResolveError),
    #[error("No default_model configured for provider: {0}")]
    NoDefaultModelConfigured(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_config() {
        let toml_content = r#"
            [providers.anthropic]
            api_key = "test-key-123"
        "#;

        let config = AppConfig::try_from(toml_content);
        assert!(config.is_ok(), "Should parse valid TOML");
    }

    #[test]
    fn test_parse_config_with_multiple_providers() {
        let toml_content = r#"
            [providers.openai]
            api_key = "api_key"

            [providers.anthropic]
            api_key = "anthropic-key"

            [providers.ollama]
            endpoint = "http://localhost:11434"
        "#;

        let config = AppConfig::try_from(toml_content);
        assert!(config.is_ok(), "Should parse config with multiple providers");
    }

    #[test]
    fn test_parse_config_with_global_settings() {
        let toml_content = r#"
            [providers]
            temperature = 0.8
            max_tokens = 2000
            system = "You are a helpful assistant"

            [providers.anthropic]
            api_key = "test-key"
        "#;

        let config = AppConfig::try_from(toml_content);
        assert!(config.is_ok(), "Should parse config with global provider settings");
    }

    #[test]
    fn test_parse_invalid_toml() {
        let invalid_toml = r#"
            default_model = "test"
            editor = "vim"
            this is not valid toml
        "#;

        let config = AppConfig::try_from(invalid_toml);
        assert!(config.is_err(), "Should fail on invalid TOML");
    }

    #[test]
    fn test_empty_providers_section() {
        let toml_content = r#"
            [providers]
        "#;

        let config = AppConfig::try_from(toml_content);
        assert!(config.is_ok(), "Should parse config with empty providers section");
    }

    #[test]
    fn test_groups() {
        let toml_content = r#"
[groups.group1]
providers = [
    "openai", "anthropic"
]

[groups.group2]
providers = [
    { name = "openai", weight = 1 },
    { name = "ollama", weight = 2 },
]
"#;
        let config = AppConfig::try_from(toml_content).unwrap();
        let groups = config.groups;

        println!("{:?}", groups);
    }

}
