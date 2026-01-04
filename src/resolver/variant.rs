use llm::builder::LLMBuilder;

use crate::resolver::{resolved::{anthropic, ollama, openai, ModelInfo, ToLLMBuilderError, ToModelInfoError}, ResolvedProperty, ResolvedPropertySource, ResolvedProviderConfig, VariantProviderConfigSource};

#[derive(Debug)]
pub struct Variant {
    pub name: String,
    pub base_name: String,
    // pub source: VariantProviderConfigSource<'a>,
    pub resolved: ResolvedProviderConfig,
    pub model_info: Result<ModelInfo, ToModelInfoError>,
    // pub conf: ResolvedProviderConfig,
    // pub base_config: ProviderConfigSource<'a>,
}

impl Variant {
    pub fn new(
        name: String,
        base_name: String,
        source: VariantProviderConfigSource,
        model_resolved_property: Option<ResolvedProperty<String>>) -> Self {

        let (model_info, resolved) = match source {
            VariantProviderConfigSource::Ollama(variant_config, base_config) => {

                let resolved = ollama::ResolvedProviderConfigBuilder::from(
                    (base_config, ResolvedPropertySource::Base(base_name.clone()))
                ).override_from(
                        &ollama::ResolvedProviderConfigBuilder::from((variant_config, ResolvedPropertySource::Variant(name.clone())))
                            .build()
                    ).override_model(model_resolved_property).build();

                    (ModelInfo::try_from(&resolved), ResolvedProviderConfig::Ollama(resolved))
            },
            VariantProviderConfigSource::Anthropic(variant_config, base_config) => {
                let resolved = anthropic::ResolvedProviderConfigBuilder::from(
                    (base_config, ResolvedPropertySource::Base(base_name.clone()))
                ).override_from(
                        &anthropic::ResolvedProviderConfigBuilder::from((variant_config, ResolvedPropertySource::Variant(name.clone())))
                            .build()
                    ).override_model(model_resolved_property).build();

                    (ModelInfo::try_from(&resolved), ResolvedProviderConfig::Anthropic(resolved))
            },
            VariantProviderConfigSource::OpenAI(variant_config, base_config) => {
                let resolved = openai::ResolvedProviderConfigBuilder::from(
                    (base_config, ResolvedPropertySource::Base(base_name.clone()))
                ).override_from(
                        &openai::ResolvedProviderConfigBuilder::from((variant_config, ResolvedPropertySource::Variant(name.clone())))
                            .build()
                    ).override_model(model_resolved_property).build();

                    (ModelInfo::try_from(&resolved), ResolvedProviderConfig::OpenAI(resolved))
            },
        };

        Self {
            name,
            base_name: base_name.clone(),
            // source,
            resolved,
            model_info
        }
    }
}

impl TryFrom<&Variant> for (ModelInfo, LLMBuilder) {
    type Error = ToLLMBuilderError;

    fn try_from(variant: &Variant) -> std::result::Result<Self, Self::Error> {
        match &variant.resolved {
            ResolvedProviderConfig::Ollama(resolved) =>
                Ok((ModelInfo::try_from(resolved)?, LLMBuilder::try_from(resolved)?)),
            ResolvedProviderConfig::Anthropic(resolved) =>
                Ok((ModelInfo::try_from(resolved)?, LLMBuilder::try_from(resolved)?)),
            ResolvedProviderConfig::OpenAI(resolved) =>
                Ok((ModelInfo::try_from(resolved)?, LLMBuilder::try_from(resolved)?)),
        }
    }
}
