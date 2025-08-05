//! This module contains all the structs and functions to work with Mavlink messages.
//!
//! It serves also as an abstraction wrapper around the `skyward_mavlink` crate, facilitating
//! rapid switching between different mavlink versions and profiles (_dialects_).

mod error;
pub mod reflection;

use std::time::Instant;

// Re-export from the mavlink crate
pub use skyward_mavlink::{
    mavlink::*, orion::*,
    reflection::ORION_MAVLINK_PROFILE_SERIALIZED as MAVLINK_PROFILE_SERIALIZED,
};

/// Default port for the Ethernet connection
pub const DEFAULT_RCV_ETHERNET_PORT: u16 = 42069;
pub const DEFAULT_SEND_ETHERNET_PORT: u16 = 42070;

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

    pub fn id(&self) -> u32 {
        self.message.message_id()
    }
}
