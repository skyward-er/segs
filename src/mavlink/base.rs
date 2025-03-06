//! Wrapper around the `skyward_mavlink` crate
//!
//! This facilitates rapid switching between different mavlink versions and profiles.
//!
//! In addition, it provides few utility functions to work with mavlink messages.

use std::time::Instant;

// Re-export from the mavlink crate
pub use skyward_mavlink::{
    mavlink::*, orion::*,
    reflection::ORION_MAVLINK_PROFILE_SERIALIZED as MAVLINK_PROFILE_SERIALIZED,
};

use crate::error::ErrInstrument;

use super::error::{MavlinkError, Result};

/// A wrapper around the `MavMessage` struct, adding a received time field.
#[derive(Debug, Clone)]
pub struct TimedMessage {
    /// The underlying mavlink message
    pub message: MavMessage,
    /// The time instant at which the message was received
    pub time: Instant,
}

impl TimedMessage {
    /// Create a new `TimedMessage` instance with the given message and the current time
    pub fn just_received(message: MavMessage) -> Self {
        Self {
            message,
            time: Instant::now(),
        }
    }
}

/// Extract fields from a MavLink message using string keys
#[profiling::function]
pub fn extract_from_message<K, T>(
    message: &MavMessage,
    fields: impl IntoIterator<Item = K>,
) -> Result<Vec<T>>
where
    K: AsRef<str>,
    T: serde::de::DeserializeOwned + Default,
{
    let value: serde_json::Value =
        serde_json::to_value(message).log_expect("MavMessage should be serializable");
    Ok(fields
        .into_iter()
        .flat_map(|field| {
            let field = field.as_ref();
            let value = value
                .get(field)
                .ok_or(MavlinkError::UnknownField(field.to_string()))?;
            serde_json::from_value::<T>(value.clone()).map_err(MavlinkError::from)
        })
        .collect())
}
