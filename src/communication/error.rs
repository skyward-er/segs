//! Error handling for communication modules.
//!
//! Contains definitions for errors that can occur during serial or Ethernet communication.

use skyward_mavlink::mavlink::error::MessageWriteError;
use thiserror::Error;

/// Represents communication errors.
#[derive(Debug, Error)]
pub enum CommunicationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Connection closed")]
    ConnectionClosed,
}

/// Represents errors during connection setup.
#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Wrong configuration: {0}")]
    WrongConfiguration(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unknown error")]
    Unknown(String),
}

impl From<MessageWriteError> for CommunicationError {
    fn from(e: MessageWriteError) -> Self {
        match e {
            MessageWriteError::Io(e) => Self::Io(e),
        }
    }
}
