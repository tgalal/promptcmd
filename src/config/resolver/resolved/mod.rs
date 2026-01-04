// pub mod ollama;
// pub mod anthropic;
// pub mod openai;
use thiserror::Error;

pub const DEFAULT_MAX_TOKENS: u32 = 1000;
pub const DEFAULT_STREAM: bool = false;
pub const DEFAULT_TEMPERATURE: f32 = 1.0;
pub const DEFAULT_SYSTEM: &str = "You are useful AI assistant. Give me brief answers. Do not use special formatting like markdown.";

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub provider: String,
    pub model: String
}

#[derive(Error, Debug)]
pub enum ToLLMBuilderError {
    #[error("'{0}' is required but not configured")]
    RequiredConfiguration(&'static str),
    #[error("{0}")]
    ModelError(#[from] ToModelInfoError)
}

#[derive(Error, Debug, Clone)]
#[error("'{0}' is required but not configured")]
pub enum ToModelInfoError {
    RequiredConfiguration(&'static str)
}

#[macro_export]
macro_rules! define_resolved_provider_config {
    ($provider_name:literal { $($field:ident : $type:ty),* $(,)? }) => {

        #[derive(Debug, Deserialize, Default)]
        pub struct Providers {

            #[serde(flatten)]
            pub config: Config,

            #[serde(flatten)]
            pub named: std::collections::HashMap<String, Config>,
        }

        #[derive(Debug, Deserialize, Default)]
        pub struct Config {
            pub temperature: Option<f32>,
            pub system: Option<String>,
            pub stream: Option<bool>,
            pub max_tokens: Option<u32>,
            pub default_model: Option<String>,

            $(pub $field: Option<$type>),*
        }

        #[derive(Debug, Default)]
        pub struct ResolvedProviderConfigBuilder {
            // Shared fields
            pub temperature: Option<ResolvedProperty<f32>>,
            pub system: Option<ResolvedProperty<String>>,
            // stream: Option<ResolvedProperty<bool>>,
            // max_tokens: Option<ResolvedProperty<u32>>,
            pub model: Option<ResolvedProperty<String>>,
            // Custom fields
            $(pub $field: Option<ResolvedProperty<$type>>),*
        }

        pub struct ResolvedProviderConfig {
            // Shared fields
            pub temperature: Option<ResolvedProperty<f32>>,
            pub system: Option<ResolvedProperty<String>>,
            // stream: Option<ResolvedProperty<bool>>,
            // max_tokens: Option<ResolvedProperty<u32>>,
            pub model: Option<ResolvedProperty<String>>,
            // Custom fields
            $(pub $field: Option<ResolvedProperty<$type>>),*
        }

        impl fmt::Debug for ResolvedProviderConfig {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "temperature: ")?;
                if let Some(val) = &self.temperature {
                    write!(f, "{} (source: {})", val, val.source)?;
                }
                write!(f, "\nsystem: ")?;
                if let Some(val) = &self.system {
                    if val.value.len() > 50 {
                        write!(f, "{:.50}... (source: {})", val.value, val.source)?;
                    } else {
                        write!(f, "{}... (source: {})", val, val.source)?;
                    }
                }
                write!(f, "\nmodel: ")?;
                if let Some(val) = &self.model {
                    write!(f, "{} (source: {})", val, val.source)?;
                }

                $(
                write!(f, "\n{}: ", stringify!($field))?;
                if let Some(val) = &self.$field {
                    write!(f, "{} (source: {})", val, val.source)?;
                }
                );*
                // writeln!(f, "temperature={}", self.temperature.as_ref()
                //     .map_or("".to_string(),|v| v.value.to_string()))?;
                Ok(())
            }
        }

        impl fmt::Display for ResolvedProviderConfig {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "temperature: ")?;
                if let Some(val) = &self.temperature {
                    write!(f, "{} [source: {}]", val, val.source)?;
                }
                write!(f, "\nsystem: ")?;
                if let Some(val) = &self.system {
                    if val.value.len() > 50 {
                        write!(f, "{:.50}... [source: {}]", val.value, val.source)?;
                    } else {
                        write!(f, "{}... [source: {}]", val, val.source)?;
                    }
                }
                write!(f, "\nmodel: ")?;
                if let Some(val) = &self.model {
                    write!(f, "{} [source: {}]", val, val.source)?;
                }
                $(
                write!(f, "\n{}: ", stringify!($field))?;
                if let Some(val) = &self.$field {
                    if "api_key" == stringify!($field) && val.value.len() > 15 {
                        write!(f, "{:.15}xxxxx...redacted [source: {}]", val.value, val.source)?;
                    } else {
                        write!(f, "{} [source: {}]", val, val.source)?;
                    }
                }
                );*
                // writeln!(f, "temperature={}", self.temperature.as_ref()
                //     .map_or("".to_string(),|v| v.value.to_string()))?;
                Ok(())
            }
        }

        impl ResolvedProviderConfigBuilder {
            pub fn new(
            ) -> Self {
                Self {
                    temperature: None,
                    system: None,
                    model: None,
                    $($field: None),*
                }
            }

            pub fn override_model(mut self, model: Option<ResolvedProperty<String>>) -> Self {
                self.model = model.or(self.model);

                self
            }

            pub fn override_from(mut self, other: &ResolvedProviderConfig) -> Self {
                self.temperature = other.temperature.clone().or(self.temperature);
                self.system = other.system.clone().or(self.system);

                $(self.$field = other.$field.clone().or(self.$field));*;

                self
            }

            fn apply_env(mut self) -> Self {

                fn read_env<T: std::str::FromStr>(key: &str, def: Option<ResolvedProperty<T>>) -> Option<ResolvedProperty<T>> {
                    let env_prefix = String::from("PROMPTCMD_")
                        + $provider_name.to_uppercase().as_str();
                    let env_field_name = env_prefix.clone() + "_" + key;
                    debug!("READING {}", env_field_name);
                    let env_field_value = env::var(&env_field_name).ok().map(|value| {
                        debug!("{}={}", env_field_name, value);
                        value.parse().ok()
                    })
                    .flatten()
                    .map(|value| {
                        ResolvedProperty {
                            source: ResolvedPropertySource::Env(env_field_name),
                            value
                        }
                    })
                    .or(def);
                    env_field_value
                }

                self.temperature = read_env("TEMPERATURE", self.temperature);
                self.system = read_env("SYSTEM", self.system);
                self.model = read_env("MODEL", self.model);

                $(self.$field = read_env(&stringify!($field).to_uppercase(), self.$field));*;
                self
            }

            fn apply_default(mut self) -> Self {
                self.temperature = self.temperature.or(
                    Some(ResolvedProperty {
                        source: ResolvedPropertySource::Default,
                        value: resolved::DEFAULT_TEMPERATURE
                    })
                );
                self.system = self.system.or(
                    Some(ResolvedProperty {
                        source: ResolvedPropertySource::Default,
                        value: resolved::DEFAULT_SYSTEM.into()
                    })
                );

                self
            }

            pub fn build(self) -> ResolvedProviderConfig {
                let finalized_builder = self.apply_default().apply_env();
                ResolvedProviderConfig {
                    temperature: finalized_builder.temperature,
                    system: finalized_builder.system,
                    model: finalized_builder.model,
                    $($field: finalized_builder.$field),*
                }
            }
        }

        impl From<ResolvedProviderConfigBuilder> for ResolvedProviderConfig {
            fn from(builder: ResolvedProviderConfigBuilder) -> Self {
                builder.build()
            }
        }

        impl From<(&Config, ResolvedPropertySource)> for ResolvedProviderConfigBuilder {
            fn from(tuple: (&Config, ResolvedPropertySource)) -> Self {
                let source_config = tuple.0;
                let source = tuple.1;
                Self {
                    temperature: source_config.temperature.as_ref().map(|value| ResolvedProperty {
                        source: source.clone(),
                        value: value.clone()
                    }),
                    system: source_config.system.as_ref().map(|value| ResolvedProperty {
                        source:source.clone(),
                        value: value.clone()
                    }),
                    model: source_config.default_model.as_ref().map(|value| ResolvedProperty {
                        source:source.clone(),
                        value: value.clone()
                    }),
                    $($field: source_config.$field.as_ref().map(|value| ResolvedProperty {
                        source: source.clone(),
                        value: value.clone()
                    })),*
                }
            }
        }

        impl TryFrom<&ResolvedProviderConfig> for ModelInfo {
            type Error = ToModelInfoError;
            fn try_from(config: &ResolvedProviderConfig) -> Result<Self, Self::Error> {
                let provider_name = stringify!($source_config).replace("Config", "").to_lowercase();
                config.model.as_ref()
                    .map(|property| ModelInfo { provider: provider_name, model: property.value.clone() })
                    .ok_or(ToModelInfoError::RequiredConfiguration("model"))
            }
        }
    };
}


