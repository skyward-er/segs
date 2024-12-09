//! This module is a wrapper around the `skyward_mavlink` crate, facilitates
//! rapid switching between different mavlink versions and profiles.
//!
//! In addition, it provides few utility functions to work with mavlink messages.

use std::time::Instant;

use skyward_mavlink::mavlink::peek_reader::PeekReader;

// Re-export from the mavlink crate
pub use skyward_mavlink::{
    mavlink::*, orion::*,
    reflection::ORION_MAVLINK_PROFILE_SERIALIZED as MAVLINK_PROFILE_SERIALIZED,
};

/// A wrapper around the `MavMessage` struct, adding a received time field.
#[derive(Debug, Clone)]
pub struct TimedMessage {
    /// The underlying mavlink message
    pub message: MavMessage,
    /// The time instant at which the message was received
    pub time: Instant,
}

impl TimedMessage {
    pub fn just_received(message: MavMessage) -> Self {
        Self {
            message,
            time: Instant::now(),
        }
    }
}

pub fn extract_from_message<K, T>(
    message: &MavMessage,
    fields: impl IntoIterator<Item = K>,
) -> Vec<T>
where
    K: AsRef<str>,
    T: serde::de::DeserializeOwned + Default,
{
    let value: serde_json::Value = serde_json::to_value(message).unwrap();
    fields
        .into_iter()
        .map(|field| {
            let field = field.as_ref();
            let value = value.get(field).unwrap();
            serde_json::from_value::<T>(value.clone()).unwrap_or_default()
        })
        .collect()
}

/// Helper function to read a stream of bytes and return an iterator of MavLink messages
pub fn byte_parser(buf: &[u8]) -> impl Iterator<Item = (MavHeader, MavMessage)> + '_ {
    let mut reader = PeekReader::new(buf);
    std::iter::from_fn(move || read_v1_msg(&mut reader).ok())
}
