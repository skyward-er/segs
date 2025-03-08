use thiserror::Error;

#[derive(Debug, Error)]
pub enum MavlinkError {
    #[error("Error parsing message: {0}")]
    ParseError(#[from] serde_json::Error),
}
