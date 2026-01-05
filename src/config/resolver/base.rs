use llm::builder::LLMBuilder;

use crate::config::resolver::{BaseProviderConfigSource, ResolvedPropertySource};
use crate::config::{providers::{error::ToModelInfoError, ModelInfo}, resolver::{ResolvedProperty,
    ResolvedProviderConfig}};
use crate::config::providers;

#[derive(Debug)]
pub struct Base {
    pub name: String,
    pub resolved: ResolvedProviderConfig,
    pub model_info: Result<ModelInfo, ToModelInfoError>
}

impl Base {
    pub fn new(
        name: String,
        source: BaseProviderConfigSource,
        model_resolved_property: Option<ResolvedProperty<String>>) -> Self {

        let (model_info, resolved) = match source {
            BaseProviderConfigSource::Ollama(source_config) => {
                let resolved = providers::ollama::ResolvedProviderConfigBuilder::from(
                        (source_config, ResolvedPropertySource::Base(name.clone()))
                    ).override_model(model_resolved_property).build();
                (ModelInfo::try_from(&resolved),ResolvedProviderConfig::Ollama(resolved))
            },

            BaseProviderConfigSource::Anthropic(source_config) => {
                let resolved = providers::anthropic::ResolvedProviderConfigBuilder::from(
                        (source_config, ResolvedPropertySource::Base(name.clone()))
                    ).override_model(model_resolved_property).build();
                (ModelInfo::try_from(&resolved),ResolvedProviderConfig::Anthropic(resolved))

            },
            BaseProviderConfigSource::OpenAI(source_config) => {
                let resolved = providers::openai::ResolvedProviderConfigBuilder::from(
                        (source_config, ResolvedPropertySource::Base(name.clone()))
                    ).override_model(model_resolved_property).build();
                (ModelInfo::try_from(&resolved),ResolvedProviderConfig::OpenAI(resolved))
            },
            BaseProviderConfigSource::Google(source_config) => {
                let resolved = providers::google::ResolvedProviderConfigBuilder::from(
                        (source_config, ResolvedPropertySource::Base(name.clone()))
                    ).override_model(model_resolved_property).build();
                (ModelInfo::try_from(&resolved),ResolvedProviderConfig::Google(resolved))
            },
            BaseProviderConfigSource::OpenRouter(source_config) => {
                let resolved = providers::openrouter::ResolvedProviderConfigBuilder::from(
                        (source_config, ResolvedPropertySource::Base(name.clone()))
                    ).override_model(model_resolved_property).build();
                (ModelInfo::try_from(&resolved),ResolvedProviderConfig::OpenRouter(resolved))
            },
        };
        Self {
            name,
            resolved,
            model_info
        }
    }
}

impl TryFrom<&Base> for (ModelInfo, LLMBuilder) {
    type Error = providers::error::ToLLMBuilderError;

    fn try_from(base: &Base) -> std::result::Result<Self, Self::Error> {
        match &base.resolved {
            ResolvedProviderConfig::Ollama(resolved) => {
                let model_info = ModelInfo::try_from(resolved)?;
                let llmbuilder = LLMBuilder::try_from(resolved)?;
                Ok((model_info, llmbuilder))
            }
            ResolvedProviderConfig::Anthropic(resolved) =>
                Ok((ModelInfo::try_from(resolved)?, LLMBuilder::try_from(resolved)?)),
            ResolvedProviderConfig::OpenAI(resolved) =>
                Ok((ModelInfo::try_from(resolved)?, LLMBuilder::try_from(resolved)?)),
            ResolvedProviderConfig::Google(resolved) =>
                Ok((ModelInfo::try_from(resolved)?, LLMBuilder::try_from(resolved)?)),
            ResolvedProviderConfig::OpenRouter(resolved) =>
                Ok((ModelInfo::try_from(resolved)?, LLMBuilder::try_from(resolved)?)),
        }
    }
}
