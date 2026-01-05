pub mod edit;
pub mod enable;
pub mod disable;
pub mod create;
pub mod list;
pub mod cat;
pub mod run;
pub mod import;
pub mod stats;
pub mod resolve;

mod templates;

use thiserror::Error;

use ::edit::edit as _edit;

#[derive(Error, Debug)]
pub enum TextEditorError {
    #[error("Editor Error")]
    IoError(#[from] std::io::Error)
}

pub trait TextEditor {
    fn edit(&self, input: &str) -> Result<String, TextEditorError>;
}

pub struct BasicTextEditor;

impl TextEditor for BasicTextEditor {
    fn edit(&self, input: &str) -> Result<String, TextEditorError> {
        let result = _edit(input);

        result.map_err(TextEditorError::IoError)
    }
}
