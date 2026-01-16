use thiserror::Error;


#[derive(Error, Debug)]
pub enum ToLLMBuilderError {
    #[error("{0} {1} is required but not configured")]
    RequiredConfiguration(&'static str, &'static str),
    #[error("{0}")]
    ModelError(#[from] ToModelInfoError)
}

#[derive(Error, Debug, Clone)]
#[error("'{0}' is required but not configured")]
pub enum ToModelInfoError {
    RequiredConfiguration(&'static str)
}
