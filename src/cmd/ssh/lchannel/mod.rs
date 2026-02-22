use thiserror::Error;
use std::{string::FromUtf8Error};
use async_trait::async_trait;

mod stream_common;
pub mod tcp;
pub mod usock;
pub mod ssh;
pub mod multissh;

#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Encoding Error: {0}")]
    EncodingError(#[from] FromUtf8Error),
    #[error("Timeout waiting for channel")]
    TimeoutError,
    #[error("Channel Error: {0}")]
    Other(String)
}

#[async_trait]
pub trait LChannel: Send {
    async fn run(&self) -> Result<(), ChannelError>;
}
