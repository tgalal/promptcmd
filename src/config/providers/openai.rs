use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct OpenAIProviders {

    #[serde(flatten)]
    pub config: OpenAIConfig,

    #[serde(flatten)]
    pub named: std::collections::HashMap<String, OpenAIConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct OpenAIConfig {
    pub temperature: Option<f32>,
    pub system: Option<String>,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,
    pub default_model: Option<String>,

    #[serde(default)]
    pub endpoint: Option<String>,

    #[serde(default)]
    pub api_key: Option<String>,
}
