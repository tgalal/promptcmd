use std::collections::HashMap;

use serde::Deserialize;
use toml;
use toml::de::Error as TomlError;
use thiserror::Error;

use crate::config::providers;
use crate::config::resolver;
use crate::dotprompt::ParsedFrontmatter;


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
    #[serde(default)]
    pub ssh: Vec<Ssh>,
}

impl AppConfig {
    pub fn find_ssh_best_match(&self, host: &str, user: Option<&str>) -> Option<&Ssh> {
        self.ssh
            .iter()
            .filter_map(|remote| {
                let remote_user = remote.user.as_ref().map(|s| s.as_str());
                // Check if host matches (either equal or None)
                let host_matches = remote.host.as_deref() == Some(host) || remote.host.is_none();
                // Check if user matches (either equal or None)
                let user_matches = remote_user == user || remote_user.is_none() || user.is_none();

                // Only consider remotes that match
                if host_matches && user_matches {
                    // Calculate match score: 2 points for exact match, 1 point for None
                    let score =
                        (if remote.host.as_deref() == Some(host) { 2 } else { 1 }) +
                        (if remote_user == user { 2 } else { 1 });
                    Some((remote, score))
                } else {
                    None
                }
            })
            .max_by_key(|(_, score)| *score)
            .map(|(remote, _)| remote)
    }
}

#[derive(Debug, Deserialize, Default, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum BashMethod {
    Posix,
    Rc,
    #[default]
    Exports
}

#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ShellOptions {
    #[default]
    Auto,
    Bash,
    Zsh,
    Sh,
    Ash,
    Dash,
    Fish
}

#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChannelOptions {
    #[default]
    Auto,
    Nc,
    Socat,
    BashTcp,
    Fifo,
    FifoSingle,
}

#[derive(Debug, Deserialize)]
pub struct Ssh {
    pub host: Option<String>,
    pub user: Option<String>,
    #[serde(default)]
    pub bash_method: BashMethod,
    #[serde(default)]
    pub shell: ShellOptions,
    #[serde(default)]
    pub channel: ChannelOptions,
    #[serde(default)]
    pub remote_socket: RemoteSocket,
    #[serde(default)]
    pub remote_ports: PortSettings,
    #[serde(default)]
    pub local_ports: PortSettings,
    #[serde(default = "default_motd")]
    pub motd: bool
}

#[derive(Debug, Deserialize)]
pub struct RemoteSocket {
    #[serde(default = "default_socket_path")]
    pub path: String,
    #[serde(default = "default_socket_random")]
    pub random: bool
}

#[derive(Debug, Deserialize)]
pub struct PortSettings {
    #[serde(default = "default_port_start")]
    pub start: u32,
    #[serde(default = "default_port_end")]
    pub end: u32,
}

fn default_port_start() -> u32 { 49152 }
fn default_port_end() -> u32 { 65535 }
fn default_socket_path() -> String { "/tmp/".to_string() }
fn default_socket_random() -> bool { true }
fn default_motd() -> bool { false }

impl Default for PortSettings {
   fn default() -> Self {
        Self {
            start: default_port_start(),
            end: default_port_end(),
        }
    }
}
impl Default for RemoteSocket {
    fn default() -> Self {
        Self {
            random: default_socket_random(),
            path: default_socket_path()
        }
    }
}
impl Default for Ssh {
    fn default() -> Self {
        Self {
            host: None,
            user: None,
            bash_method: BashMethod::default(),
            shell: ShellOptions::Auto,
            channel: ChannelOptions::Auto,
            remote_socket: RemoteSocket::default(),
            remote_ports: PortSettings::default(),
            local_ports: PortSettings::default(),
            motd: true
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct Create {
    #[serde(default)]
    pub no_enable: bool,
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

#[derive(Debug, Deserialize, Default, Clone)]
pub struct GlobalProviderProperties {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub model: Option<String>,
    pub system: Option<String>,
    pub cache_ttl: Option<u32>,
    pub stream: Option<bool>,
}

impl From<&ParsedFrontmatter> for GlobalProviderProperties {
    fn from(fm: &ParsedFrontmatter) -> Self {
        GlobalProviderProperties {
            temperature: fm.config.as_ref().and_then(|config| config.temperature),
            max_tokens: fm.config.as_ref().and_then(|config| config.max_output_tokens),
            model:  fm.model.clone(),
            system: None,
            cache_ttl: fm.config.as_ref().and_then(|config| config.cache_ttl),
            stream: None
        }
    }
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
    #[error("Config file has invalid format: {0}")]
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
    use rstest::rstest;

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

    #[rstest]
    #[case(
        r#"
[[ssh]]
host = "host1"
shell = "sh"
user = "user1"

"#, "host1", Some("user1"), Some(ShellOptions::Sh))]

    #[case(
        r#"
[[ssh]]
host = "host1"
shell = "sh"
user = "user1"

"#, "host5", None, None)]

    #[case(
        r#"
[[ssh]]
host = "host1"
shell = "sh"
user = "user1"

"#, "host1", None, Some(ShellOptions::Sh))]

    #[case(
        r#"
[[ssh]]
host = "host1"
shell = "bash"

[[ssh]]
host = "host1"
user = "user1"
shell = "zsh"

"#, "host1", None, Some(ShellOptions::Bash))]

    #[case(
        r#"
[[ssh]]
host = "host1"
shell = "bash"

[[ssh]]
host = "host1"
user = "user1"
shell = "zsh"

"#, "host1", Some("user1"), Some(ShellOptions::Zsh))]

    #[case(
        r#"
[[ssh]]
host = "host1"
shell = "bash"

[[ssh]]
host = "host1"
user = "user1"
shell = "zsh"

"#, "host1", Some("user2"), Some(ShellOptions::Bash))]

    #[case(
        r#"
[[ssh]]
host = "host1"
shell = "bash"

[[ssh]]
host = "host1"
user = "user1"
shell = "zsh"

"#, "", Some("user1"), None)]
    #[case(
        r#"
[[ssh]]
host = "host1"
shell = "bash"

[[ssh]]
host = "host1"
user = "user1"
shell = "zsh"

"#, "", Some(""), None)]
    fn test_ssh(#[case] toml_content: &str,
        #[case] host: &str,
        #[case] user: Option<&str>,
        #[case] shellopt: Option<ShellOptions>) {
        let config = AppConfig::try_from(toml_content).unwrap();

        let rconf = config.find_ssh_best_match(host, user);

        if let Some(shellopt) = shellopt {
            assert_eq!(shellopt, rconf.unwrap().shell);
        } else {
            assert!(rconf.is_none());
        }
    }
}
