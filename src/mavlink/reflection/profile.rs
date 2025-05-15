//! Reflection context for MAVLink messages.
//!
//! This module defines the [`ReflectionContext`] struct, which provides methods to query
//! information about MAVLink messages and their fields. It is used as the core context
//! for all reflection-based operations in the MAVLink reflection subsystem.

use std::collections::HashMap;

use mavlink_bindgen::parser::{MavProfile, MavType};

use crate::error::ErrInstrument;

use super::{
    MAVLINK_PROFILE_SERIALIZED,
    conversion::{FieldLike, MessageLike},
    fields::IndexedField,
};

pub use mavlink_bindgen::parser::MavMessage;

/// Reflection context for MAVLink messages.
///
/// This struct provides methods to query information about MAVLink messages and their fields.
/// It is constructed from a serialized MAVLink profile and maintains a mapping from message IDs
/// to message definitions for efficient lookup.
pub struct ReflectionContext {
    /// The deserialized MAVLink profile containing all message and field definitions.
    pub(super) mavlink_profile: MavProfile,
    /// A map from message ID to message definition for fast lookup.
    pub(super) id_msg_map: HashMap<u32, MavMessage>,
}

impl ReflectionContext {
    /// Create a new reflection context by deserializing the MAVLink profile.
    ///
    /// # Panics
    /// Panics if the profile cannot be deserialized.
    pub fn new() -> Self {
        let profile: MavProfile = serde_json::from_str(MAVLINK_PROFILE_SERIALIZED)
            .log_expect("Failed to deserialize MavProfile");
        let id_msg_map = profile
            .messages
            .values()
            .map(|m| (m.id, m.clone()))
            .collect();
        Self {
            mavlink_profile: profile,
            id_msg_map,
        }
    }

    /// Get a reference to a message definition by message identifier.
    ///
    /// # Arguments
    /// * `msg` - A message identifier (ID or name) implementing [`MessageLike`].
    ///
    /// # Returns
    /// * `Some(&MavMessage)` if found, otherwise `None`.
    pub fn get_msg(&'static self, msg: impl MessageLike) -> Option<&'static MavMessage> {
        msg.to_mav_message(self).ok()
    }

    /// Get all fields for a message by its identifier.
    ///
    /// # Arguments
    /// * `message_id` - A message identifier implementing [`MessageLike`].
    ///
    /// # Returns
    /// * `Some(Vec<IndexedField>)` if the message exists, otherwise `None`.
    pub fn get_fields(&'static self, message_id: impl MessageLike) -> Option<Vec<IndexedField>> {
        message_id.to_mav_message(self).ok().map(|msg| {
            msg.fields
                .iter()
                .enumerate()
                .map(|(i, f)| IndexedField {
                    id: i,
                    msg,
                    field: f,
                })
                .collect()
        })
    }

    /// Get all message definitions in a sorted vector by name.
    ///
    /// # Returns
    /// * `Vec<&MavMessage>` sorted by message name.
    pub fn get_sorted_msgs(&self) -> Vec<&MavMessage> {
        let mut msgs: Vec<(&str, &MavMessage)> = self
            .mavlink_profile
            .messages
            .iter()
            .map(|(k, m)| (k.as_str(), m))
            .collect();
        msgs.sort_by_cached_key(|(k, _)| *k);
        msgs.into_iter().map(|(_, m)| m).collect()
    }

    /// Get all plottable fields for a message by its identifier.
    ///
    /// Plottable fields are those with numeric types suitable for plotting.
    ///
    /// # Arguments
    /// * `message_id` - A message identifier implementing [`MessageLike`].
    ///
    /// # Returns
    /// * `Some(Vec<IndexedField>)` if the message exists, otherwise `None`.
    pub fn get_plottable_fields(
        &'static self,
        message_id: impl MessageLike,
    ) -> Option<Vec<IndexedField>> {
        let msg = message_id.to_mav_message(self).ok()?;
        msg.fields
            .iter()
            .filter(|f| {
                matches!(
                    f.mavtype,
                    MavType::UInt8
                        | MavType::UInt16
                        | MavType::UInt32
                        | MavType::UInt64
                        | MavType::Int8
                        | MavType::Int16
                        | MavType::Int32
                        | MavType::Int64
                        | MavType::Float
                        | MavType::Double
                )
            })
            .map(|f| f.to_mav_field(msg.id, self).ok())
            .collect()
    }

    /// Get all fields whose names end with "state" or "status" for a message.
    ///
    /// # Arguments
    /// * `message_id` - A message identifier implementing [`MessageLike`].
    ///
    /// # Returns
    /// * `Some(Vec<IndexedField>)` if the message exists, otherwise `None`.
    pub fn get_all_state_fields(
        &'static self,
        message_id: impl MessageLike,
    ) -> Option<Vec<IndexedField>> {
        let msg = message_id.to_mav_message(self).ok()?;
        msg.fields
            .iter()
            .filter(|f| {
                f.name.to_lowercase().ends_with("state")
                    || f.name.to_lowercase().ends_with("status")
            })
            .map(|f| f.to_mav_field(msg.id, self).ok())
            .collect()
    }
}
