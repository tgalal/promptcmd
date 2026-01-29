use llm::builder::LLMBuilder;

use crate::config::{providers::{error::ToModelInfoError, ModelInfo}, resolver::{ResolvedGlobalProperties, ResolvedProperty, ResolvedPropertySource, ResolvedProviderConfig, VariantProviderConfigSource}};
use crate::config::providers;

#[derive(Debug, PartialEq)]
pub struct Variant {
    pub name: String,
    pub base_name: String,
    pub resolved: ResolvedProviderConfig,
    pub model_info: Result<ModelInfo, ToModelInfoError>,
    pub globals: ResolvedGlobalProperties
}

impl Variant {
    pub fn new(
        name: String,
        base_name: String,
        source: VariantProviderConfigSource,
        fm_properties: Option<ResolvedGlobalProperties>,
        overrides: Option<ResolvedGlobalProperties>,
        global_provider_properties: ResolvedGlobalProperties,
        model_resolved_property: Option<ResolvedProperty<String>>) -> Self {

        macro_rules! resolve_final_config {
            ($provider:ident, $base_config:ident, $variant_config:ident) => {
            {

            providers::$provider::ResolvedProviderConfigBuilder::from_defaults()
                .apply_providers_env()
                .apply_global_overrides(Some(global_provider_properties))
                .apply_env()
                .override_from(
                    &providers::$provider::ResolvedProviderConfigBuilder::from(
                        ($base_config, ResolvedPropertySource::Base(base_name.clone()))).build()
                )                .apply_global_overrides(fm_properties)
                .apply_variant_env(&name)
                    .override_from(
                        &providers::$provider::ResolvedProviderConfigBuilder::from(($variant_config, ResolvedPropertySource::Variant(name.clone())))
                            .build()
                )
                .apply_global_overrides(overrides)
                .override_model(model_resolved_property)
                .build()
            }
            }
        }

        let (globals, model_info, resolved) = match source {
            VariantProviderConfigSource::Ollama(variant_config, base_config) => {

            let resolved = resolve_final_config!(ollama, base_config, variant_config);

                (resolved.globals.clone(), ModelInfo::try_from(&resolved), ResolvedProviderConfig::Ollama(resolved))
            },
            VariantProviderConfigSource::Anthropic(variant_config, base_config) => {
                let resolved = resolve_final_config!(anthropic, base_config, variant_config);

                (resolved.globals.clone(), ModelInfo::try_from(&resolved), ResolvedProviderConfig::Anthropic(resolved))
            },
            VariantProviderConfigSource::OpenAI(variant_config, base_config) => {
                let resolved = resolve_final_config!(openai, base_config, variant_config);

                (resolved.globals.clone(), ModelInfo::try_from(&resolved), ResolvedProviderConfig::OpenAI(resolved))
            },
            VariantProviderConfigSource::Google(variant_config, base_config) => {
                let resolved = resolve_final_config!(google, base_config, variant_config);

                (resolved.globals.clone(), ModelInfo::try_from(&resolved), ResolvedProviderConfig::Google(resolved))
            },
            VariantProviderConfigSource::OpenRouter(variant_config, base_config) => {
                let resolved = resolve_final_config!(openrouter, base_config, variant_config);

                (resolved.globals.clone(), ModelInfo::try_from(&resolved), ResolvedProviderConfig::OpenRouter(resolved))
            },
        };

        Self {
            name,
            base_name: base_name.clone(),
            resolved,
            model_info,
            globals
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
