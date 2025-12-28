use serde::Deserialize;
use toml;
use toml::de::Error as TomlError;
use thiserror::Error;
use crate::config::providers;
use log::debug;

const PROVIDER_NAME_OLLAMA : &str = "ollama";
const PROVIDER_NAME_OPENAI : &str = "openai";
const PROVIDER_NAME_GOOGLE : &str = "google";
const PROVIDER_NAME_OPENROUTER : &str = "openrouter";
const PROVIDER_NAME_ANTHROPIC : &str = "anthropic";

#[derive(Debug, Deserialize, Default)]
pub struct GroupConfig {
    pub name: String,
    pub providers: Vec<GroupProviderConfig>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct LongGroupProviderConfig {
    pub name: String,
    pub weight: Option<u32>,
    pub fallback: Option<bool>
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GroupProviderConfig{
    Short(String),
    Long(LongGroupProviderConfig)
}

impl GroupProviderConfig {
    pub fn to_long(&self) -> LongGroupProviderConfig {
        match self {
            Self::Short(name) => LongGroupProviderConfig {
                name: name.to_string(),
                weight: Some(1 as u32) ,
                fallback: Some(true)
            },
            Self::Long(config) => config.clone()
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct AppConfig {
    pub providers: providers::Providers,
    pub group: Option<Vec<GroupConfig>>
}

#[derive(Error, Debug)]
pub enum AppConfigError {
    #[error("Error Reading Config file")]
    ReadConfigError(#[from] TomlError)
}

#[derive(Error, Debug)]
pub enum ResolveGroupError {
    #[error("No such group: {0}")]
    NoSuchGroup(String)
}

impl TryFrom<&String> for AppConfig {
    type Error = AppConfigError;

    fn try_from(contents: &String) -> Result<Self, Self::Error> {
        Ok(toml::from_str::<AppConfig>(contents)?)
    }
}

pub struct ResolvedModelName {
    pub provider: String,
    pub model: String,
    pub weight: u32,
    pub fallback: bool,
}

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Error parsing model string:{0}")]
    ParseNameError(String),
    #[error("Could not resolve model or group: {0}")]
    ResolveFailed(String),
    #[error("No default_model configured for provider: {0}")]
    NoDefaultModelConfigured(String),
}

impl AppConfig {
    pub fn resolve_group(&self, name: &str) -> Result<&GroupConfig, ResolveGroupError> {
        if let Some(groups) = &self.group {
            for group in groups {
                if group.name == name {
                    return Ok(group)
                }
            }
        }
        Err(ResolveGroupError::NoSuchGroup(name.into()))
    }

    pub fn resolve_model_name(&self, name: &str, resolve_group: bool) -> Result<Vec<ResolvedModelName>, ModelError> {
        debug!("Resolving {}, resolve_group:{} ", name, resolve_group);
        if name.contains("/") {
            debug!("{} has provider/model format, no resolution needed", name);
            // explicit model/provider format, no resolution needed
            let (provider, model) = name.split_once("/")
                .ok_or(ModelError::ParseNameError(name.to_string()))?;

            Ok(vec![ResolvedModelName {
                provider: provider.to_string(),
                model: model.to_string(),
                weight: 1,
                fallback: false
            }])
        } else {
            debug!("{name} is not in provider/model format, it's either a provider or a group name");
            // Either group name or short provider name
            let (provider_name, model_name ) = match self.providers.resolve(name) {
                providers::ProviderVariant::Anthropic(conf) => (PROVIDER_NAME_ANTHROPIC, conf.default_model(&self.providers)),
                providers::ProviderVariant::Google(conf) => (PROVIDER_NAME_GOOGLE, conf.default_model(&self.providers)),
                providers::ProviderVariant::Ollama(conf) => (PROVIDER_NAME_OLLAMA, conf.default_model(&self.providers)),
                providers::ProviderVariant::OpenAi(conf) => (PROVIDER_NAME_OPENAI, conf.default_model(&self.providers)),
                providers::ProviderVariant::OpenRouter(conf) => (PROVIDER_NAME_OPENROUTER, conf.default_model(&self.providers)),
                _ => ("", None)
            };

            if let Some(model_name) = model_name {
                debug!("{name} detected to be a provider name for {provider_name}");
                // It's a provider name
                Ok(vec![ResolvedModelName {
                    provider: provider_name.to_string(),
                    model: model_name,
                    weight: 1,
                    fallback: false
                }])
            } else if !provider_name.is_empty() {
                Err(ModelError::NoDefaultModelConfigured(provider_name.to_string()))
            } else if resolve_group {
                debug!("Resolving {name} as a group name");
                // group
                let group_config = self.resolve_group(name).map_err(
                    |_| ModelError::ResolveFailed(name.to_string())
                )?;

                debug!("Found a matching group config for {name}");

                let mut result: Vec<ResolvedModelName> = Vec::new();
                for ele in &group_config.providers {
                    let long_config = ele.to_long();

                    let mut resolved_modelname = self.resolve_model_name(&long_config.name, false)?;
                    if let Some(weight) = long_config.weight {
                        resolved_modelname[0].weight = weight;
                    }
                    result.append(&mut resolved_modelname);
                }
                debug!("Resolved {name} as a group");
                Ok(result)
            } else {
                debug!("Will not attempt to resolve {name} as group name, resolution failed");
                Err(ModelError::ResolveFailed(name.to_string()))
            }
        }
    }
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

        let config = AppConfig::try_from(&toml_content.to_string());
        assert!(config.is_ok(), "Should parse valid TOML");

        let config = config.unwrap();
        assert_eq!(config.providers.anthropic.config.api_key(&config.providers).unwrap(), "test-key-123");
    }

    #[test]
    fn test_parse_config_with_multiple_providers() {
        let toml_content = r#"
            default_model = "gpt-4"
            editor = "nano"

            [providers.openai]
            endpoint = "https://api.openai.com/v1"

            [providers.anthropic]
            api_key = "anthropic-key"

            [providers.ollama]
            endpoint = "http://localhost:11434"
        "#;

        let config = AppConfig::try_from(&toml_content.to_string());
        assert!(config.is_ok(), "Should parse config with multiple providers");
    }

    #[test]
    fn test_parse_config_with_global_settings() {
        let toml_content = r#"
            default_model = "claude-3-5-sonnet-20241022"
            editor = "vim"

            [providers]
            temperature = 0.8
            max_tokens = 2000
            stream = true
            system = "You are a helpful assistant"

            [providers.anthropic]
            api_key = "test-key"
        "#;

        let config = AppConfig::try_from(&toml_content.to_string());
        assert!(config.is_ok(), "Should parse config with global provider settings");
    }

    #[test]
    fn test_parse_invalid_toml() {
        let invalid_toml = r#"
            default_model = "test"
            editor = "vim"
            this is not valid toml
        "#;

        let config = AppConfig::try_from(&invalid_toml.to_string());
        assert!(config.is_err(), "Should fail on invalid TOML");
    }

    #[test]
    fn test_parse_missing_required_fields() {
        let incomplete_toml = r#"
            editor = "vim"
        "#;

        let config = AppConfig::try_from(&incomplete_toml.to_string());
        assert!(config.is_err(), "Should fail when required fields are missing");
    }

    #[test]
    fn test_provider_resolution() {
        let toml_content = r#"
            default_model = "claude-3-5-sonnet-20241022"
            editor = "vim"

            [providers.anthropic]
            api_key = "anthropic-key"

            [providers.ollama]
            endpoint = "http://localhost:11434"

            [providers.ollama.custom_ollama]
            endpoint = "http://custom:11434"

            [providers.anthropic.custom_claude]
            api_key = "custom-key"
        "#;

        let config = AppConfig::try_from(&toml_content.to_string()).unwrap();

        // Test direct provider resolution - only checks the resolve() method logic
        let anthropic = config.providers.resolve("anthropic");
        assert!(matches!(anthropic, providers::ProviderVariant::Anthropic(_)));

        let ollama = config.providers.resolve("ollama");
        assert!(matches!(ollama, providers::ProviderVariant::Ollama(_)));

        // Test named provider resolution
        let custom_ollama = config.providers.resolve("custom_ollama");
        assert!(matches!(custom_ollama, providers::ProviderVariant::Ollama(_)));

        let custom_claude = config.providers.resolve("custom_claude");
        assert!(matches!(custom_claude, providers::ProviderVariant::Anthropic(_)));

        // Test non-existent provider
        let none = config.providers.resolve("nonexistent");
        assert!(matches!(none, providers::ProviderVariant::None));
    }

    #[test]
    fn test_empty_providers_section() {
        let toml_content = r#"
            default_model = "test"
            editor = "vim"

            [providers]
        "#;

        let config = AppConfig::try_from(&toml_content.to_string());
        assert!(config.is_ok(), "Should parse config with empty providers section");
    }

    #[test]
    fn test_groups() {
        let toml_content = r#"
[providers]

[[group]]
name = "group1"
providers = [
    "openai", "anthropic"
]

[[group]]
name = "group2"
providers = [
    { name = "openai", weight = 1 },
    { name = "ollama", weight = 2 },
]
"#;
        let config = AppConfig::try_from(&toml_content.to_string()).unwrap();
        let groups = config.group.unwrap();

        println!("{:?}", groups);

    }

    #[test]
    fn test_resolve_model() {
        let toml_content = r#"
[providers]

[providers.ollama]
system = "Ollama system prompt"
endpoint="http://ollama_endpoint:11341"
default_model = "abc"

[providers.anthropic]
api_key = "sk-ant-xyz"
default_model = "claude"

[providers.ollama.xyz]
endpoint="http://ollama_endpoint2:11341"

[[group]]
name = "group1"
providers = [
    "openai", "anthropic"
]

[[group]]
name = "group2"
providers = [
    { name = "openai", weight = 1 },
    { name = "ollama", weight = 2 },
]
"#;
        let config = AppConfig::try_from(&toml_content.to_string()).unwrap();

        let mut res = config.resolve_model_name("anthropic/sonnet", true).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].provider, "anthropic");
        assert_eq!(res[0].model, "sonnet");

        // test default model
        res = config.resolve_model_name("anthropic", true).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].provider, "anthropic");
        assert_eq!(res[0].model, "claude");

        // test default model
        res = config.resolve_model_name("xyz", true).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].provider, "ollama");
        assert_eq!(res[0].model, "abc");
    }

    #[test]
    fn test_resolve_groups() {
        let toml_content = r#"
[providers]

[providers.ollama]
system = "Ollama system prompt"
endpoint="http://ollama_endpoint:11341"
default_model = "abc"

[providers.anthropic]
api_key = "sk-ant-xyz"
default_model = "claude"

[providers.ollama.xyz]
endpoint="http://ollama_endpoint2:11341"

[[group]]
name = "group1"
providers = [
    "openai", "anthropic"
]

[[group]]
name = "group2"
providers = [
    { name = "anthropic", weight = 1 },
    { name = "ollama", weight = 2 },
]

[[group]]
name = "group3"
providers = [
    { name = "anthropic", weight = 1 },
    { name = "xyz", weight = 2 },
    { name = "anthropic/sonnet", weight = 2 },
]
"#;
        let config = AppConfig::try_from(&toml_content.to_string()).unwrap();

        let mut res = config.resolve_model_name("group2", true).unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].provider, "anthropic");
        assert_eq!(res[0].model, "claude");
        assert_eq!(res[0].weight, 1);
        assert_eq!(res[1].provider, "ollama");
        assert_eq!(res[1].model, "abc");
        assert_eq!(res[1].weight, 2);

        // group1 needs openai, but there is not one
        assert!(config.resolve_model_name("group1", true).is_err());

        res = config.resolve_model_name("group3", true).unwrap();
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].provider, "anthropic");
        assert_eq!(res[0].model, "claude");
        assert_eq!(res[0].weight, 1);
        assert_eq!(res[1].provider, "ollama");
        assert_eq!(res[1].model, "abc");
        assert_eq!(res[1].weight, 2);
        assert_eq!(res[2].provider, "anthropic");
        assert_eq!(res[2].model, "sonnet");
        assert_eq!(res[2].weight, 2);

    }
}
