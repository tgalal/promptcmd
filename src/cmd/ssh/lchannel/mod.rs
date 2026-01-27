use thiserror::Error;
use std::{string::FromUtf8Error};

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
