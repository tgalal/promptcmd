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
pub mod config;

mod templates;

use thiserror::Error;

use ::edit::Builder;
use ::edit::edit_with_builder;

#[derive(Error, Debug)]
pub enum TextEditorError {
    #[error("Editor Error")]
    IoError(#[from] std::io::Error)
}

pub enum TextEditorFileType {
    Toml,
    Dotprompt
}

pub trait TextEditor {
    fn edit(&self, input: &str, eftype: TextEditorFileType) -> Result<String, TextEditorError>;
}

pub struct BasicTextEditor;

impl TextEditor for BasicTextEditor {
    fn edit(&self, input: &str, eftype: TextEditorFileType) -> Result<String, TextEditorError> {

        let mut b = Builder::new();

        match eftype {
            TextEditorFileType::Toml => b.suffix(".toml"),
            TextEditorFileType::Dotprompt => b.suffix(".prompt"),
        };

        let result = edit_with_builder(input, &b);

        result.map_err(TextEditorError::IoError)
    }
}
