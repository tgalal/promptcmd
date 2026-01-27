use thiserror::Error;
use std::{string::FromUtf8Error};
use async_trait::async_trait;

mod stream_common;
pub mod tcp;
pub mod usock;

#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Encoding Error: {0}")]
    EncodingError(#[from] FromUtf8Error),
}

#[async_trait]
pub trait LChannel: Send {
    async fn run(&self) -> Result<(), ChannelError>;
}
