use llm::builder::LLMBuilder;

use crate::config::{appconfig::GlobalProviderProperties, providers::{error::ToModelInfoError, ModelInfo}, resolver::{ResolvedProperty, ResolvedPropertySource, ResolvedProviderConfig, VariantProviderConfigSource}};
use crate::config::providers;

#[derive(Debug)]
pub struct Variant {
    pub name: String,
    pub base_name: String,
    pub resolved: ResolvedProviderConfig,
    pub model_info: Result<ModelInfo, ToModelInfoError>,
    pub cache_ttl: Option<ResolvedProperty<u32>>
}

impl Variant {
    pub fn new(
        name: String,
        base_name: String,
        source: VariantProviderConfigSource,
        global_provider_properties: &GlobalProviderProperties,
        model_resolved_property: Option<ResolvedProperty<String>>) -> Self {

        let (cache_ttl, model_info, resolved) = match source {
            VariantProviderConfigSource::Ollama(variant_config, base_config) => {
                let resolved = providers::ollama::ResolvedProviderConfigBuilder::from(
                    global_provider_properties
                ).override_from(
                    &providers::ollama::ResolvedProviderConfigBuilder::from(
                    (base_config, ResolvedPropertySource::Base(base_name.clone()))
                ).build()).override_from(
                        &providers::ollama::ResolvedProviderConfigBuilder::from((variant_config, ResolvedPropertySource::Variant(name.clone())))
                            .build()
                ).override_model(model_resolved_property).apply_env().apply_default().build();

                (resolved.cache_ttl.clone(), ModelInfo::try_from(&resolved), ResolvedProviderConfig::Ollama(resolved))
            },
            VariantProviderConfigSource::Anthropic(variant_config, base_config) => {
                let resolved = providers::anthropic::ResolvedProviderConfigBuilder::from(
                    global_provider_properties
                ).override_from(
                    &providers::anthropic::ResolvedProviderConfigBuilder::from(
                    (base_config, ResolvedPropertySource::Base(base_name.clone()))
                ).build()).override_from(
                        &providers::anthropic::ResolvedProviderConfigBuilder::from((variant_config, ResolvedPropertySource::Variant(name.clone())))
                            .build()
                ).override_model(model_resolved_property).apply_env().apply_default().build();

                (resolved.cache_ttl.clone(), ModelInfo::try_from(&resolved), ResolvedProviderConfig::Anthropic(resolved))
            },
            VariantProviderConfigSource::OpenAI(variant_config, base_config) => {
                let resolved = providers::openai::ResolvedProviderConfigBuilder::from(
                    global_provider_properties
                ).override_from(
                    &providers::openai::ResolvedProviderConfigBuilder::from(
                    (base_config, ResolvedPropertySource::Base(base_name.clone()))
                ).build()).override_from(
                        &providers::openai::ResolvedProviderConfigBuilder::from((variant_config, ResolvedPropertySource::Variant(name.clone())))
                            .build()
                ).override_model(model_resolved_property).apply_env().apply_default().build();

                (resolved.cache_ttl.clone(), ModelInfo::try_from(&resolved), ResolvedProviderConfig::OpenAI(resolved))
            },
            VariantProviderConfigSource::Google(variant_config, base_config) => {
                let resolved = providers::google::ResolvedProviderConfigBuilder::from(
                    global_provider_properties
                ).override_from(
                    &providers::google::ResolvedProviderConfigBuilder::from(
                    (base_config, ResolvedPropertySource::Base(base_name.clone()))
                ).build()).override_from(
                        &providers::google::ResolvedProviderConfigBuilder::from((variant_config, ResolvedPropertySource::Variant(name.clone())))
                            .build()
                ).override_model(model_resolved_property).apply_env().apply_default().build();

                (resolved.cache_ttl.clone(), ModelInfo::try_from(&resolved), ResolvedProviderConfig::Google(resolved))
            },
            VariantProviderConfigSource::OpenRouter(variant_config, base_config) => {
                let resolved = providers::openrouter::ResolvedProviderConfigBuilder::from(
                    global_provider_properties
                ).override_from(
                    &providers::openrouter::ResolvedProviderConfigBuilder::from(
                    (base_config, ResolvedPropertySource::Base(base_name.clone()))
                ).build()).override_from(
                        &providers::openrouter::ResolvedProviderConfigBuilder::from((variant_config, ResolvedPropertySource::Variant(name.clone())))
                            .build()
                ).override_model(model_resolved_property).apply_env().apply_default().build();

                (resolved.cache_ttl.clone(), ModelInfo::try_from(&resolved), ResolvedProviderConfig::OpenRouter(resolved))
            },
        };

        Self {
            name,
            base_name: base_name.clone(),
            // source,
            resolved,
            model_info,
            cache_ttl
        }
    }
}

impl TryFrom<&Variant> for (ModelInfo, LLMBuilder) {
    type Error = providers::error::ToLLMBuilderError;

    fn try_from(variant: &Variant) -> std::result::Result<Self, Self::Error> {
        match &variant.resolved {
            ResolvedProviderConfig::Ollama(resolved) =>
                Ok((ModelInfo::try_from(resolved)?, LLMBuilder::try_from(resolved)?)),
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
