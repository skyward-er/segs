use thiserror::Error;

pub type Result<T> = std::result::Result<T, MavlinkError>;

#[derive(Debug, Error)]
pub enum MavlinkError {
    #[error("Error parsing field: {0}")]
    UnknownField(String),
    #[error("Error parsing message: {0}")]
    ParseError(#[from] serde_json::Error),
}
