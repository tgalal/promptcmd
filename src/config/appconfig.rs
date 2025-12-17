use serde::Deserialize;
use serde::Serialize;
use toml;
use toml::de::Error as TomlError;
use thiserror::Error;
use crate::config::providers;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub providers: providers::Providers,
}


#[derive(Error, Debug)]
pub enum AppConfigError {
    #[error("Error Reading Config file")]
    ReadConfigError(#[from] TomlError)
}

impl TryFrom<&String> for AppConfig {
    type Error = AppConfigError;

    fn try_from(contents: &String) -> Result<Self, Self::Error> {
        Ok(toml::from_str::<AppConfig>(contents)?)
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
        assert_eq!(config.providers.anthropic.config.api_key, "test-key-123");
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
    fn test_parse_config_with_named_providers() {
        let toml_content = r#"
            default_model = "claude-3-5-sonnet-20241022"
            editor = "vim"

            [providers.anthropic]
            api_key = "default-key"

            [providers.custom_claude]
            api_key = "custom-key"
            temperature = 0.5
            max_tokens = 4000
        "#;

        let config = AppConfig::try_from(&toml_content.to_string());
        assert!(config.is_ok(), "Should parse config with named providers");
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
        // This test only validates configuration parsing and provider resolution logic
        // No HTTP calls are made - we're just testing the resolve() method
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
}
