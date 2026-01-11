use std::collections::HashMap;
pub mod keyvalue;
pub mod argmatches;


use thiserror::Error;

use handlebars::{HelperDef, RenderError as HBRenderError, TemplateError};

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Unsupported data type: {data_type} for key: {key}")]
    UnsupportedDataType {
        key: String,
        data_type: String
    },

    #[error("HandlebarsRenderTemplateError: {0}")]
    HandlebarsRenderTemplateError(#[from] HBRenderError),

    #[error("Rendering handlebars template failed: {0}")]
    HandlebarsRegisterTemplateError(#[from] TemplateError),
}

pub trait Render<T> {
    fn render(&self,
        t: T,
        // extra: &PromptInputs,
        // helpers: HashMap<&str, Box<dyn HelperDef + Send + Sync>>
        helpers: HashMap<&str, Box<dyn HelperDef + Send + Sync>>
    ) -> Result<String, RenderError>;
}

