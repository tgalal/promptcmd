pub mod base;
// mod base2;
pub mod variant;
pub mod group;
pub mod resolved;
use indenter::indented;
use std::fmt::{self, Write, Display};

use thiserror::Error;
use log::debug;
use crate::{config::{appconfig::{AppConfig, GroupProviderConfig, LongGroupProviderConfig}, providers::{
    anthropic::AnthropicConfig, ollama::OllamaConfig, openai::OpenAIConfig
    // google::GoogleConfig,
    // openai::OpenAIConfig,
    // openrouter::OpenRouterConfig
}}, resolver::{base::Base, group::{Group, GroupMember}, variant::Variant}};


#[derive(Debug)]
pub enum ResolvedConfig {
    Base(base::Base),
    Variant(variant::Variant),
    Group(group::Group)
}

impl fmt::Display for ResolvedConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolvedConfig::Base(base) => {
                write!(f, "{base}")
            },
            ResolvedConfig::Variant(variant) => {
                write!(f, "{variant}")

            },
            ResolvedConfig::Group(group) => {
                write!(f, "{group}")
            },
        }
    }
}


pub enum BaseProviderConfigSource<'a> {
    Ollama(&'a OllamaConfig),
    Anthropic(&'a AnthropicConfig),
    OpenAI(&'a OpenAIConfig),
    // OpenRouter(&'a OpenRouterConfig),
    // Google(&'a GoogleConfig),
}

pub enum VariantProviderConfigSource<'a> {
    Ollama(&'a OllamaConfig, &'a OllamaConfig),
    Anthropic(&'a AnthropicConfig, &'a AnthropicConfig),
    OpenAI(&'a OpenAIConfig, &'a OpenAIConfig),
    // OpenRouter(&'a OpenRouterConfig),
    // Google(&'a GoogleConfig),
}

// struct ProviderConfigSource<'a, T> {
//     source: &'a T
// }

#[derive(Debug)]
pub enum ResolvedProviderConfig {
    Ollama(resolved::ollama::ResolvedProviderConfig),
    Anthropic(resolved::anthropic::ResolvedProviderConfig),
    OpenAI(resolved::openai::ResolvedProviderConfig),
    // OpenRouter(ResolvedAnthropicConfig),
    // Google(ResolvedGoogleConfig),
}

impl fmt::Display for ResolvedProviderConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Provider: ")?;
        match self {
            Self::Ollama(conf) => {
                writeln!(f, "Ollama")?;
                writeln!(f, "Configuration:")?;
                write!(indented(f).with_str("  "), "{conf}")?
            },
            Self::Anthropic(conf) => {
                writeln!(f, "Anthropic")?;
                writeln!(f, "Configuration:")?;
                write!(indented(f).with_str("  "), "{conf}")?
            },
            Self::OpenAI(conf) => {
                writeln!(f, "OpenAI")?;
                writeln!(f, "Configuration:")?;
                write!(indented(f).with_str("  "), "{conf}")?
            }
        };

        Ok(())
    }
}


#[derive(Clone, Debug)]
pub struct ResolvedProperty<T> {
    pub source: ResolvedPropertySource,
    pub value: T
}

impl<T:Display> fmt::Display for ResolvedProperty<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}


#[derive(Clone, Debug)]
pub enum ResolvedPropertySource {
    Group(String, String),
    Variant(String),
    Base(String),
    Env(String),
    Default,
    Dotprompt(String),
    Input(String),
    Other(String)
}

impl fmt::Display for ResolvedPropertySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Group(group_name, value) => write!(f, "Group({group_name},{value})"),
            Self::Variant(name) => write!(f, "Variant({name})"),
            Self::Base(name) => write!(f, "Base({name})"),
            Self::Env(name) => write!(f, "Env({name})"),
            Self::Default => write!(f, "Default"),
            Self::Dotprompt(name) => write!(f, "Frontmatter({name})"),
            Self::Input(name) => write!(f, "Input({name})"),
            Self::Other(name) => write!(f, "Other({name})")
        }
    }
}

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("'{0}' not found")]
    NotFound(String),
    #[error("'{0}' not supported")]
    NotSupported(String),
    #[error("No groups defined in config")]
    NoGroups,
    #[error("Group '{0}' references '{1}' which is not found")]
    GroupMemberNotFound(String, String),
    #[error("Group '{0}' failed to load member: {1}")]
    GroupMemberError(String, Box<ResolveError>),
}

fn resolve_base(appconfig: &AppConfig, base_name: &str,
    model_source: Option<ResolvedPropertySource>) -> Result<Base, ResolveError> {

    debug!("Attempting to resolve {base_name} as Base");

    let (provider, model ) = if let Some (dissected) = base_name.split_once("/") {
        (dissected.0, Some(dissected.1))
    } else {
        (base_name, None)
    };

    let model_resolved_property =
        model_source.zip(model).map(|(source, value)| {
        ResolvedProperty {
            source,
            value: value.to_string()
        }
    });

    match provider {
        "ollama" => {
            debug!("Resolving {base_name} as Ollama Base");
            Ok(Base::new(
                provider.to_string(),
                BaseProviderConfigSource::Ollama(&appconfig.providers.ollama.config),
                model_resolved_property
            ))
        }
        "anthropic" => {
            debug!("Resolving {base_name} as Anthropic Base");
            Ok(Base::new(
                provider.to_string(),
                BaseProviderConfigSource::Anthropic(&appconfig.providers.anthropic.config),
                model_resolved_property
            ))
        }
        "openai" => {
            debug!("Resolving {base_name} as OpenAI Base");
            Ok(Base::new(
                provider.to_string(),
                BaseProviderConfigSource::OpenAI(&appconfig.providers.openai.config),
                model_resolved_property
            ))
        }
        // "openai" => {
        //     debug!("Resolved {base_name} as OpenAI Base");
        //     Some(Base {
        //         name: "openai",
        //         conf: ProviderConfig::OpenAI("openai", &appconfig.providers.openai.config)
        //     })
        // }
        // "openrouter" => {
        //     debug!("Resolved {base_name} as OpenRouter Base");
        //     Some(Base {
        //         name: "openrouter",
        //         conf: ProviderConfig::OpenRouter("openrouter", &appconfig.providers.openrouter.config)
        //     })
        // }
        // "google" => {
        //     debug!("Resolved {base_name} as Google Base");
        //     Some(Base {
        //         name: "google",
        //         conf: ProviderConfig::Google("google", &appconfig.providers.google.config)
        //     })
        // }
        _ => {
            Err(ResolveError::NotFound(base_name.to_string()))
        }
    }
}

fn resolve_variant(
        appconfig: &AppConfig,
        variant_name: &str,
        model_source: Option<ResolvedPropertySource>) -> Result<Variant, ResolveError> {
    debug!("Attempting to resolve {variant_name} as Variant");

    let (provider, model) = if let Some (dissected) = variant_name.split_once("/") {
        (dissected.0, Some(dissected.1))
    } else {
        (variant_name, None)
    };

    let model_resolved_property =
        model_source.zip(model).map(|(source, value)| {
        ResolvedProperty {
            source,
            value: value.to_string()
        }
    });

    if let Some(conf) = appconfig.providers.ollama.named.get(provider) {
        Ok(Variant::new(
            provider.into(),
            "ollama".into(),
            VariantProviderConfigSource::Ollama(conf, &appconfig.providers.ollama.config),
            model_resolved_property
        ))
    } else if let Some(conf) = appconfig.providers.anthropic.named.get(provider) {
        Ok(Variant::new(
            provider.into(),
            "anthropic".into(),
            VariantProviderConfigSource::Anthropic(conf, &appconfig.providers.anthropic.config),
            model_resolved_property
        ))
    } else if let Some(conf) = appconfig.providers.openai.named.get(provider) {
        Ok(Variant::new(
            provider.into(),
            "openai".into(),
            VariantProviderConfigSource::OpenAI(conf, &appconfig.providers.openai.config),
            model_resolved_property
        ))
    } else {
        Err(ResolveError::NotFound(variant_name.to_string()))
    }
    // else if let Some(conf) = appconfig.providers.google.named.get(variant_name) {
    //     Some(Variant {
    //         name: variant_name.to_string(),
    //         conf: ProviderConfig::Google("google", conf),
    //         base: resolve_base(appconfig, "google")?
    //     })
    // } else if let Some(conf) = appconfig.providers.openrouter.named.get(variant_name) {
    //     Some(Variant {
    //         name: variant_name.to_string(),
    //         conf: ProviderConfig::OpenRouter("openrouter", conf),
    //         base: resolve_base(appconfig, "openrouter")?
    //     })
    // } else  {
    //     appconfig.providers.openai.named.get(variant_name).map(|conf| {
    //         Variant {
    //             name: variant_name.to_string(),
    //             conf: ProviderConfig::OpenAI("openai", conf),
    //             base: resolve_base(appconfig, "openai").unwrap()
    //         }
    //     })
    // }
}

fn resolve_group(appconfig: &AppConfig, group_name: &str) -> Result<Group, ResolveError> {
    let groups = appconfig.group.as_ref().ok_or(ResolveError::NoGroups)?;
    let group = groups.iter().find(|item| item.name == group_name).ok_or(
        ResolveError::NotFound(group_name.to_string()))?;

    let members = group.providers.iter().map(|item| {
        match item {
            GroupProviderConfig::Short(name) => {
                LongGroupProviderConfig {
                    name: name.to_string(),
                    weight: Some(1),
                    fallback: Some(false)
                }
            },
            GroupProviderConfig::Long(long_config) => long_config.clone()
        }
    }).map(|item| {
        let source = Some(ResolvedPropertySource::Group(group_name.into(), item.name.clone()));
        resolve_base(appconfig, &item.name, source.clone())
                .map(|base| GroupMember::Base(base, item.weight.unwrap_or(1)))
                .or_else(|_| {
                    resolve_variant(appconfig, &item.name, source)
                    .map(|variant| GroupMember::Variant(variant, item.weight.unwrap_or(1)))
                })
    }).collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            match e {
                ResolveError::NotFound(name) => ResolveError::GroupMemberNotFound(group_name.to_string(), name),
                err => ResolveError::GroupMemberError(group_name.to_string(), Box::from(err))
            }
    })?;
    Ok(Group {
        name: group_name.to_string(),
        members
    })
}

pub fn resolve(
    appconfig: &AppConfig,
    name: &str,
    model_source: Option<ResolvedPropertySource>
) -> Result<ResolvedConfig, ResolveError> {
    debug!("Resolving {name}");
    match resolve_base(appconfig, name, model_source.clone()) {
        Ok(base) => {
            debug!("Resolved {name} as base");
            Ok(ResolvedConfig::Base(base))
        }
        Err(ResolveError::NotFound(_)) => {
            match resolve_variant(appconfig, name, model_source) {
                Ok(variant) => {
                    debug!("Resolved {name} as variant");
                    Ok(ResolvedConfig::Variant(variant))
                },
                Err(ResolveError::NotFound(_)) => {
                    match resolve_group(appconfig, name) {
                        Ok(group) => {
                            debug!("Resolved {name} as group");
                            Ok(ResolvedConfig::Group(group))
                        }
                        Err(err) => Err(err)
                    }
                },
                Err(err) => Err(err)
            }
        }
        Err(err) => Err(err)
    }
}
