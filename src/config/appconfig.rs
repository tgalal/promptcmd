use serde::Deserialize;

#[derive(Deserialize)]
pub struct ProviderConfig {
    pub temperature: Option<u32>,
    pub endpoint: String,
    pub api_key: Option<String>
}

#[derive(Deserialize)]
pub struct AppConfig {
    pub providers: Vec<ProviderConfig>
}
