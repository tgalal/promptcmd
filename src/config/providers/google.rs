use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct GoogleProviders {

    #[serde(flatten)]
    pub config: GoogleConfig,

    #[serde(flatten)]
    pub named: std::collections::HashMap<String, GoogleConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct GoogleConfig {
    temperature: Option<f32>,
    system: Option<String>,
    stream: Option<bool>,
    max_tokens: Option<u32>,
    default_model: Option<String>,
    api_key: Option<String>
}
