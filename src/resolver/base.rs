use llm::builder::LLMBuilder;

use crate::resolver::resolved::{ModelInfo, ToLLMBuilderError, ToModelInfoError};
use crate::resolver::{self, BaseProviderConfigSource, ResolvedProperty, ResolvedPropertySource, ResolvedProviderConfig};
use std::fmt;


#[derive(Debug)]
pub struct Base {
    pub name: String,
    pub resolved: ResolvedProviderConfig,
    pub model_info: Result<ModelInfo, ToModelInfoError>
    // pub source: BaseProviderConfigSource<'a>,
    // pub config: ResolvedProviderConfig
}

impl Base {
    pub fn new(
        name: String,
        source: BaseProviderConfigSource,
        model_resolved_property: Option<ResolvedProperty<String>>) -> Self {

        let (model_info, resolved) = match source {
            BaseProviderConfigSource::Ollama(source_config) => {
                let resolved = resolver::resolved::ollama::ResolvedProviderConfigBuilder::from(
                        (source_config, ResolvedPropertySource::Base(name.clone()))
                    ).override_model(model_resolved_property).build();
                (ModelInfo::try_from(&resolved),ResolvedProviderConfig::Ollama(resolved))
            },

            BaseProviderConfigSource::Anthropic(source_config) => {
                let resolved = resolver::resolved::anthropic::ResolvedProviderConfigBuilder::from(
                        (source_config, ResolvedPropertySource::Base(name.clone()))
                    ).override_model(model_resolved_property).build();
                (ModelInfo::try_from(&resolved),ResolvedProviderConfig::Anthropic(resolved))

            },
            BaseProviderConfigSource::OpenAI(source_config) => {
                let resolved = resolver::resolved::openai::ResolvedProviderConfigBuilder::from(
                        (source_config, ResolvedPropertySource::Base(name.clone()))
                    ).override_model(model_resolved_property).build();
                (ModelInfo::try_from(&resolved),ResolvedProviderConfig::OpenAI(resolved))
            },
        };
        Self {
            name,
            resolved,
            model_info
        }
    }
}

impl TryFrom<Base> for (ModelInfo, LLMBuilder) {
    type Error = ToLLMBuilderError;

    fn try_from(base: resolver::base::Base) -> std::result::Result<Self, Self::Error> {
        match base.resolved {
            resolver::ResolvedProviderConfig::Ollama(resolved) => {
                let model_info = ModelInfo::try_from(&resolved)?;
                let llmbuilder = LLMBuilder::try_from(&resolved)?;
                Ok((model_info, llmbuilder))
            }
            resolver::ResolvedProviderConfig::Anthropic(resolved) =>
                Ok((ModelInfo::try_from(&resolved)?, LLMBuilder::try_from(&resolved)?)),
            resolver::ResolvedProviderConfig::OpenAI(resolved) =>
                Ok((ModelInfo::try_from(&resolved)?, LLMBuilder::try_from(&resolved)?))
        }
    }
}

impl fmt::Display for Base {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Base: {}", &self.name)?;
        write!(f, "{}", self.resolved)
    }
}
