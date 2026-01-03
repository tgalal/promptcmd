use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct AnthropicProviders {

    #[serde(flatten)]
    pub config: AnthropicConfig,

    #[serde(flatten)]
    pub named: std::collections::HashMap<String, AnthropicConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AnthropicConfig {
    pub temperature: Option<f32>,
    pub system: Option<String>,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,
    pub default_model: Option<String>,

    pub api_key: Option<String>
}
