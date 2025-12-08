use serde::Deserialize;
use serde::Serialize;
use toml;
use toml::de::Error as TomlError;
use thiserror::Error;
use crate::config::providers;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    default_model: String,
    editor: String,
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
