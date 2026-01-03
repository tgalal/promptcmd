use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct OpenRouterProviders  {

    #[serde(flatten)]
    pub config: OpenRouterConfig,

    #[serde(flatten)]
    pub named: std::collections::HashMap<String, OpenRouterConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct OpenRouterConfig {
    temperature: Option<f32>,
    system: Option<String>,
    stream: Option<bool>,
    max_tokens: Option<u32>,
    default_model: Option<String>,

    api_key: Option<String>
}
