#[macro_export]
macro_rules! create_provider {

    ($provider_name:literal { $($field:ident : $type:ty),* $(,)? }) => {
        create_provider!(@internal $provider_name { $($field : $type),* };
            temperature: f32,
            system: String,
            stream: bool,
            max_tokens: u32,
            model: String
        );
    };

    (@internal $provider_name:literal { $($field:ident : $type:ty),* }; $($global_field:ident : $global_field_type:ty),*) => {
        use $crate::config::resolver::ResolvedProperty;
        use $crate::config::resolver::ResolvedPropertySource;
        use $crate::config::providers::constants;
        use $crate::config::providers::ModelInfo;
        use $crate::config::providers::error;
        use $crate::config::appconfig::GlobalProviderProperties;


        // Serialized formats for reading configuration files
        #[derive(Debug, Deserialize, Default)]
        pub struct Providers {
            #[serde(flatten)]
            pub config: Config,

            #[serde(flatten)]
            pub named: std::collections::HashMap<String, Config>,
        }

        #[derive(Debug, Deserialize, Default)]
        pub struct Config {
            $(pub $global_field: Option<$global_field_type>),*,
            $(pub $field: Option<$type>),*
        }

        // Resolution Structs
        // Builder enables overriding
        #[derive(Debug, Default)]
        pub struct ResolvedProviderConfigBuilder {

            $(pub $global_field: Option<ResolvedProperty<$global_field_type>>),*,
            $(pub $field: Option<ResolvedProperty<$type>>),*
        }

        // Finalized Configuration with all sources
        #[derive(Debug)]
        pub struct ResolvedProviderConfig {
            $(pub $global_field: Option<ResolvedProperty<$global_field_type>>),*,
            $(pub $field: Option<ResolvedProperty<$type>>),*
        }

        impl ResolvedProviderConfig {
            pub fn from_env_globals() -> Self {
                let mut builder = ResolvedProviderConfigBuilder::new();

                $(
                    builder.$global_field = read_env(
                        &stringify!($global_field_type).to_uppercase(),
                        true, None
                    );
                )*

                builder.build()
            }
        }

        fn read_env<T: std::str::FromStr>(
            key: &str,
            global: bool,
            def: Option<ResolvedProperty<T>>) -> Option<ResolvedProperty<T>> {
            let mut env_field_name_builder = vec!["PROMPTCMD"];
            let provider_prefix = $provider_name.to_uppercase();

            if !global {
                env_field_name_builder.push(&provider_prefix);
            }

            env_field_name_builder.push(key);

            let env_field_name = env_field_name_builder.join("_");
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

        impl fmt::Display for ResolvedProviderConfig {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

                let mut first = true;

                $(
                    if !first {
                        writeln!(f)?;
                    }
                    first = false;
                    write!(f, stringify!($global_field))?;
                    write!(f, ": ")?;

                    if let Some(val) = &self.$global_field {
                        let value = val.to_string();
                        if value.len() > 50 {
                            write!(f, "{:.50}... [source: {}]", value, val.source)?;
                        } else {
                            write!(f, "{} [source: {}]", value, val.source)?;
                        }
                    }
                );*

                $(
                write!(f, "\n{}: ", stringify!($field))?;
                if let Some(val) = &self.$field {
                    if "api_key" == stringify!($field) && val.value.len() > 0 {
                        write!(f, "xxxxx...redacted [source: {}]", val.source)?;
                    } else {
                        write!(f, "{} [source: {}]", val, val.source)?;
                    }
                }
                );*
                Ok(())
            }
        }

        impl ResolvedProviderConfigBuilder {
            pub fn new(
            ) -> Self {
                Self {
                    $($global_field: None),*,
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
                $(
                    self.$global_field = read_env(&stringify!($global_field).to_uppercase(), false, self.$global_field);
                )*

                $(self.$field = read_env(&stringify!($field).to_uppercase(), false, self.$field));*;
                self
            }

            fn apply_default(mut self) -> Self {
                self.temperature = self.temperature.or(
                    Some(ResolvedProperty {
                        source: ResolvedPropertySource::Default,
                        value: constants::DEFAULT_TEMPERATURE
                    })
                );
                self.system = self.system.or(
                    Some(ResolvedProperty {
                        source: ResolvedPropertySource::Default,
                        value: constants::DEFAULT_SYSTEM.into()
                    })
                );

                self
            }

            pub fn build(self) -> ResolvedProviderConfig {
                let finalized_builder = self.apply_default().apply_env();
                ResolvedProviderConfig {
                    $($global_field: finalized_builder.$global_field),*,
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
                    $(
                        $global_field: source_config.$global_field.as_ref().map(|value| ResolvedProperty {
                            source: source.clone(),
                            value: value.clone()
                        })
                    ),*,
                    $($field: source_config.$field.as_ref().map(|value| ResolvedProperty {
                        source: source.clone(),
                        value: value.clone()
                    })),*
                }
            }
        }

        impl From<&GlobalProviderProperties> for ResolvedProviderConfig {
            fn from(globals: &GlobalProviderProperties) -> Self {
                let mut builder = ResolvedProviderConfigBuilder::new();
                $(
                    builder.$global_field = globals.$global_field.as_ref().map(|value| ResolvedProperty {
                        source: ResolvedPropertySource::Globals,
                        value: value.clone()
                    });
                )*
                builder.build()
            }
        }

        impl TryFrom<&ResolvedProviderConfig> for ModelInfo {
            type Error = error::ToModelInfoError;
            fn try_from(config: &ResolvedProviderConfig) -> Result<Self, Self::Error> {
                config.model.as_ref()
                    .map(|property| ModelInfo { provider: $provider_name.into(), model: property.value.clone() })
                    .ok_or(error::ToModelInfoError::RequiredConfiguration("model"))
            }
        }
    };
}


