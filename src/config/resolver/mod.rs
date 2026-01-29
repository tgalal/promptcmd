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


#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum ResolvedProviderConfig {
    Ollama(ollama::ResolvedProviderConfig),
    Anthropic(anthropic::ResolvedProviderConfig),
    OpenAI(openai::ResolvedProviderConfig),
    Google(google::ResolvedProviderConfig),
    OpenRouter(openrouter::ResolvedProviderConfig),
}


#[derive(Clone, Debug, PartialEq)]
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

#[derive(Default, Debug, Clone, PartialEq)]
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

    fn resolve_base(&self, appconfig: &AppConfig, base_name_property: ResolvedProperty<String>) -> Result<Base, error::ResolveError> {

        let base_name = base_name_property.value.as_str();
        debug!("Attempting to resolve {base_name} as Base");

        let (provider, model ) = if let Some (dissected) = base_name.split_once("/") {
            (dissected.0, Some(dissected.1))
        } else {
            (base_name, None)
        };

        // If the base name is a short form (e.g., openai) this is going to be None.
        let model_resolved_property = model.map(|val| {
            ResolvedProperty {
                source: base_name_property.source.clone(),
                value: val.to_string()
            }
        });

        let global_provider_properties = ResolvedGlobalProperties::from(
            (&appconfig.providers.globals, ResolvedPropertySource::Globals)
        );

        // In certain cases we omit the model specified in FM:
        // - We have a model specified via command line/inputs, this should
        // override whatever the frontmatter says
        // - The model name comes from inside a group, in this case whatever the
        // FM says about the model name is irrelevant
        // - The FM has only a "shortform" (e.g., openai). This is not a full
        // model name and has already been used in resolving the
        // provider/group/variant.
        let fm_properties = self.fm_properties.clone().map(|props| {
            let model =  if matches!(base_name_property.source,
                ResolvedPropertySource::Inputs | ResolvedPropertySource::Group(_, _)) {
                None
            } else {
                props.model.and_then(|m| if !m.value.contains("/") {None} else {Some(m)})
            };
            ResolvedGlobalProperties {
                model,
                ..props
            }
        });

        let base = match provider {
            "ollama" => {
                debug!("Resolving {base_name} as Ollama Base");
                Base::new(
                    provider.to_string(),
                    BaseProviderConfigSource::Ollama(&appconfig.providers.ollama.config),
                    fm_properties,
                    self.overrides.clone(),
                    global_provider_properties,
                    model_resolved_property
                )
            }
            "anthropic" => {
                debug!("Resolving {base_name} as Anthropic Base");
                Base::new(
                    provider.to_string(),
                    BaseProviderConfigSource::Anthropic(&appconfig.providers.anthropic.config),
                    fm_properties,
                    self.overrides.clone(),
                    global_provider_properties,
                    model_resolved_property
                )
            }
            "openai" => {
                debug!("Resolving {base_name} as OpenAI Base");
                Base::new(
                    provider.to_string(),
                    BaseProviderConfigSource::OpenAI(&appconfig.providers.openai.config),
                    fm_properties,
                    self.overrides.clone(),
                    global_provider_properties,
                    model_resolved_property
                )
            },
            "google" => {
                debug!("Resolving {base_name} as Google Base");
                Base::new(
                    provider.to_string(),
                    BaseProviderConfigSource::Google(&appconfig.providers.google.config),
                    fm_properties,
                    self.overrides.clone(),
                    global_provider_properties,
                    model_resolved_property
                )
            },
            "openrouter" => {
                debug!("Resolving {base_name} as OpenRouter Base");
                Base::new(
                    provider.to_string(),
                    BaseProviderConfigSource::OpenRouter(&appconfig.providers.openrouter.config),
                    fm_properties,
                    self.overrides.clone(),
                    global_provider_properties,
                    model_resolved_property
                )
            },
            _ => {
                Err(error::ResolveError::NotFound(base_name.to_string()))?
            }
        };

        if base.model_info.is_err() {
            Err(error::ResolveError::NoNameToResolve)
        } else {
            Ok(base)
        }
    }

    fn resolve_variant(
            &self,
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

        // In certain cases we omit the model specified in FM:
        // - We have a model specified via command line/inputs, this should
        // override whatever the frontmatter says
        // - The model name comes from inside a group, in this case whatever the
        // FM says about the model name is irrelevant
        // - The FM has only a "shortform" (e.g., openai). This is not a full
        // model name and has already been used in resolving the
        // provider/group/variant.
        let fm_properties = self.fm_properties.clone().map(|props| {
            let model =  if matches!(variant_name_property.source,
                ResolvedPropertySource::Inputs | ResolvedPropertySource::Group(_, _)) {
                None
            } else {
                props.model.and_then(|m| if !m.value.contains("/") {None} else {Some(m)})
            };
            ResolvedGlobalProperties {
                model,
                ..props
            }
        });

        let global_provider_properties = ResolvedGlobalProperties::from(
            (&appconfig.providers.globals, ResolvedPropertySource::Globals)
        );

        let variant = if let Some(conf) = appconfig.providers.ollama.named.get(provider) {
            Variant::new(
                provider.into(),
                "ollama".into(),
                VariantProviderConfigSource::Ollama(conf, &appconfig.providers.ollama.config),
                fm_properties,
                self.overrides.clone(),
                global_provider_properties,
                model_resolved_property
            )
        } else if let Some(conf) = appconfig.providers.anthropic.named.get(provider) {
            Variant::new(
                provider.into(),
                "anthropic".into(),
                VariantProviderConfigSource::Anthropic(conf, &appconfig.providers.anthropic.config),
                fm_properties,
                self.overrides.clone(),
                global_provider_properties,
                model_resolved_property
            )
        } else if let Some(conf) = appconfig.providers.openai.named.get(provider) {
            Variant::new(
                provider.into(),
                "openai".into(),
                VariantProviderConfigSource::OpenAI(conf, &appconfig.providers.openai.config),
                fm_properties,
                self.overrides.clone(),
                global_provider_properties,
                model_resolved_property
            )
        } else if let Some(conf) = appconfig.providers.google.named.get(provider) {
            Variant::new(
                provider.into(),
                "google".into(),
                VariantProviderConfigSource::Google(conf, &appconfig.providers.google.config),
                fm_properties,
                self.overrides.clone(),
                global_provider_properties,
                model_resolved_property
            )
        } else if let Some(conf) = appconfig.providers.openrouter.named.get(provider) {
            Variant::new(
                provider.into(),
                "openrouter".into(),
                VariantProviderConfigSource::OpenRouter(conf, &appconfig.providers.openrouter.config),
                fm_properties,
                self.overrides.clone(),
                global_provider_properties,
                model_resolved_property
            )
        } else {
            Err(error::ResolveError::NotFound(variant_name.to_string()))?
        };

        if variant.model_info.is_err() {
            Err(error::ResolveError::NoNameToResolve)
        } else {
            Ok(variant)
        }
    }

    fn resolve_group(&self, appconfig: &AppConfig,
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

    /// This functions determines the name of the provider and model to use in resolution.
    /// It does several look ups fro highest to lower priority. The end result
    /// is either a short form (e.g., openai) or a full one  (e.g., openai/gpt5).
    ///
    /// If only a provider is found, then it gets use for resolving the configuration.
    /// If additionally a model is specified, then depending on configuration resolution
    /// it may or may not be used as the final model name, depending on its and how this
    /// source fits in the configuration resolution priority list.
    ///
    /// The current name look up list (high to low prio) is:
    /// - Direct input (e.g., from a command override)
    /// - Direct Overrides
    /// - Frontmatter
    /// - Global Configuration for all Providers (model key)
    /// - Global Configuration for all Providers (default key)
    /// - Environment Variable
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
        &self,
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

    use rstest::{rstest};
    use pretty_assertions::{assert_eq};

    use crate::{config::{providers::{constants::DEFAULT_STREAM, ModelInfo}, resolver::error::ResolveError}, dotprompt::DotPrompt};

    use super::*;

    enum ResolveType {Base, Variant, Group, Fail}
    // env, appconfig, promptfile, overrides (e.g., from command line)
    const CONFIG_1: (&str, &str, &str, GlobalProviderProperties) = (
r#"
PROMPTCMD_OPENAI_CACHE_TTL=30
PROMPTCMD_OPENAI_TEMPERATURE=0.3
PROMPTCMD_OPENAI_SYSTEM=system 3
PROMPTCMD_OPENAI_MAX_TOKENS=300
PROMPTCMD_ANTHROPIC_STREAM=true
"#,
r#"
[providers]
cache_ttl = 20
temperature = 0.2
system = "system 2"
max_tokens = 200
default = "openai"

[providers.openai]
api_key = "openaikey"
endpoint = "openaiendpoint"
model = "gpt4"
cache_ttl = 40
temperature = 0.4
system = "system 4"

[providers.anthropic]
api_key = "anthropickey"
model = "claude"
cache_ttl = 50
temperature = 0.5
system = "system 5"

[providers.google]
api_key = "googlekey"

[providers.anthropic.rust-coder]
system = "rust-coder sys msg"

[providers.anthropic.rust-coder-diffmodel]
system = "rust-coder sys msg"
model = "clauderust"

[providers.anthropic.override_apikey]
api_key = "overridden_key"

[groups.group_of_short_bases]
providers = ["anthropic", "openai"]

[groups.group_of_short_variants]
providers = ["rust-coder-diffmodel", "rust-coder"]

[groups.group_of_mixed_shorts]
providers = ["openai", "rust-coder"]

[groups.group_with_missing_provider]
providers = ["openai", "badname"]

"#,
r#"
---
config:
    cache_ttl: 70
    temperature: 0.7
---
Templ
"#, GlobalProviderProperties {
    cache_ttl: Some(80),
    temperature: None,
    max_tokens: None,
    model: None,
    system: None,
    stream: None
}
);

    #[rstest]
#[case::one(
  Some("openai".to_string()),
  Ok(ResolvedConfig::Base(Base {
                name: "openai".to_string(),
                resolved:
                    ResolvedProviderConfig::OpenAI(openai::ResolvedProviderConfig {
                        api_key: Some(ResolvedProperty {         source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("openaikey") }),
                        endpoint: Some(ResolvedProperty {        source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("openaiendpoint") }),
                        globals: ResolvedGlobalProperties {
                            cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                            temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                            system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("system 4") }),
                            max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()), value: 300 }),
                            model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("openai".to_string()),                     value: "gpt4".to_string() }),
                            stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Default,                                        value: DEFAULT_STREAM }),
                        },
                    })
                ,
                model_info: Ok(ModelInfo {
                    provider: "openai".to_string(),
                    model: "gpt4".to_string()
                }),
                globals: ResolvedGlobalProperties {
                    cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                    temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                    system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("system 4") }),
                    max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()), value: 300 }),
                    model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("openai".to_string()),                     value: "gpt4".to_string() }),
                    stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Default,                                        value: DEFAULT_STREAM }),
                },
    }))
)]
#[case::two(
  Some("openai/gpt5".to_string()),
  Ok(ResolvedConfig::Base(Base {
                name: "openai".to_string(),
                resolved:
                    ResolvedProviderConfig::OpenAI(openai::ResolvedProviderConfig {
                        api_key: Some(ResolvedProperty {         source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("openaikey") }),
                        endpoint: Some(ResolvedProperty {        source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("openaiendpoint") }),
                        globals: ResolvedGlobalProperties {
                            cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                            temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                            system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("system 4") }),
                            max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()), value: 300 }),
                            model: Some(ResolvedProperty {       source: ResolvedPropertySource::Inputs,                                        value: "gpt5".to_string() }),
                            stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Default,                                        value: DEFAULT_STREAM }),
                        },
                    })
                ,
                model_info: Ok(ModelInfo {
                    provider: "openai".to_string(),
                    model: "gpt5".to_string()
                }),
                globals: ResolvedGlobalProperties {
                    cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                    temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                    system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("system 4") }),
                    max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()), value: 300 }),
                    model: Some(ResolvedProperty {       source: ResolvedPropertySource::Inputs,                                        value: "gpt5".to_string() }),
                    stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Default,                                        value: DEFAULT_STREAM }),
                },
    }))
)]
#[case::three(
  Some("anthropic".to_string()),
  Ok(ResolvedConfig::Base(Base {
                name: "anthropic".to_string(),
                resolved:
                    ResolvedProviderConfig::Anthropic(anthropic::ResolvedProviderConfig {
                        api_key: Some(ResolvedProperty {         source: ResolvedPropertySource::Base("anthropic".to_string()),                 value: String::from("anthropickey") }),
                        globals: ResolvedGlobalProperties {
                            cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                            temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                            system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("anthropic".to_string()),                   value: String::from("system 5") }),
                            max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                            model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("anthropic".to_string()),                             value: "claude".to_string() }),
                            stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                        },
                    })
                ,
                model_info: Ok(ModelInfo {
                    provider: "anthropic".to_string(),
                    model: "claude".to_string()
                }),
                globals: ResolvedGlobalProperties {
                    cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                    temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                    system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("anthropic".to_string()),                   value: String::from("system 5") }),
                    max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                    model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("anthropic".to_string()),                             value: "claude".to_string() }),
                    stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                },
    }))
)]
#[case::variant(
  Some("rust-coder".to_string()),
  Ok(ResolvedConfig::Variant(Variant {
                base_name: "anthropic".to_string(),
                name: "rust-coder".to_string(),
                resolved:
                    ResolvedProviderConfig::Anthropic(anthropic::ResolvedProviderConfig {
                        api_key: Some(ResolvedProperty {         source: ResolvedPropertySource::Base("anthropic".to_string()),                 value: String::from("anthropickey") }),
                        globals: ResolvedGlobalProperties {
                            cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                            temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                            system: Some(ResolvedProperty {      source: ResolvedPropertySource::Variant("rust-coder".to_string()),                   value: String::from("rust-coder sys msg") }),
                            max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                            model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("anthropic".to_string()),                             value: "claude".to_string() }),
                            stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                        },
                    })
                ,
                model_info: Ok(ModelInfo {
                    provider: "anthropic".to_string(),
                    model: "claude".to_string()
                }),
                globals: ResolvedGlobalProperties {
                    cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                    temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                    system: Some(ResolvedProperty {      source: ResolvedPropertySource::Variant("rust-coder".to_string()),                   value: String::from("rust-coder sys msg") }),
                    max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                    model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("anthropic".to_string()),                             value: "claude".to_string() }),
                    stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                },
    }))
)]
#[case::variant_withmodel(
  Some("rust-coder/custom_model".to_string()),
  Ok(ResolvedConfig::Variant(Variant {
                base_name: "anthropic".to_string(),
                name: "rust-coder".to_string(),
                resolved:
                    ResolvedProviderConfig::Anthropic(anthropic::ResolvedProviderConfig {
                        api_key: Some(ResolvedProperty {         source: ResolvedPropertySource::Base("anthropic".to_string()),                 value: String::from("anthropickey") }),
                        globals: ResolvedGlobalProperties {
                            cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                            temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                            system: Some(ResolvedProperty {      source: ResolvedPropertySource::Variant("rust-coder".to_string()),                   value: String::from("rust-coder sys msg") }),
                            max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                            model: Some(ResolvedProperty {       source: ResolvedPropertySource::Inputs,                             value: "custom_model".to_string() }),
                            stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                        },
                    })
                ,
                model_info: Ok(ModelInfo {
                    provider: "anthropic".to_string(),
                    model: "custom_model".to_string()
                }),
                globals: ResolvedGlobalProperties {
                    cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                    temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                    system: Some(ResolvedProperty {      source: ResolvedPropertySource::Variant("rust-coder".to_string()),                   value: String::from("rust-coder sys msg") }),
                    max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                    model: Some(ResolvedProperty {       source: ResolvedPropertySource::Inputs,                             value: "custom_model".to_string() }),
                    stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                },
    }))
)]
#[case::variant_with_overriden_apikey(
  Some("override_apikey".to_string()),
  Ok(ResolvedConfig::Variant(Variant {
                base_name: "anthropic".to_string(),
                name: "override_apikey".to_string(),
                resolved:
                    ResolvedProviderConfig::Anthropic(anthropic::ResolvedProviderConfig {
                        api_key: Some(ResolvedProperty {         source: ResolvedPropertySource::Variant("override_apikey".to_string()),                 value: String::from("overridden_key") }),
                        globals: ResolvedGlobalProperties {
                            cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                            temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                            system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("anthropic".to_string()),                   value: String::from("system 5") }),
                            max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                            model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("anthropic".to_string()),                             value: "claude".to_string() }),
                            stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                        },
                    })
                ,
                model_info: Ok(ModelInfo {
                    provider: "anthropic".to_string(),
                    model: "claude".to_string()
                }),
                globals: ResolvedGlobalProperties {
                    cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                    temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                    system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("anthropic".to_string()),                   value: String::from("system 5") }),
                    max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                    model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("anthropic".to_string()),                             value: "claude".to_string() }),
                    stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                },
    }))
)]
#[case::variant_with_diffmodel(
  Some("rust-coder-diffmodel".to_string()),
  Ok(ResolvedConfig::Variant(Variant {
                base_name: "anthropic".to_string(),
                name: "rust-coder-diffmodel".to_string(),
                resolved:
                    ResolvedProviderConfig::Anthropic(anthropic::ResolvedProviderConfig {
                        api_key: Some(ResolvedProperty {         source: ResolvedPropertySource::Base("anthropic".to_string()),                 value: String::from("anthropickey") }),
                        globals: ResolvedGlobalProperties {
                            cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                            temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                            system: Some(ResolvedProperty {      source: ResolvedPropertySource::Variant("rust-coder-diffmodel".to_string()),                   value: String::from("rust-coder sys msg") }),
                            max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                            model: Some(ResolvedProperty {       source: ResolvedPropertySource::Variant("rust-coder-diffmodel".to_string()),                             value: "clauderust".to_string() }),
                            stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                        },
                    })
                ,
                model_info: Ok(ModelInfo {
                    provider: "anthropic".to_string(),
                    model: "clauderust".to_string()
                }),
                globals: ResolvedGlobalProperties {
                    cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                    temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                    system: Some(ResolvedProperty {      source: ResolvedPropertySource::Variant("rust-coder-diffmodel".to_string()),                   value: String::from("rust-coder sys msg") }),
                    max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                    model: Some(ResolvedProperty {       source: ResolvedPropertySource::Variant("rust-coder-diffmodel".to_string()),                             value: "clauderust".to_string() }),
                    stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                },
    }))
)]
#[case::none(
  None,
  Ok(ResolvedConfig::Base(Base {
                name: "openai".to_string(),
                resolved:
                    ResolvedProviderConfig::OpenAI(openai::ResolvedProviderConfig {
                        api_key: Some(ResolvedProperty {         source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("openaikey") }),
                        endpoint: Some(ResolvedProperty {        source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("openaiendpoint") }),
                        globals: ResolvedGlobalProperties {
                            cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                            temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                            system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("system 4") }),
                            max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()), value: 300 }),
                            model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("openai".to_string()),                     value: "gpt4".to_string() }),
                            stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Default,                                        value: DEFAULT_STREAM }),
                        },
                    })
                ,
                model_info: Ok(ModelInfo {
                    provider: "openai".to_string(),
                    model: "gpt4".to_string()
                }),
                globals: ResolvedGlobalProperties {
                    cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                    temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                    system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("system 4") }),
                    max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()), value: 300 }),
                    model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("openai".to_string()),                     value: "gpt4".to_string() }),
                    stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Default,                                        value: DEFAULT_STREAM }),
                },
    }))
)]
#[case::group_of_short_bases(
    Some("group_of_short_bases".to_string()),
    Ok(ResolvedConfig::Group(
            Group {
                name: "group_of_short_bases".to_string(),
                members: vec![
                    GroupMember::Base( Base {
                        name: "anthropic".to_string(),
                        resolved:
                            ResolvedProviderConfig::Anthropic(anthropic::ResolvedProviderConfig {
                                api_key: Some(ResolvedProperty {         source: ResolvedPropertySource::Base("anthropic".to_string()),                 value: String::from("anthropickey") }),
                                globals: ResolvedGlobalProperties {
                                    cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                                    temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                                    system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("anthropic".to_string()),                   value: String::from("system 5") }),
                                    max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                                    model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("anthropic".to_string()),                             value: "claude".to_string() }),
                                    stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                                },
                            })
                        ,
                        model_info: Ok(ModelInfo {
                            provider: "anthropic".to_string(),
                            model: "claude".to_string()
                        }),
                        globals: ResolvedGlobalProperties {
                            cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                            temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                            system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("anthropic".to_string()),                   value: String::from("system 5") }),
                            max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                            model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("anthropic".to_string()),                             value: "claude".to_string() }),
                            stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                        },
                        } , 1),
                    GroupMember::Base(Base {
                        name: "openai".to_string(),
                        resolved:
                            ResolvedProviderConfig::OpenAI(openai::ResolvedProviderConfig {
                                api_key: Some(ResolvedProperty {         source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("openaikey") }),
                                endpoint: Some(ResolvedProperty {        source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("openaiendpoint") }),
                                globals: ResolvedGlobalProperties {
                                    cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                                    temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                                    system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("system 4") }),
                                    max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()), value: 300 }),
                                    model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("openai".to_string()),                     value: "gpt4".to_string() }),
                                    stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Default,                                        value: DEFAULT_STREAM }),
                                },
                            })
                        ,
                        model_info: Ok(ModelInfo {
                            provider: "openai".to_string(),
                            model: "gpt4".to_string()
                        }),
                        globals: ResolvedGlobalProperties {
                            cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                            temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                            system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("system 4") }),
                            max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()), value: 300 }),
                            model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("openai".to_string()),                     value: "gpt4".to_string() }),
                            stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Default,                                        value: DEFAULT_STREAM }),
                        },
                    }, 1)
                ]
            }
    ))
)]
#[case::group_of_short_variants(
    Some("group_of_short_variants".to_string()),
    Ok(ResolvedConfig::Group(
            Group {
                name: "group_of_short_variants".to_string(),
                members: vec![
                    GroupMember::Variant(Variant {
                        base_name: "anthropic".to_string(),
                        name: "rust-coder-diffmodel".to_string(),
                        resolved:
                            ResolvedProviderConfig::Anthropic(anthropic::ResolvedProviderConfig {
                                api_key: Some(ResolvedProperty {         source: ResolvedPropertySource::Base("anthropic".to_string()),                 value: String::from("anthropickey") }),
                                globals: ResolvedGlobalProperties {
                                    cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                                    temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                                    system: Some(ResolvedProperty {      source: ResolvedPropertySource::Variant("rust-coder-diffmodel".to_string()),                   value: String::from("rust-coder sys msg") }),
                                    max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                                    model: Some(ResolvedProperty {       source: ResolvedPropertySource::Variant("rust-coder-diffmodel".to_string()),                             value: "clauderust".to_string() }),
                                    stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                                },
                            })
                        ,
                        model_info: Ok(ModelInfo {
                            provider: "anthropic".to_string(),
                            model: "clauderust".to_string()
                        }),
                        globals: ResolvedGlobalProperties {
                            cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                            temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                            system: Some(ResolvedProperty {      source: ResolvedPropertySource::Variant("rust-coder-diffmodel".to_string()),                   value: String::from("rust-coder sys msg") }),
                            max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                            model: Some(ResolvedProperty {       source: ResolvedPropertySource::Variant("rust-coder-diffmodel".to_string()),                             value: "clauderust".to_string() }),
                            stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                        },
                        } , 1),
                    GroupMember::Variant(Variant {
                        base_name: "anthropic".to_string(),
                        name: "rust-coder".to_string(),
                        resolved:
                            ResolvedProviderConfig::Anthropic(anthropic::ResolvedProviderConfig {
                                api_key: Some(ResolvedProperty {         source: ResolvedPropertySource::Base("anthropic".to_string()),                 value: String::from("anthropickey") }),
                                globals: ResolvedGlobalProperties {
                                    cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                                    temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                                    system: Some(ResolvedProperty {      source: ResolvedPropertySource::Variant("rust-coder".to_string()),                   value: String::from("rust-coder sys msg") }),
                                    max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                                    model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("anthropic".to_string()),                             value: "claude".to_string() }),
                                    stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                                },
                            })
                        ,
                        model_info: Ok(ModelInfo {
                            provider: "anthropic".to_string(),
                            model: "claude".to_string()
                        }),
                        globals: ResolvedGlobalProperties {
                            cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                            temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                            system: Some(ResolvedProperty {      source: ResolvedPropertySource::Variant("rust-coder".to_string()),                   value: String::from("rust-coder sys msg") }),
                            max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                            model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("anthropic".to_string()),                             value: "claude".to_string() }),
                            stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                        },
                    }, 1)
                ]
            }
    ))
)]
#[case::group_of_mixed_shorts(
    Some("group_of_mixed_shorts".to_string()),
    Ok(ResolvedConfig::Group(
            Group {
                name: "group_of_mixed_shorts".to_string(),
                members: vec![
                    GroupMember::Base(Base {
                        name: "openai".to_string(),
                        resolved:
                            ResolvedProviderConfig::OpenAI(openai::ResolvedProviderConfig {
                                api_key: Some(ResolvedProperty {         source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("openaikey") }),
                                endpoint: Some(ResolvedProperty {        source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("openaiendpoint") }),
                                globals: ResolvedGlobalProperties {
                                    cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                                    temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                                    system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("system 4") }),
                                    max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()), value: 300 }),
                                    model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("openai".to_string()),                     value: "gpt4".to_string() }),
                                    stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Default,                                        value: DEFAULT_STREAM }),
                                },
                            })
                        ,
                        model_info: Ok(ModelInfo {
                            provider: "openai".to_string(),
                            model: "gpt4".to_string()
                        }),
                        globals: ResolvedGlobalProperties {
                            cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                            temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                            system: Some(ResolvedProperty {      source: ResolvedPropertySource::Base("openai".to_string()),                     value: String::from("system 4") }),
                            max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Env("PROMPTCMD_OPENAI_MAX_TOKENS".to_string()), value: 300 }),
                            model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("openai".to_string()),                     value: "gpt4".to_string() }),
                            stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Default,                                        value: DEFAULT_STREAM }),
                        },
                    }, 1),
                    GroupMember::Variant(Variant {
                        base_name: "anthropic".to_string(),
                        name: "rust-coder".to_string(),
                        resolved:
                            ResolvedProviderConfig::Anthropic(anthropic::ResolvedProviderConfig {
                                api_key: Some(ResolvedProperty {         source: ResolvedPropertySource::Base("anthropic".to_string()),                 value: String::from("anthropickey") }),
                                globals: ResolvedGlobalProperties {
                                    cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                                    temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                                    system: Some(ResolvedProperty {      source: ResolvedPropertySource::Variant("rust-coder".to_string()),                   value: String::from("rust-coder sys msg") }),
                                    max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                                    model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("anthropic".to_string()),                             value: "claude".to_string() }),
                                    stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                                },
                            })
                        ,
                        model_info: Ok(ModelInfo {
                            provider: "anthropic".to_string(),
                            model: "claude".to_string()
                        }),
                        globals: ResolvedGlobalProperties {
                            cache_ttl: Some(ResolvedProperty {   source: ResolvedPropertySource::Inputs,                                         value: 80 }),
                            temperature: Some(ResolvedProperty { source: ResolvedPropertySource::Dotprompt("test".to_string()),                  value: 0.7 }),
                            system: Some(ResolvedProperty {      source: ResolvedPropertySource::Variant("rust-coder".to_string()),                   value: String::from("rust-coder sys msg") }),
                            max_tokens: Some(ResolvedProperty {  source: ResolvedPropertySource::Globals, value: 200 }),
                            model: Some(ResolvedProperty {       source: ResolvedPropertySource::Base("anthropic".to_string()),                             value: "claude".to_string() }),
                            stream: Some(ResolvedProperty {      source: ResolvedPropertySource::Env("PROMPTCMD_ANTHROPIC_STREAM".to_string()),             value: true }),
                        },
                    }, 1)
                ]
            }
    ))
)]
#[case(
  Some("group_with_missing_provider".to_string()), Err(ResolveError::GroupMemberNotFound("group_with_missing_provider".to_string(), "badname".to_string()))
)]
#[case::groups_cannot_be_indexed(
  Some("group_of_short_bases/openai".to_string()), Err(ResolveError::NotFound("group_of_short_bases/openai".to_string()))
)]
#[case(
  Some("google".to_string()), Err(ResolveError::NoNameToResolve)
)]
#[case(
  Some("badname".to_string()), Err(ResolveError::NotFound("badname".to_string()))
)]
    pub fn test_basic_resolution(
        #[case] resolve_name: Option<String>,
        #[case] expected_resolution: Result<ResolvedConfig, ResolveError>
    ) {

        let (env, appconfig, promptfile, overrides) = CONFIG_1;

        let appconfig = AppConfig::try_from(appconfig).unwrap();
        let promptfile = DotPrompt::try_from(promptfile).unwrap();

        // parse the env data into key value
        let env = env.trim().split("\n").map(|item| {
            item.split_once("=").map(|(k, v)| (k.trim().to_string(), Some(v.to_string()))).unwrap()
        }).collect::<Vec<_>>();

        temp_env::with_vars(env, || {
            let resolver = Resolver {
                overrides: Some(ResolvedGlobalProperties::from((&overrides, ResolvedPropertySource::Inputs))),
                fm_properties: Some(
                    ResolvedGlobalProperties::from(
                        (&GlobalProviderProperties::from(&promptfile.frontmatter), ResolvedPropertySource::Dotprompt("test".to_string()))
                    )
                )
            };
            let resolved = resolver.resolve(&appconfig, resolve_name);
            assert_eq!(resolved, expected_resolution);
        });
    }

    #[rstest]
    #[case::from_env_full(
r#"
PROMPTCMD_MODEL=openai/gpt5
"#,

r#"
"#,

r#"
---
---
Templ
"#,
        None,
        Some(ModelInfo {
            provider: "openai".to_string(),
            model: "gpt5".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Env("PROMPTCMD_MODEL".to_string()),
            value: "gpt5".to_string()
        }), ResolveType::Base
    )]
    #[case::from_env_short_no_default(
r#"
PROMPTCMD_MODEL=openai
"#,

r#"
"#,

r#"
---
---
Templ
"#,
        None,
        None, None, ResolveType::Fail
    )]
    #[case::from_env_short_with_default_in_base(
r#"
PROMPTCMD_MODEL=openai
"#,

r#"
[providers.openai]
model="gpt5"
"#,

r#"
---
---
Templ
"#,
        None,
        Some(ModelInfo {
            provider: "openai".to_string(),
            model: "gpt5".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Base("openai".to_string()),
            value: "gpt5".to_string()
        }),
        ResolveType::Base
    )]
    #[case::from_frontmatter_short(
r#"
PROMPTCMD_MODEL=anthropic
"#,

r#"
[providers]
default="anthropic"

[providers.openai]
model="gpt5"
"#,

r#"
---
model: openai
---
Templ
"#,
        None,
        Some(ModelInfo {
            provider: "openai".to_string(),
            model: "gpt5".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Base("openai".to_string()),
            value: "gpt5".to_string()
        }),
        ResolveType::Base
    )]
    #[case::from_frontmatter_fullbase(
r#"
PROMPTCMD_MODEL=anthropic
"#,

r#"
[providers]
default="anthropic"

[providers.openai]
model="gpt5"
"#,

r#"
---
model: ollama/gpt-oss:20b
---
Templ
"#,
        None,
        Some(ModelInfo {
            provider: "ollama".to_string(),
            model: "gpt-oss:20b".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Dotprompt("test".to_string()),
            value: "gpt-oss:20b".to_string()
        }),
        ResolveType::Base
    )]
    #[case::input_overrides_frontmatter(
r#"
PROMPTCMD_MODEL=anthropic
"#,

r#"
[providers]
default="anthropic"

[providers.openai]
model="gpt5"

[providers.openai.rust-coder]
"#,

r#"
---
model: rust-coder
---
Templ
"#,
        Some("ollama/somemodel".to_string()),
        Some(ModelInfo {
            provider: "ollama".to_string(),
            model: "somemodel".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Inputs,
            value: "somemodel".to_string()
        }),
        ResolveType::Base
    )]
    #[case::input_overrides_frontmatter_but_fails_if_no_default(
r#"
PROMPTCMD_MODEL=anthropic
"#,

r#"
[providers]
default="anthropic"

[providers.openai]
model="gpt5"

[providers.openai.rust-coder]
"#,

r#"
---
model: rust-coder
---
Templ
"#,
        Some("ollama".to_string()),
        None,
        None,
        ResolveType::Fail
    )]
    #[case::from_frontmatter_variant(
r#"
PROMPTCMD_MODEL=anthropic
"#,

r#"
[providers]
default="anthropic"

[providers.openai]
model="gpt5"

[providers.openai.rust-coder]
"#,

r#"
---
model: rust-coder
---
Templ
"#,
        None,
        Some(ModelInfo {
            provider: "openai".to_string(),
            model: "gpt5".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Base("openai".to_string()),
            value: "gpt5".to_string()
        }),
        ResolveType::Variant
    )]
    #[case::from_override_short(
r#"
"#,

r#"
[providers.openai]
model = "gpt4"
"#,

r#"
---
model: openrouter/anthropic/claude
---
Templ
"#,
        Some("openai".to_string()),
        Some(ModelInfo {
            provider: "openai".to_string(),
            model: "gpt4".to_string()
        }),
        Some(ResolvedProperty {
            // The source is Base (not Inputs) because inputs only gave the
            // provider name while the model itself what configured in base
            // config
            source: ResolvedPropertySource::Base("openai".to_string()),
            value: "gpt4".to_string()
        }), ResolveType::Base
    )]
    #[case::from_override_short_and_model_in_globals(
r#"
"#,

r#"
[providers]
model = "gpt4"

[providers.openai]
"#,

r#"
---
model: openrouter/anthropic/claude
---
Templ
"#,
        Some("openai".to_string()),
        Some(ModelInfo {
            provider: "openai".to_string(),
            model: "gpt4".to_string()
        }),
        Some(ResolvedProperty {
            // The source is Base (not Inputs) because inputs only gave the
            // provider name while the model itself what configured in base
            // config
            source: ResolvedPropertySource::Globals,
            value: "gpt4".to_string()
        }), ResolveType::Base
    )]
    // So should never use the model in FM something is given in cmd
    #[case::from_override_short_and_no_model(
r#"
"#,

r#"
[providers]

[providers.openai]
"#,

r#"
---
model: openrouter/anthropic/claude
---
Templ
"#,
        Some("openrouter".to_string()),
        None,
        None, ResolveType::Fail
    )]
    #[case::from_override_long(
r#"
"#,

r#"
[providers.openai]
model = "gpt4"
"#,

r#"
---
model: openrouter/anthropic/claude
---
Templ
"#,
        Some("openai/gpt5".to_string()),
        Some(ModelInfo {
            provider: "openai".to_string(),
            model: "gpt5".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Inputs,
            value: "gpt5".to_string()
        }), ResolveType::Base
    )]
    #[case::from_variant_override_short(
r#"
"#,

r#"
[providers.openai]
model = "gpt4"

[providers.openai.myvariant]
"#,

r#"
---
model: openrouter/anthropic/claude
---
Templ
"#,
        Some("myvariant".to_string()),
        Some(ModelInfo {
            provider: "openai".to_string(),
            model: "gpt4".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Base("openai".to_string()),
            value: "gpt4".to_string()
        }), ResolveType::Variant
    )]
    #[case::from_variant_override_short_and_model_in_variantconfig(
r#"
"#,

r#"
[providers.openai]
model = "gpt4"

[providers.openai.myvariant]
model = "gpt100"
"#,

r#"
---
model: openrouter/anthropic/claude
---
Templ
"#,
        Some("myvariant".to_string()),
        Some(ModelInfo {
            provider: "openai".to_string(),
            model: "gpt100".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Variant("myvariant".to_string()),
            value: "gpt100".to_string()
        }), ResolveType::Variant
    )]
    #[case::from_variant_override_long(
r#"
"#,

r#"
[providers.openai]
model = "gpt4"

[providers.openai.myvariant]
model = "gpt100"
"#,

r#"
---
model: openrouter/anthropic/claude
---
Templ
"#,
        Some("myvariant/gpt200".to_string()),
        Some(ModelInfo {
            provider: "openai".to_string(),
            model: "gpt200".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Inputs,
            value: "gpt200".to_string()
        }), ResolveType::Variant
    )]
    #[case::from_group_override(
r#"
"#,

r#"
[providers.openai]
model = "gpt4"

[providers.anthropic]
model = "claude"

[groups.mygroup]
providers = ["openai"]
"#,

r#"
---
model: openrouter/anthropic/claude
---
Templ
"#,
        Some("mygroup".to_string()),
        Some(ModelInfo {
            provider: "openai".to_string(),
            model: "gpt4".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Base("openai".to_string()),
            value: "gpt4".to_string()
        }), ResolveType::Group
    )]
    #[case::from_group_with_variants_override(
r#"
"#,

r#"
[providers.openai]
model = "gpt4"

[providers.anthropic]
model = "abc"

[providers.anthropic.coder]

[groups.mygroup]
providers = ["coder"]
"#,

r#"
---
model: openai/gpt4
---
Templ
"#,
        Some("mygroup".to_string()),
        Some(ModelInfo {
            provider: "anthropic".to_string(),
            model: "abc".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Base("anthropic".to_string()),
            value: "abc".to_string()
        }), ResolveType::Group
    )]
    #[case::from_group_with_variants_and_model_override(
r#"
"#,

r#"
[providers.openai]
model = "gpt4"

[providers.anthropic]
model = "abc"

[providers.anthropic.coder]

[groups.mygroup]
providers = ["coder/xyz"]
"#,

r#"
---
model: openai/gpt4
---
Templ
"#,
        Some("mygroup".to_string()),
        Some(ModelInfo {
            provider: "anthropic".to_string(),
            model: "xyz".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Group("mygroup".to_string(), "coder/xyz".to_string()),
            value: "xyz".to_string()
        }), ResolveType::Group
    )]
    pub fn test_model_sources(
        #[case] env: &str,
        #[case] appconfig: &str,
        #[case] dotprompt: &str,
        // How the command line (currently) overrides all configured models
        #[case] requested_model: Option<String>,
        #[case] modelinfo: Option<ModelInfo>,
        #[case] model: Option<ResolvedProperty<String>>,
        #[case] resolve_type: ResolveType
    ) {

        let appconfig = AppConfig::try_from(appconfig).unwrap();
        let dotprompt = DotPrompt::try_from(dotprompt).unwrap();

        // parse the env data into key value
        let env = if env.trim().is_empty() {
            Vec::new()
        } else {
            env.trim().split("\n").map(|item| {
                item.split_once("=").map(|(k, v)| (k.trim().to_string(), Some(v.to_string()))).unwrap()
            }).collect::<Vec<_>>()
        };

        temp_env::with_vars(env, || {
            let resolver = Resolver {
                overrides: None,
                fm_properties: Some(
                    ResolvedGlobalProperties::from(
                        (&GlobalProviderProperties::from(&dotprompt.frontmatter), ResolvedPropertySource::Dotprompt("test".to_string()))
                    )
                )
            };
            let resolved = resolver.resolve(&appconfig, requested_model);
            match resolved  {
                Ok(ResolvedConfig::Base(base)) => {
                    assert!(matches!(resolve_type, ResolveType::Base));
                    if let Some(modelinfo) = modelinfo {
                        assert_eq!(base.model_info.unwrap(), modelinfo);
                    } else {
                        assert!(base.model_info.is_err())
                    }

                    match base.resolved {
                        ResolvedProviderConfig::Ollama(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::Anthropic(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::OpenAI(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::Google(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::OpenRouter(conf) => assert_eq!(conf.globals.model, model),
                    }
                }
                Ok(ResolvedConfig::Variant(variant)) => {
                    assert!(matches!(resolve_type, ResolveType::Variant));
                    assert_eq!(variant.model_info.unwrap(), modelinfo.unwrap());
                    match variant.resolved {
                        ResolvedProviderConfig::Ollama(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::Anthropic(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::OpenAI(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::Google(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::OpenRouter(conf) => assert_eq!(conf.globals.model, model),
                    }
                }
                Ok(ResolvedConfig::Group(group)) => {
                    assert!(matches!(resolve_type, ResolveType::Group));
                    let member = group.members.first().unwrap();
                    match member {
                        GroupMember::Base(base, _) => {
                            if let Some(modelinfo) = modelinfo {
                                assert_eq!(base.model_info.as_ref().unwrap(), &modelinfo);
                            } else {
                                assert!(base.model_info.is_err())
                            }

                            match &base.resolved {
                                ResolvedProviderConfig::Ollama(conf) => assert_eq!(conf.globals.model, model),
                                ResolvedProviderConfig::Anthropic(conf) => assert_eq!(conf.globals.model, model),
                                ResolvedProviderConfig::OpenAI(conf) => assert_eq!(conf.globals.model, model),
                                ResolvedProviderConfig::Google(conf) => assert_eq!(conf.globals.model, model),
                                ResolvedProviderConfig::OpenRouter(conf) => assert_eq!(conf.globals.model, model),
                            }
                        }
                        GroupMember::Variant(variant, _) => {
                            assert_eq!(variant.model_info.as_ref().unwrap(), modelinfo.as_ref().unwrap());
                            match &variant.resolved {
                                ResolvedProviderConfig::Ollama(conf) => assert_eq!(conf.globals.model, model),
                                ResolvedProviderConfig::Anthropic(conf) => assert_eq!(conf.globals.model, model),
                                ResolvedProviderConfig::OpenAI(conf) => assert_eq!(conf.globals.model, model),
                                ResolvedProviderConfig::Google(conf) => assert_eq!(conf.globals.model, model),
                                ResolvedProviderConfig::OpenRouter(conf) => assert_eq!(conf.globals.model, model),
                            }
                        }
                    }
                }
                Err(_) => {
                    assert!(matches!(resolve_type, ResolveType::Fail));
                }
            }
        });
    }

    #[rstest]
    #[case::as_default_short(
r#"
"#,

r#"
[providers]
default = "openrouter"

[providers.openrouter]
model = "openai/gpt5"
"#,

r#"
---
---
Templ
"#,
        None,
        Some(ModelInfo {
            provider: "openrouter".to_string(),
            model: "openai/gpt5".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Base("openrouter".to_string()),
            value: "openai/gpt5".to_string()
        }), ResolveType::Base
    )]
    #[case::as_default_full(
r#"
"#,

r#"
[providers]
default = "openrouter/anthropic/claude"

[providers.openrouter]
model = "openai/gpt5"
"#,

r#"
---
---
Templ
"#,
        None,
        Some(ModelInfo {
            provider: "openrouter".to_string(),
            model: "anthropic/claude".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Globals,
            value: "anthropic/claude".to_string()
        }), ResolveType::Base
    )]
    #[case::as_default_env_short(
r#"
PROMPTCMD_MODEL=openrouter
"#,

r#"

[providers.openrouter]
model = "openai/gpt5"
"#,

r#"
---
---
Templ
"#,
        None,
        Some(ModelInfo {
            provider: "openrouter".to_string(),
            model: "openai/gpt5".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Base("openrouter".to_string()),
            value: "openai/gpt5".to_string()
        }), ResolveType::Base
    )]
    #[case::as_default_env_long(
r#"
PROMPTCMD_MODEL=openrouter/openai/gpt4
"#,

r#"

[providers.openrouter]
model = "openai/gpt5"
"#,

r#"
---
---
Templ
"#,
        None,
        Some(ModelInfo {
            provider: "openrouter".to_string(),
            model: "openai/gpt4".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Env("PROMPTCMD_MODEL".to_string()),
            value: "openai/gpt4".to_string()
        }), ResolveType::Base
    )]
    #[case::from_frontmatter_short(
r#"
"#,

r#"

[providers.openrouter]
model = "openai/gpt5"
"#,

r#"
---
model: openrouter
---
Templ
"#,
        None,
        Some(ModelInfo {
            provider: "openrouter".to_string(),
            model: "openai/gpt5".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Base("openrouter".to_string()),
            value: "openai/gpt5".to_string()
        }), ResolveType::Base
    )]
    #[case::from_frontmatter_long(
r#"
"#,

r#"

[providers.openrouter]
model = "openai/gpt5"
"#,

r#"
---
model: openrouter/anthropic/claude
---
Templ
"#,
        None,
        Some(ModelInfo {
            provider: "openrouter".to_string(),
            model: "anthropic/claude".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Dotprompt("test".to_string()),
            value: "anthropic/claude".to_string()
        }), ResolveType::Base
    )]
    #[case::from_frontmatter_long_withno_default(
r#"
"#,

r#"
"#,

r#"
---
model: openrouter/anthropic/claude
---
Templ
"#,
        None,
        Some(ModelInfo {
            provider: "openrouter".to_string(),
            model: "anthropic/claude".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Dotprompt("test".to_string()),
            value: "anthropic/claude".to_string()
        }), ResolveType::Base
    )]
    #[case::from_override_short(
r#"
"#,

r#"
[providers.openrouter]
model = "random/model"
"#,

r#"
---
model: openrouter/anthropic/claude
---
Templ
"#,
        Some("openrouter".to_string()),
        Some(ModelInfo {
            provider: "openrouter".to_string(),
            model: "random/model".to_string()
        }),
        Some(ResolvedProperty {
            // The source is Base (not Inputs) because inputs only gave the
            // provider name while the model itself what configured in base
            // config
            source: ResolvedPropertySource::Base("openrouter".to_string()),
            value: "random/model".to_string()
        }), ResolveType::Base
    )]
    #[case::from_override_long(
r#"
"#,

r#"
[providers.openrouter]
model = "random/model"
"#,

r#"
---
model: openrouter/anthropic/claude
---
Templ
"#,
        Some("openrouter/random2/model2".to_string()),
        Some(ModelInfo {
            provider: "openrouter".to_string(),
            model: "random2/model2".to_string()
        }),
        Some(ResolvedProperty {
            source: ResolvedPropertySource::Inputs,
            value: "random2/model2".to_string()
        }), ResolveType::Base
    )]
    pub fn test_openrouter(
        #[case] env: &str,
        #[case] appconfig: &str,
        #[case] dotprompt: &str,
        // How the command line (currently) overrides all configured models
        #[case] requested_model: Option<String>,
        #[case] modelinfo: Option<ModelInfo>,
        #[case] model: Option<ResolvedProperty<String>>,
        #[case] resolve_type: ResolveType
    ) {

        let appconfig = AppConfig::try_from(appconfig).unwrap();
        let dotprompt = DotPrompt::try_from(dotprompt).unwrap();

        // parse the env data into key value
        let env = if env.trim().is_empty() {
            Vec::new()
        } else {
            env.trim().split("\n").map(|item| {
                item.split_once("=").map(|(k, v)| (k.trim().to_string(), Some(v.to_string()))).unwrap()
            }).collect::<Vec<_>>()
        };

        temp_env::with_vars(env, || {
            let resolver = Resolver {
                overrides: None,
                fm_properties: Some(
                    ResolvedGlobalProperties::from(
                        (&GlobalProviderProperties::from(&dotprompt.frontmatter), ResolvedPropertySource::Dotprompt("test".to_string()))
                    )
                )
            };
            let resolved = resolver.resolve(&appconfig, requested_model);
            match resolved  {
                Ok(ResolvedConfig::Base(base)) => {
                    assert!(matches!(resolve_type, ResolveType::Base));
                    if let Some(modelinfo) = modelinfo {
                        assert_eq!(base.model_info.unwrap(), modelinfo);
                    } else {
                        assert!(base.model_info.is_err())
                    }

                    match base.resolved {
                        ResolvedProviderConfig::Ollama(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::Anthropic(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::OpenAI(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::Google(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::OpenRouter(conf) => assert_eq!(conf.globals.model, model),
                    }
                }
                Ok(ResolvedConfig::Variant(variant)) => {
                    assert!(matches!(resolve_type, ResolveType::Variant));
                    assert_eq!(variant.model_info.unwrap(), modelinfo.unwrap());
                    match variant.resolved {
                        ResolvedProviderConfig::Ollama(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::Anthropic(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::OpenAI(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::Google(conf) => assert_eq!(conf.globals.model, model),
                        ResolvedProviderConfig::OpenRouter(conf) => assert_eq!(conf.globals.model, model),
                    }
                }
                Ok(ResolvedConfig::Group(_)) => {
                    assert!(matches!(resolve_type, ResolveType::Group));
                }
                Err(_) => {
                    assert!(matches!(resolve_type, ResolveType::Fail));
                }
            }
        });
    }
}
