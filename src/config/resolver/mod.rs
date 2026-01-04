pub mod base;
pub mod variant;
pub mod group;
pub mod error;
mod display;

use crate::config::appconfig::{AppConfig, GroupProviderConfig, LongGroupProviderConfig};
use crate::config::providers::ollama;
use crate::config::providers::openai;
use crate::config::providers::anthropic;

pub use variant::Variant;
pub use base::Base;
pub use group::{Group, GroupMember};

use log::debug;

#[derive(Debug)]
pub enum ResolvedConfig {
    Base(base::Base),
    Variant(variant::Variant),
    Group(group::Group)
}

pub enum BaseProviderConfigSource<'a> {
    Ollama(&'a ollama::Config),
    Anthropic(&'a anthropic::Config),
    OpenAI(&'a  openai::Config),
    // OpenRouter(&'a OpenRouterConfig),
    // Google(&'a GoogleConfig),
}

pub enum VariantProviderConfigSource<'a> {
    Ollama(&'a ollama::Config, &'a ollama::Config),
    Anthropic(&'a anthropic::Config, &'a anthropic::Config),
    OpenAI(&'a openai::Config, &'a openai::Config),
    // OpenRouter(&'a OpenRouterConfig),
    // Google(&'a GoogleConfig),
}

#[derive(Debug)]
pub enum ResolvedProviderConfig {
    Ollama(ollama::ResolvedProviderConfig),
    Anthropic(anthropic::ResolvedProviderConfig),
    OpenAI(openai::ResolvedProviderConfig),
    // OpenRouter(ResolvedAnthropicConfig),
    // Google(ResolvedGoogleConfig),
}

#[derive(Clone, Debug)]
pub struct ResolvedProperty<T> {
    pub source: ResolvedPropertySource,
    pub value: T
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

fn resolve_base(appconfig: &AppConfig, base_name: &str,
    model_source: Option<ResolvedPropertySource>) -> Result<Base, error::ResolveError> {

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
            Err(error::ResolveError::NotFound(base_name.to_string()))
        }
    }
}

fn resolve_variant(
        appconfig: &AppConfig,
        variant_name: &str,
        model_source: Option<ResolvedPropertySource>) -> Result<Variant, error::ResolveError> {
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
        Err(error::ResolveError::NotFound(variant_name.to_string()))
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

fn resolve_group(appconfig: &AppConfig, group_name: &str) -> Result<Group, error::ResolveError> {
    let group = appconfig.groups.get(group_name).ok_or(
        error::ResolveError::NotFound(group_name.to_string())
    )?;

    let members = group.providers.iter().map(|item| {
        match item {
            GroupProviderConfig::Short(name) => {
                LongGroupProviderConfig {
                    name: name.to_string(),
                    weight: Some(1),
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
                error::ResolveError::NotFound(name) => error::ResolveError::GroupMemberNotFound(group_name.to_string(), name),
                err => error::ResolveError::GroupMemberError(group_name.to_string(), Box::from(err))
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
) -> Result<ResolvedConfig, error::ResolveError> {
    debug!("Resolving {name}");
    match resolve_base(appconfig, name, model_source.clone()) {
        Ok(base) => {
            debug!("Resolved {name} as base");
            Ok(ResolvedConfig::Base(base))
        }
        Err(error::ResolveError::NotFound(_)) => {
            match resolve_variant(appconfig, name, model_source) {
                Ok(variant) => {
                    debug!("Resolved {name} as variant");
                    Ok(ResolvedConfig::Variant(variant))
                },
                Err(error::ResolveError::NotFound(_)) => {
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
