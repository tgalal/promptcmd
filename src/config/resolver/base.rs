use llm::builder::LLMBuilder;

use crate::config::resolver::{BaseProviderConfigSource, ResolvedGlobalProperties, ResolvedPropertySource};
use crate::config::{providers::{error::ToModelInfoError, ModelInfo}, resolver::{ResolvedProperty,
    ResolvedProviderConfig}};
use crate::config::providers;

#[derive(Debug, PartialEq)]
pub struct Base {
    pub name: String,
    pub resolved: ResolvedProviderConfig,
    pub model_info: Result<ModelInfo, ToModelInfoError>,
    pub globals: ResolvedGlobalProperties
}

impl Base {
    pub fn new(
        name: String,
        source: BaseProviderConfigSource,
        fm_properties: Option<ResolvedGlobalProperties>,
        overrides: Option<ResolvedGlobalProperties>,
        global_provider_properties: ResolvedGlobalProperties,
        model_resolved_property: Option<ResolvedProperty<String>>) -> Self {

        macro_rules! resolve_final_config {
            ($provider:ident, $base_config:ident) => {

            {
                providers::$provider::ResolvedProviderConfigBuilder::from_defaults()
                    .apply_providers_env()
                    .apply_global_overrides(Some(global_provider_properties))
                    .apply_env()
                    .override_from(
                        &providers::$provider::ResolvedProviderConfigBuilder::from(
                            ($base_config, ResolvedPropertySource::Base(name.clone()))).build()
                    )                .apply_global_overrides(fm_properties)
                    .apply_global_overrides(overrides)
                    .override_model(model_resolved_property)
                    .build()
            }
            }
        }


        let (globals, model_info, resolved) = match source {
            BaseProviderConfigSource::Ollama(source_config) => {
                let resolved = resolve_final_config!(ollama, source_config);

                (resolved.globals.clone(), ModelInfo::try_from(&resolved),ResolvedProviderConfig::Ollama(resolved))
            },

            BaseProviderConfigSource::Anthropic(source_config) => {
                let resolved = resolve_final_config!(anthropic, source_config);

                (resolved.globals.clone(), ModelInfo::try_from(&resolved),ResolvedProviderConfig::Anthropic(resolved))

            },
            BaseProviderConfigSource::OpenAI(source_config) => {
                let resolved = resolve_final_config!(openai, source_config);

                (resolved.globals.clone(), ModelInfo::try_from(&resolved),ResolvedProviderConfig::OpenAI(resolved))
            },
            BaseProviderConfigSource::Google(source_config) => {
                let resolved = resolve_final_config!(google, source_config);

                (resolved.globals.clone(), ModelInfo::try_from(&resolved),ResolvedProviderConfig::Google(resolved))
            },
            BaseProviderConfigSource::OpenRouter(source_config) => {
                let resolved = resolve_final_config!(openrouter, source_config);

                (resolved.globals.clone(), ModelInfo::try_from(&resolved),ResolvedProviderConfig::OpenRouter(resolved))
            },
        };
        Self {
            name,
            resolved,
            model_info,
            globals,
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
