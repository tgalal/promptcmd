use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct OllamaProviders {
    #[serde(flatten)]
    pub config: OllamaConfig,

    #[serde(flatten)]
    pub named: std::collections::HashMap<String, OllamaConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct OllamaConfig {
    pub temperature: Option<f32>,
    pub system: Option<String>,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,
    pub default_model: Option<String>,

    #[serde(default)]
    pub endpoint: Option<String>
}
