use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OpenAIProviders {

    #[serde(flatten)]
    pub config: OpenAIConfig,
    
    #[serde(flatten)]
    pub named: std::collections::HashMap<String, OpenAIConfig>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct OpenAIConfig {
    temperature: Option<f32>,
    system: Option<String>,
    stream: Option<bool>,
    max_tokens: Option<u32>,

    #[serde(default)]
    pub endpoint: String
}
