mod provider;
pub mod error;
pub mod ollama;
pub mod anthropic;
pub mod openai;
pub mod google;
pub mod openrouter;
pub mod constants;


#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub provider: String,
    pub model: String
}
