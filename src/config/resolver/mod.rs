pub mod base;
pub mod variant;
pub mod group;
pub mod error;
mod display;

use std::env;

use crate::config::appconfig::{AppConfig, GlobalProviderProperties, GroupProviderConfig, LongGroupProviderConfig};
use crate::config::providers::ollama;
use crate::config::providers::openai;
use crate::config::providers::anthropic;
use crate::config::providers::google;
use crate::config::providers::openrouter;

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
    Google(&'a google::Config),
    OpenRouter(&'a openrouter::Config),
}

pub enum VariantProviderConfigSource<'a> {
    Ollama(&'a ollama::Config, &'a ollama::Config),
    Anthropic(&'a anthropic::Config, &'a anthropic::Config),
    OpenAI(&'a openai::Config, &'a openai::Config),
    Google(&'a google::Config, &'a google::Config),
    OpenRouter(&'a openrouter::Config, &'a openrouter::Config),
}

#[derive(Debug)]
pub enum ResolvedProviderConfig {
    Ollama(ollama::ResolvedProviderConfig),
    Anthropic(anthropic::ResolvedProviderConfig),
    OpenAI(openai::ResolvedProviderConfig),
    Google(google::ResolvedProviderConfig),
    OpenRouter(openrouter::ResolvedProviderConfig),
}


#[derive(Clone, Debug)]
pub struct ResolvedProperty<T> {
    pub source: ResolvedPropertySource,
    pub value: T
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResolvedPropertySource {
    Group(String, String),
    Variant(String),
    Base(String),
    Env(String),
    Default,
    Globals,
    Dotprompt(String),
    Input(String),
    Inputs,
    Other(String)
}

#[derive(Default, Debug, Clone)]
pub struct ResolvedGlobalProperties {
    pub temperature: Option<ResolvedProperty<f32>>,
    pub max_tokens: Option<ResolvedProperty<u32>>,
    pub model: Option<ResolvedProperty<String>>,
    pub system: Option<ResolvedProperty<String>>,
    pub cache_ttl: Option<ResolvedProperty<u32>>,
    pub stream: Option<ResolvedProperty<bool>>,
}

impl From<(&GlobalProviderProperties, ResolvedPropertySource)> for ResolvedGlobalProperties {
    fn from(value: (&GlobalProviderProperties, ResolvedPropertySource)) -> Self {
        let (props, source) = value;
        ResolvedGlobalProperties {
            temperature: props.temperature.map(|value| ResolvedProperty { source: source.clone(), value }),
            max_tokens: props.max_tokens.map(|value| ResolvedProperty { source: source.clone(), value }),
            model: props.model.as_ref().map(|value| ResolvedProperty { source: source.clone(), value: value.clone() }),
            system: props.system.as_ref().map(|value| ResolvedProperty { source: source.clone(), value: value.clone() }),
            cache_ttl: props.cache_ttl.map(|value| ResolvedProperty { source: source.clone(), value }),
            stream: props.stream.map(|value| ResolvedProperty { source: source.clone(), value }),
        }
    }
}

pub struct Resolver {
    pub overrides: Option<ResolvedGlobalProperties>,
    pub fm_properties: Option<ResolvedGlobalProperties>
}

impl Resolver {

    fn resolve_base(&mut self, appconfig: &AppConfig, base_name_property: ResolvedProperty<String>) -> Result<Base, error::ResolveError> {

        let base_name = base_name_property.value.as_str();
        debug!("Attempting to resolve {base_name} as Base");

        let (provider, model ) = if let Some (dissected) = base_name.split_once("/") {
            (dissected.0, Some(dissected.1))
        } else {
            (base_name, None)
        };


        let model_resolved_property = model.map(|val| {
            ResolvedProperty {
                source: base_name_property.source.clone(),
                value: val.to_string()
            }
        });

        let global_provider_properties = ResolvedGlobalProperties::from(
            (&appconfig.providers.globals, ResolvedPropertySource::Globals)
        );

        match provider {
            "ollama" => {
                debug!("Resolving {base_name} as Ollama Base");
                Ok(Base::new(
                    provider.to_string(),
                    BaseProviderConfigSource::Ollama(&appconfig.providers.ollama.config),
                    self.fm_properties.take(),
                    self.overrides.take(),
                    global_provider_properties,
                    model_resolved_property
                ))
            }
            "anthropic" => {
                debug!("Resolving {base_name} as Anthropic Base");
                Ok(Base::new(
                    provider.to_string(),
                    BaseProviderConfigSource::Anthropic(&appconfig.providers.anthropic.config),
                    self.fm_properties.take(),
                    self.overrides.take(),
                    global_provider_properties,
                    model_resolved_property
                ))
            }
            "openai" => {
                debug!("Resolving {base_name} as OpenAI Base");
                Ok(Base::new(
                    provider.to_string(),
                    BaseProviderConfigSource::OpenAI(&appconfig.providers.openai.config),
                    self.fm_properties.take(),
                    self.overrides.take(),
                    global_provider_properties,
                    model_resolved_property
                ))
            },
            "google" => {
                debug!("Resolving {base_name} as Google Base");
                Ok(Base::new(
                    provider.to_string(),
                    BaseProviderConfigSource::Google(&appconfig.providers.google.config),
                    self.fm_properties.take(),
                    self.overrides.take(),
                    global_provider_properties,
                    model_resolved_property
                ))
            },
            "openrouter" => {
                debug!("Resolving {base_name} as OpenRouter Base");
                Ok(Base::new(
                    provider.to_string(),
                    BaseProviderConfigSource::OpenRouter(&appconfig.providers.openrouter.config),
                    self.fm_properties.take(),
                    self.overrides.take(),
                    global_provider_properties,
                    model_resolved_property
                ))
            },
            _ => {
                Err(error::ResolveError::NotFound(base_name.to_string()))
            }
        }
    }

    fn resolve_variant(
            &mut self,
            appconfig: &AppConfig,
            variant_name_property: ResolvedProperty<String>,
            ) -> Result<Variant, error::ResolveError> {


        let variant_name = variant_name_property.value.as_str();
        debug!("Attempting to resolve {variant_name} as Variant");

        let (provider, model ) = if let Some (dissected) = variant_name.split_once("/") {
            (dissected.0, Some(dissected.1))
        } else {
            (variant_name, None)
        };


        let model_resolved_property = model.map(|val| {
            ResolvedProperty {
                source: variant_name_property.source.clone(),
                value: val.to_string()
            }
        });

        let global_provider_properties = ResolvedGlobalProperties::from(
            (&appconfig.providers.globals, ResolvedPropertySource::Globals)
        );

        if let Some(conf) = appconfig.providers.ollama.named.get(provider) {
            Ok(Variant::new(
                provider.into(),
                "ollama".into(),
                VariantProviderConfigSource::Ollama(conf, &appconfig.providers.ollama.config),
                self.fm_properties.take(),
                self.overrides.take(),
                global_provider_properties,
                model_resolved_property
            ))
        } else if let Some(conf) = appconfig.providers.anthropic.named.get(provider) {
            Ok(Variant::new(
                provider.into(),
                "anthropic".into(),
                VariantProviderConfigSource::Anthropic(conf, &appconfig.providers.anthropic.config),
                self.fm_properties.take(),
                self.overrides.take(),
                global_provider_properties,
                model_resolved_property
            ))
        } else if let Some(conf) = appconfig.providers.openai.named.get(provider) {
            Ok(Variant::new(
                provider.into(),
                "openai".into(),
                VariantProviderConfigSource::OpenAI(conf, &appconfig.providers.openai.config),
                self.fm_properties.take(),
                self.overrides.take(),
                global_provider_properties,
                model_resolved_property
            ))
        } else if let Some(conf) = appconfig.providers.google.named.get(provider) {
            Ok(Variant::new(
                provider.into(),
                "google".into(),
                VariantProviderConfigSource::Google(conf, &appconfig.providers.google.config),
                self.fm_properties.take(),
                self.overrides.take(),
                global_provider_properties,
                model_resolved_property
            ))
        } else if let Some(conf) = appconfig.providers.openrouter.named.get(provider) {
            Ok(Variant::new(
                provider.into(),
                "openrouter".into(),
                VariantProviderConfigSource::OpenRouter(conf, &appconfig.providers.openrouter.config),
                self.fm_properties.take(),
                self.overrides.take(),
                global_provider_properties,
                model_resolved_property
            ))
        } else {
            Err(error::ResolveError::NotFound(variant_name.to_string()))
        }
    }

    fn resolve_group(&mut self, appconfig: &AppConfig,
            group_name_property: ResolvedProperty<String>,
    ) -> Result<Group, error::ResolveError> {
        let group_name = group_name_property.value.as_str();

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
            let resolved_name_property = ResolvedProperty {
                    source: ResolvedPropertySource::Group(group_name.into(), item.name.clone()),
                    value: item.name.clone()
            };

            self.resolve_base(appconfig, resolved_name_property.clone())
                    .map(|base| GroupMember::Base(base, item.weight.unwrap_or(1)))
                    .or_else(|_| {
                        self.resolve_variant(appconfig, resolved_name_property)
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

    fn resolve_name(&self, input_name: Option<String>,
        global_provider_properties: &GlobalProviderProperties,
        config_default: Option<&String>,
    ) -> Option<ResolvedProperty<String>> {
        if let Some(name) = input_name {
            Some(ResolvedProperty {
                source: ResolvedPropertySource::Inputs,
                value: name
            })
        } else if let Some(overrides) = self.overrides.as_ref() && let Some(model) = overrides.model.as_ref() {
            Some(ResolvedProperty {
                source: model.source.clone(),
                value: model.value.clone()
            })
        } else if let Some(fm) = self.fm_properties.as_ref() && let Some(model) = fm.model.as_ref() {
            Some(ResolvedProperty {
                source: model.source.clone(),
                value: model.value.clone()
            })
        } else if let Some(model) = global_provider_properties.model.as_ref() {
            Some(ResolvedProperty {
                source: ResolvedPropertySource::Globals,
                value: model.clone()
            })
        } else if let Some(model) = config_default {
            Some(ResolvedProperty {
                source: ResolvedPropertySource::Globals,
                value: model.clone()
            })
        }
        else if let Ok(model) = env::var("PROMPTCMD_MODEL") {
            Some(ResolvedProperty {
                source: ResolvedPropertySource::Env("PROMPTCMD_MODEL".to_string()),
                value: model.clone()
            })
        } else {
            None
        }
    }

    pub fn resolve(
        &mut self,
        appconfig: &AppConfig,
        input_name: Option<String>,
    ) -> Result<ResolvedConfig, error::ResolveError> {

        let resolved_name_property = self.resolve_name(
            input_name,
            &appconfig.providers.globals,
                        appconfig.providers.default.as_ref())
            .ok_or(error::ResolveError::NoNameToResolve)?;

        debug!("Resolving {resolved_name_property}");

        let name = &resolved_name_property.value;

        match self.resolve_base(appconfig, resolved_name_property.clone()) {
            Ok(base) => {
                debug!("Resolved {name} as base");
                Ok(ResolvedConfig::Base(base))
            }
            Err(error::ResolveError::NotFound(_)) => {
                match self.resolve_variant(appconfig, resolved_name_property.clone()) {
                    Ok(variant) => {
                        debug!("Resolved {name} as variant");
                        Ok(ResolvedConfig::Variant(variant))
                    },
                    Err(error::ResolveError::NotFound(_)) => {
                        match self.resolve_group(appconfig, resolved_name_property.clone()) {
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
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_openai() {
        let mut resolver = Resolver {
            overrides: Some(
                ResolvedGlobalProperties::from((
                     &GlobalProviderProperties {
                        cache_ttl: Some(80),
                        // temperature: Some(0.7),
                        // system: Some("system 7".to_string()),
                        // max_tokens: Some(800),
                        // model: Some("model 8".to_string()),
                        ..Default::default()
                    },
                    ResolvedPropertySource::Inputs
                ))
            ),
            fm_properties: Some(
                ResolvedGlobalProperties::from((
                    &GlobalProviderProperties {
                        cache_ttl: Some(70),
                        temperature: Some(0.7),
                        // system: Some("system 7".to_string()),
                        // max_tokens: Some(700),
                        // model: Some("model 7".to_string()),
                        ..Default::default()
                    },
                    ResolvedPropertySource::Dotprompt("test".to_string())
                ))
            )};

        let mut appconfig = AppConfig::default();

        appconfig.providers.globals = GlobalProviderProperties {
                cache_ttl: Some(20),
                temperature: Some(0.2),
                system: Some("system 2".to_string()),
                max_tokens: Some(200),
                model: Some("model 2".to_string()),
                ..Default::default()
        };

        appconfig.providers.openai = openai::Providers {
            config: openai::Config {
                cache_ttl: Some(40),
                temperature: Some(0.4),
                system: Some("system 4".to_string()),
                // max_tokens: Some(400),
                // model: Some("model 4".to_string()),
                // api_key: Some("api_key 4".to_string()),
                // endpoint: Some("endpoint 4".to_string())
                ..Default::default()
            },
            ..Default::default()
        };

        appconfig.providers.anthropic = anthropic::Providers {
            config: anthropic::Config {
                cache_ttl: Some(41),
                temperature: Some(0.41),
                system: Some("system 41".to_string()),
                // max_tokens: Some(400),
                // model: Some("model 4".to_string()),
                // api_key: Some("api_key 4".to_string()),
                // endpoint: Some("endpoint 4".to_string())
                ..Default::default()
            },
            ..Default::default()
        };

        temp_env::with_vars([
            ("PROMPTCMD_OPENAI_CACHE_TTL", Some("30")),
            ("PROMPTCMD_OPENAI_TEMPERATURE", Some("0.3")),
            ("PROMPTCMD_OPENAI_SYSTEM", Some("system 3")),
            ("PROMPTCMD_OPENAI_MAX_TOKENS", Some("300")),
            // ("PROMPTCMD_OPENAI_MODEL", Some("model 3")),
        ], || {
            if let ResolvedConfig::Base(base) = resolver.resolve(&appconfig, Some("openai".to_string())).unwrap() {
                if let ResolvedProviderConfig::OpenAI(conf) = base.resolved {
                        // cache_ttl comes from overrides, highest priority
                        assert_eq!(conf.globals.cache_ttl.as_ref().unwrap().source, ResolvedPropertySource::Inputs);
                        assert_eq!(conf.globals.cache_ttl.unwrap().value, 80);

                        // temp not set in above
                        assert_eq!(conf.globals.temperature.as_ref().unwrap().source, ResolvedPropertySource::Dotprompt("test".to_string()));
                        assert_eq!(conf.globals.temperature.unwrap().value, 0.7);

                        // system  not set in above
                        assert_eq!(conf.globals.system.as_ref().unwrap().source, ResolvedPropertySource::Base("openai".to_string()));
                        assert_eq!(conf.globals.system.unwrap().value, "system 4");

                        // max_tokens not set in above
                        assert_eq!(conf.globals.max_tokens.as_ref().unwrap().source, ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()));
                        assert_eq!(conf.globals.max_tokens.as_ref().unwrap().value, 300);

                        assert_eq!(conf.globals.model.as_ref().unwrap().source, ResolvedPropertySource::Globals);
                        assert_eq!(conf.globals.model.as_ref().unwrap().value, "model 2");
                } else {
                    panic!("Wrong provider");
                }
            } else {
                panic!("Wrong resolution");
            }
        });


    }

    #[test]
    fn test_anththropic() {
        let mut resolver = Resolver {
            overrides: Some(
                ResolvedGlobalProperties::from((
                     &GlobalProviderProperties {
                        cache_ttl: Some(80),
                        // temperature: Some(0.7),
                        // system: Some("system 7".to_string()),
                        // max_tokens: Some(800),
                        // model: Some("model 8".to_string()),
                        ..Default::default()
                    },
                    ResolvedPropertySource::Inputs
                ))
            ),
            fm_properties: Some(
                ResolvedGlobalProperties::from((
                    &GlobalProviderProperties {
                        cache_ttl: Some(70),
                        temperature: Some(0.7),
                        // system: Some("system 7".to_string()),
                        // max_tokens: Some(700),
                        // model: Some("model 7".to_string()),
                        ..Default::default()
                    },
                    ResolvedPropertySource::Dotprompt("test".to_string())
                ))
            )};

        let mut appconfig = AppConfig::default();

        appconfig.providers.globals = GlobalProviderProperties {
                cache_ttl: Some(20),
                temperature: Some(0.2),
                system: Some("system 2".to_string()),
                max_tokens: Some(200),
                model: Some("model 2".to_string()),
                 ..Default::default()
        };

        appconfig.providers.openai = openai::Providers {
            config: openai::Config {
                cache_ttl: Some(40),
                temperature: Some(0.4),
                system: Some("system 4".to_string()),
                // max_tokens: Some(400),
                // model: Some("model 4".to_string()),
                // api_key: Some("api_key 4".to_string()),
                // endpoint: Some("endpoint 4".to_string())
                ..Default::default()
            },
            ..Default::default()
        };

        appconfig.providers.anthropic = anthropic::Providers {
            config: anthropic::Config {
                cache_ttl: Some(41),
                temperature: Some(0.41),
                system: Some("system 41".to_string()),
                // max_tokens: Some(400),
                // model: Some("model 4".to_string()),
                // api_key: Some("api_key 4".to_string()),
                // endpoint: Some("endpoint 4".to_string())
                ..Default::default()
            },
            ..Default::default()
        };

        temp_env::with_vars([
            ("PROMPTCMD_OPENAI_CACHE_TTL", Some("30")),
            ("PROMPTCMD_OPENAI_TEMPERATURE", Some("0.3")),
            ("PROMPTCMD_OPENAI_SYSTEM", Some("system 3")),
            ("PROMPTCMD_OPENAI_MAX_TOKENS", Some("300")),
            ("PROMPTCMD_ANTHROPIC_CACHE_TTL", Some("31")),
            ("PROMPTCMD_ANTHROPIC_TEMPERATURE", Some("0.31")),
            ("PROMPTCMD_ANTHROPIC_SYSTEM", Some("system 31")),
            ("PROMPTCMD_ANTHROPIC_MAX_TOKENS", Some("301")),
            // ("PROMPTCMD_OPENAI_MODEL", Some("model 3")),
        ], || {
            if let ResolvedConfig::Base(base) = resolver.resolve(&appconfig, Some("anthropic".to_string())).unwrap() {
                if let ResolvedProviderConfig::Anthropic(conf) = base.resolved {
                        // cache_ttl comes from overrides, highest priority
                        assert_eq!(conf.globals.cache_ttl.as_ref().unwrap().source, ResolvedPropertySource::Inputs);
                        assert_eq!(conf.globals.cache_ttl.unwrap().value, 80);

                        // temp not set in above
                        assert_eq!(conf.globals.temperature.as_ref().unwrap().source, ResolvedPropertySource::Dotprompt("test".to_string()));
                        assert_eq!(conf.globals.temperature.unwrap().value, 0.7);

                        // system  not set in above
                        assert_eq!(conf.globals.system.as_ref().unwrap().source, ResolvedPropertySource::Base("anthropic".to_string()));
                        assert_eq!(conf.globals.system.unwrap().value, "system 41");

                        // max_tokens not set in above
                        assert_eq!(conf.globals.max_tokens.as_ref().unwrap().source, ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_MAX_TOKENS".to_string()));
                        assert_eq!(conf.globals.max_tokens.as_ref().unwrap().value, 301);

                        assert_eq!(conf.globals.model.as_ref().unwrap().source, ResolvedPropertySource::Globals);
                        assert_eq!(conf.globals.model.as_ref().unwrap().value, "model 2");
                } else {
                    panic!("Wrong provider");
                }
            } else {
                panic!("Wrong resolution");
            }
        });
    }

    #[test]
    fn test_openai_with_modelname() {
        let mut resolver = Resolver {
            overrides: Some(
                ResolvedGlobalProperties::from((
                     &GlobalProviderProperties {
                        cache_ttl: Some(80),
                        // temperature: Some(0.7),
                        // system: Some("system 7".to_string()),
                        // max_tokens: Some(800),
                        // model: Some("model 8".to_string()),
                        ..Default::default()
                    },
                    ResolvedPropertySource::Inputs
                ))
            ),
            fm_properties: Some(
                ResolvedGlobalProperties::from((
                    &GlobalProviderProperties {
                        cache_ttl: Some(70),
                        temperature: Some(0.7),
                        // system: Some("system 7".to_string()),
                        // max_tokens: Some(700),
                        // model: Some("model 7".to_string()),
                        ..Default::default()
                    },
                    ResolvedPropertySource::Dotprompt("test".to_string())
                ))
            )};

        let mut appconfig = AppConfig::default();

        appconfig.providers.globals = GlobalProviderProperties {
                cache_ttl: Some(20),
                temperature: Some(0.2),
                system: Some("system 2".to_string()),
                max_tokens: Some(200),
                model: Some("model 2".to_string()),
                 ..Default::default()
        };

        appconfig.providers.openai = openai::Providers {
            config: openai::Config {
                cache_ttl: Some(40),
                temperature: Some(0.4),
                system: Some("system 4".to_string()),
                // max_tokens: Some(400),
                // model: Some("model 4".to_string()),
                // api_key: Some("api_key 4".to_string()),
                // endpoint: Some("endpoint 4".to_string())
                ..Default::default()
            },
            ..Default::default()
        };

        appconfig.providers.anthropic = anthropic::Providers {
            config: anthropic::Config {
                cache_ttl: Some(41),
                temperature: Some(0.41),
                system: Some("system 41".to_string()),
                // max_tokens: Some(400),
                // model: Some("model 4".to_string()),
                // api_key: Some("api_key 4".to_string()),
                // endpoint: Some("endpoint 4".to_string())
                ..Default::default()
            },
            ..Default::default()
        };

        temp_env::with_vars([
            ("PROMPTCMD_OPENAI_CACHE_TTL", Some("30")),
            ("PROMPTCMD_OPENAI_TEMPERATURE", Some("0.3")),
            ("PROMPTCMD_OPENAI_SYSTEM", Some("system 3")),
            ("PROMPTCMD_OPENAI_MAX_TOKENS", Some("300")),
            // ("PROMPTCMD_OPENAI_MODEL", Some("model 3")),
        ], || {
            if let ResolvedConfig::Base(base) = resolver.resolve(&appconfig, Some("openai/gpt5".to_string())).unwrap() {
                if let ResolvedProviderConfig::OpenAI(conf) = base.resolved {
                        // cache_ttl comes from overrides, highest priority
                        assert_eq!(conf.globals.cache_ttl.as_ref().unwrap().source, ResolvedPropertySource::Inputs);
                        assert_eq!(conf.globals.cache_ttl.unwrap().value, 80);

                        // temp not set in above
                        assert_eq!(conf.globals.temperature.as_ref().unwrap().source, ResolvedPropertySource::Dotprompt("test".to_string()));
                        assert_eq!(conf.globals.temperature.unwrap().value, 0.7);

                        // system  not set in above
                        assert_eq!(conf.globals.system.as_ref().unwrap().source, ResolvedPropertySource::Base("openai".to_string()));
                        assert_eq!(conf.globals.system.unwrap().value, "system 4");

                        // max_tokens not set in above
                        assert_eq!(conf.globals.max_tokens.as_ref().unwrap().source, ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()));
                        assert_eq!(conf.globals.max_tokens.as_ref().unwrap().value, 300);

                        assert_eq!(conf.globals.model.as_ref().unwrap().source, ResolvedPropertySource::Inputs);
                        assert_eq!(conf.globals.model.as_ref().unwrap().value, "gpt5");
                } else {
                    panic!("Wrong provider");
                }
            } else {
                panic!("Wrong resolution");
            }
        });
    }

    #[test]
    fn test_openai_with_modelname_in_frontmatter() {
        let mut resolver = Resolver {
            overrides: Some(
                ResolvedGlobalProperties::from((
                     &GlobalProviderProperties {
                        cache_ttl: Some(80),
                        // temperature: Some(0.7),
                        // system: Some("system 7".to_string()),
                        // max_tokens: Some(800),
                        // model: Some("model 8".to_string()),
                        ..Default::default()
                    },
                    ResolvedPropertySource::Inputs
                ))
            ),
            fm_properties: Some(
                ResolvedGlobalProperties::from((
                    &GlobalProviderProperties {
                        cache_ttl: Some(70),
                        temperature: Some(0.7),
                        // system: Some("system 7".to_string()),
                        // max_tokens: Some(700),
                        model: Some("model 7".to_string()),
                        ..Default::default()
                    },
                    ResolvedPropertySource::Dotprompt("test".to_string())
                ))
            )};

        let mut appconfig = AppConfig::default();

        appconfig.providers.globals = GlobalProviderProperties {
                cache_ttl: Some(20),
                temperature: Some(0.2),
                system: Some("system 2".to_string()),
                max_tokens: Some(200),
                model: Some("model 2".to_string()),
                 ..Default::default()
        };

        appconfig.providers.openai = openai::Providers {
            config: openai::Config {
                cache_ttl: Some(40),
                temperature: Some(0.4),
                system: Some("system 4".to_string()),
                // max_tokens: Some(400),
                // model: Some("model 4".to_string()),
                // api_key: Some("api_key 4".to_string()),
                // endpoint: Some("endpoint 4".to_string())
                ..Default::default()
            },
            ..Default::default()
        };

        appconfig.providers.anthropic = anthropic::Providers {
            config: anthropic::Config {
                cache_ttl: Some(41),
                temperature: Some(0.41),
                system: Some("system 41".to_string()),
                // max_tokens: Some(400),
                // model: Some("model 4".to_string()),
                // api_key: Some("api_key 4".to_string()),
                // endpoint: Some("endpoint 4".to_string())
                ..Default::default()
            },
            ..Default::default()
        };

        temp_env::with_vars([
            ("PROMPTCMD_OPENAI_CACHE_TTL", Some("30")),
            ("PROMPTCMD_OPENAI_TEMPERATURE", Some("0.3")),
            ("PROMPTCMD_OPENAI_SYSTEM", Some("system 3")),
            ("PROMPTCMD_OPENAI_MAX_TOKENS", Some("300")),
            // ("PROMPTCMD_OPENAI_MODEL", Some("model 3")),
        ], || {
            if let ResolvedConfig::Base(base) = resolver.resolve(&appconfig, Some("openai".to_string())).unwrap() {
                if let ResolvedProviderConfig::OpenAI(conf) = base.resolved {
                        // cache_ttl comes from overrides, highest priority
                        assert_eq!(conf.globals.cache_ttl.as_ref().unwrap().source, ResolvedPropertySource::Inputs);
                        assert_eq!(conf.globals.cache_ttl.unwrap().value, 80);

                        // temp not set in above
                        assert_eq!(conf.globals.temperature.as_ref().unwrap().source, ResolvedPropertySource::Dotprompt("test".to_string()));
                        assert_eq!(conf.globals.temperature.unwrap().value, 0.7);

                        // system  not set in above
                        assert_eq!(conf.globals.system.as_ref().unwrap().source, ResolvedPropertySource::Base("openai".to_string()));
                        assert_eq!(conf.globals.system.unwrap().value, "system 4");

                        // max_tokens not set in above
                        assert_eq!(conf.globals.max_tokens.as_ref().unwrap().source, ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()));
                        assert_eq!(conf.globals.max_tokens.as_ref().unwrap().value, 300);

                        assert_eq!(conf.globals.model.as_ref().unwrap().source, ResolvedPropertySource::Dotprompt("test".to_string()));
                        assert_eq!(conf.globals.model.as_ref().unwrap().value, "model 7");
                } else {
                    panic!("Wrong provider");
                }
            } else {
                panic!("Wrong resolution");
            }
        });
    }
    #[test]
    fn test_openai_with_modelname_in_frontmatter_overriden_by_input() {
        let mut resolver = Resolver {
            overrides: Some(
                ResolvedGlobalProperties::from((
                     &GlobalProviderProperties {
                        cache_ttl: Some(80),
                        // temperature: Some(0.7),
                        // system: Some("system 7".to_string()),
                        // max_tokens: Some(800),
                        // model: Some("model 8".to_string()),
                        ..Default::default()
                    },
                    ResolvedPropertySource::Inputs
                ))
            ),
            fm_properties: Some(
                ResolvedGlobalProperties::from((
                    &GlobalProviderProperties {
                        cache_ttl: Some(70),
                        temperature: Some(0.7),
                        // system: Some("system 7".to_string()),
                        // max_tokens: Some(700),
                        model: Some("model 7".to_string()),
                        ..Default::default()
                    },
                    ResolvedPropertySource::Dotprompt("test".to_string())
                ))
            )};

        let mut appconfig = AppConfig::default();

        appconfig.providers.globals = GlobalProviderProperties {
                cache_ttl: Some(20),
                temperature: Some(0.2),
                system: Some("system 2".to_string()),
                max_tokens: Some(200),
                model: Some("model 2".to_string()),
                 ..Default::default()
        };

        appconfig.providers.openai = openai::Providers {
            config: openai::Config {
                cache_ttl: Some(40),
                temperature: Some(0.4),
                system: Some("system 4".to_string()),
                // max_tokens: Some(400),
                // model: Some("model 4".to_string()),
                // api_key: Some("api_key 4".to_string()),
                // endpoint: Some("endpoint 4".to_string())
                ..Default::default()
            },
            ..Default::default()
        };

        appconfig.providers.anthropic = anthropic::Providers {
            config: anthropic::Config {
                cache_ttl: Some(41),
                temperature: Some(0.41),
                system: Some("system 41".to_string()),
                // max_tokens: Some(400),
                // model: Some("model 4".to_string()),
                // api_key: Some("api_key 4".to_string()),
                // endpoint: Some("endpoint 4".to_string())
                ..Default::default()
            },
            ..Default::default()
        };

        temp_env::with_vars([
            ("PROMPTCMD_OPENAI_CACHE_TTL", Some("30")),
            ("PROMPTCMD_OPENAI_TEMPERATURE", Some("0.3")),
            ("PROMPTCMD_OPENAI_SYSTEM", Some("system 3")),
            ("PROMPTCMD_OPENAI_MAX_TOKENS", Some("300")),
            // ("PROMPTCMD_OPENAI_MODEL", Some("model 3")),
        ], || {
            if let ResolvedConfig::Base(base) = resolver.resolve(&appconfig, Some("openai/gpt5".to_string())).unwrap() {
                if let ResolvedProviderConfig::OpenAI(conf) = base.resolved {
                        // cache_ttl comes from overrides, highest priority
                        assert_eq!(conf.globals.cache_ttl.as_ref().unwrap().source, ResolvedPropertySource::Inputs);
                        assert_eq!(conf.globals.cache_ttl.unwrap().value, 80);

                        // temp not set in above
                        assert_eq!(conf.globals.temperature.as_ref().unwrap().source, ResolvedPropertySource::Dotprompt("test".to_string()));
                        assert_eq!(conf.globals.temperature.unwrap().value, 0.7);

                        // system  not set in above
                        assert_eq!(conf.globals.system.as_ref().unwrap().source, ResolvedPropertySource::Base("openai".to_string()));
                        assert_eq!(conf.globals.system.unwrap().value, "system 4");

                        // max_tokens not set in above
                        assert_eq!(conf.globals.max_tokens.as_ref().unwrap().source, ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()));
                        assert_eq!(conf.globals.max_tokens.as_ref().unwrap().value, 300);

                        assert_eq!(conf.globals.model.as_ref().unwrap().source, ResolvedPropertySource::Inputs);
                        assert_eq!(conf.globals.model.as_ref().unwrap().value, "gpt5");
                } else {
                    panic!("Wrong provider");
                }
            } else {
                panic!("Wrong resolution");
            }
        });
    }

    #[test]
    fn test_openai_with_full_modelname_in_frontmatter() {
        let mut resolver = Resolver {
            overrides: Some(
                ResolvedGlobalProperties::from((
                     &GlobalProviderProperties {
                        cache_ttl: Some(80),
                        // temperature: Some(0.7),
                        // system: Some("system 7".to_string()),
                        // max_tokens: Some(800),
                        // model: Some("model 8".to_string()),
                        ..Default::default()
                    },
                    ResolvedPropertySource::Inputs
                ))
            ),
            fm_properties: Some(
                ResolvedGlobalProperties::from((
                    &GlobalProviderProperties {
                        cache_ttl: Some(70),
                        temperature: Some(0.7),
                        // system: Some("system 7".to_string()),
                        // max_tokens: Some(700),
                        model: Some("openai/model 7".to_string()),
                        ..Default::default()
                    },
                    ResolvedPropertySource::Dotprompt("test".to_string())
                ))
            )};

        let mut appconfig = AppConfig::default();

        appconfig.providers.globals = GlobalProviderProperties {
                cache_ttl: Some(20),
                temperature: Some(0.2),
                system: Some("system 2".to_string()),
                max_tokens: Some(200),
                model: Some("model 2".to_string()),
                 ..Default::default()
        };

        appconfig.providers.openai = openai::Providers {
            config: openai::Config {
                cache_ttl: Some(40),
                temperature: Some(0.4),
                system: Some("system 4".to_string()),
                // max_tokens: Some(400),
                // model: Some("model 4".to_string()),
                // api_key: Some("api_key 4".to_string()),
                // endpoint: Some("endpoint 4".to_string())
                ..Default::default()
            },
            ..Default::default()
        };

        appconfig.providers.anthropic = anthropic::Providers {
            config: anthropic::Config {
                cache_ttl: Some(41),
                temperature: Some(0.41),
                system: Some("system 41".to_string()),
                // max_tokens: Some(400),
                // model: Some("model 4".to_string()),
                // api_key: Some("api_key 4".to_string()),
                // endpoint: Some("endpoint 4".to_string())
                ..Default::default()
            },
            ..Default::default()
        };

        temp_env::with_vars([
            ("PROMPTCMD_OPENAI_CACHE_TTL", Some("30")),
            ("PROMPTCMD_OPENAI_TEMPERATURE", Some("0.3")),
            ("PROMPTCMD_OPENAI_SYSTEM", Some("system 3")),
            ("PROMPTCMD_OPENAI_MAX_TOKENS", Some("300")),
            // ("PROMPTCMD_OPENAI_MODEL", Some("model 3")),
        ], || {
            if let ResolvedConfig::Base(base) = resolver.resolve(&appconfig, None).unwrap() {
                if let ResolvedProviderConfig::OpenAI(conf) = base.resolved {
                        // cache_ttl comes from overrides, highest priority
                        assert_eq!(conf.globals.cache_ttl.as_ref().unwrap().source, ResolvedPropertySource::Inputs);
                        assert_eq!(conf.globals.cache_ttl.unwrap().value, 80);

                        // temp not set in above
                        assert_eq!(conf.globals.temperature.as_ref().unwrap().source, ResolvedPropertySource::Dotprompt("test".to_string()));
                        assert_eq!(conf.globals.temperature.unwrap().value, 0.7);

                        // system  not set in above
                        assert_eq!(conf.globals.system.as_ref().unwrap().source, ResolvedPropertySource::Base("openai".to_string()));
                        assert_eq!(conf.globals.system.unwrap().value, "system 4");

                        // max_tokens not set in above
                        assert_eq!(conf.globals.max_tokens.as_ref().unwrap().source, ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()));
                        assert_eq!(conf.globals.max_tokens.as_ref().unwrap().value, 300);

                        assert_eq!(conf.globals.model.as_ref().unwrap().source, ResolvedPropertySource::Dotprompt("test".to_string()));
                        assert_eq!(conf.globals.model.as_ref().unwrap().value, "model 7");
                } else {
                    panic!("Wrong provider");
                }
            } else {
                panic!("Wrong resolution");
            }
        });
    }

    #[test]
    fn test_openrouter_variant() {
        let mut resolver = Resolver {
            overrides: Some(
                ResolvedGlobalProperties::from((
                     &GlobalProviderProperties {
                        cache_ttl: Some(80),
                        // temperature: Some(0.7),
                        // system: Some("system 7".to_string()),
                        // max_tokens: Some(800),
                        // model: Some("model 8".to_string()),
                        ..Default::default()
                    },
                    ResolvedPropertySource::Inputs
                ))
            ),
            fm_properties: Some(
                ResolvedGlobalProperties::from((
                    &GlobalProviderProperties {
                        cache_ttl: Some(70),
                        temperature: Some(0.7),
                        // system: Some("system 7".to_string()),
                        // max_tokens: Some(700),
                        // model: Some("openai/model 7".to_string()),
                        ..Default::default()
                    },
                    ResolvedPropertySource::Dotprompt("test".to_string())
                ))
            )};

        let mut appconfig = AppConfig::default();

        appconfig.providers.globals = GlobalProviderProperties {
                cache_ttl: Some(20),
                temperature: Some(0.2),
                system: Some("system 2".to_string()),
                max_tokens: Some(200),
                model: Some("model 2".to_string()),
                 ..Default::default()
        };

        appconfig.providers.openrouter = openrouter::Providers {
            config: openrouter::Config {
                cache_ttl: Some(40),
                temperature: Some(0.4),
                system: Some("system 4".to_string()),
                // max_tokens: Some(400),
                // model: Some("model 4".to_string()),
                // api_key: Some("api_key 4".to_string()),
                // endpoint: Some("endpoint 4".to_string())
                ..Default::default()
            },
            ..Default::default()
        };
        appconfig.providers.openrouter.named.insert("myvariant".to_string(),
            openrouter::Config {
                ..Default::default()
            }
        );

        temp_env::with_vars([
            ("PROMPTCMD_OPENROUTER_CACHE_TTL", Some("30")),
            ("PROMPTCMD_OPENROUTER_TEMPERATURE", Some("0.3")),
            ("PROMPTCMD_OPENROUTER_SYSTEM", Some("system 3")),
            ("PROMPTCMD_OPENROUTER_MAX_TOKENS", Some("300")),
            // ("PROMPTCMD_OPENAI_MODEL", Some("model 3")),
        ], || {
            if let ResolvedConfig::Variant(variant) = resolver.resolve(&appconfig, Some("myvariant".to_string())).unwrap() {
                if let ResolvedProviderConfig::OpenRouter(conf) = variant.resolved {
                        // cache_ttl comes from overrides, highest priority
                        assert_eq!(conf.globals.cache_ttl.as_ref().unwrap().source, ResolvedPropertySource::Inputs);
                        assert_eq!(conf.globals.cache_ttl.unwrap().value, 80);

                        // temp not set in above
                        assert_eq!(conf.globals.temperature.as_ref().unwrap().source, ResolvedPropertySource::Dotprompt("test".to_string()));
                        assert_eq!(conf.globals.temperature.unwrap().value, 0.7);

                        // system  not set in above
                        assert_eq!(conf.globals.system.as_ref().unwrap().source, ResolvedPropertySource::Base("openrouter".to_string()));
                        assert_eq!(conf.globals.system.unwrap().value, "system 4");

                        // max_tokens not set in above
                        assert_eq!(conf.globals.max_tokens.as_ref().unwrap().source, ResolvedPropertySource::Env("PROMPTCMD_OPENROUTER_MAX_TOKENS".to_string()));
                        assert_eq!(conf.globals.max_tokens.as_ref().unwrap().value, 300);

                        assert_eq!(conf.globals.model.as_ref().unwrap().source, ResolvedPropertySource::Globals);
                        assert_eq!(conf.globals.model.as_ref().unwrap().value, "model 2");
                } else {
                    panic!("Wrong provider");
                }
            } else {
                panic!("Wrong resolution");
            }
        });


    }
}
