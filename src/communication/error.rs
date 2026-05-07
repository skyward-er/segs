use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommunicationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Connection closed")]
    ConnectionClosed,
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Wrong configuration: {0}")]
    WrongConfiguration(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
