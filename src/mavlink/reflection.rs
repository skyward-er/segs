//! Reflection utilities for MAVLink messages.
//!
//! This module provides a reflection context that allows to query information about MAVLink messages
//! and their fields. This is useful for dynamically generating UI elements based on the available
//! messages and fields.

use std::collections::HashMap;

use mavlink_bindgen::parser::{MavProfile, MavType};

use super::MAVLINK_PROFILE_SERIALIZED;

/// Reflection context for MAVLink messages.
///
/// This struct provides methods to query information about MAVLink messages and their fields.
pub struct ReflectionContext {
    mavlink_profile: MavProfile,
    id_name_map: HashMap<u32, String>,
}

impl ReflectionContext {
    /// Create a new reflection context.
    pub fn new() -> Self {
        let profile: MavProfile = serde_json::from_str(MAVLINK_PROFILE_SERIALIZED)
            .expect("Failed to deserialize MavProfile");
        let id_name_map = profile
            .messages
            .iter()
            .map(|(name, m)| (m.id, name.clone()))
            .collect();
        Self {
            mavlink_profile: profile,
            id_name_map,
        }
    }

    /// Get the name of a message by its ID.
    pub fn get_name_from_id(&self, message_id: u32) -> Option<&str> {
        self.id_name_map.get(&message_id).map(|s| s.as_str())
    }

    /// Get all message names in a sorted vector.
    pub fn sorted_messages(&self) -> Vec<&str> {
        let mut msgs: Vec<&str> = self
            .mavlink_profile
            .messages
            .keys()
            .map(|s| s.as_str())
            .collect();
        msgs.sort();
        msgs
    }

    /// Get all field names for a message by its ID.
    pub fn get_fields_by_id(&self, message_id: u32) -> Vec<&str> {
        self.mavlink_profile
            .messages
            .iter()
            .find(|(_, m)| m.id == message_id)
            .map(|(_, m)| &m.fields)
            .unwrap_or_else(|| {
                panic!("Message ID {} not found in profile", message_id);
            })
            .iter()
            .map(|f| f.name.as_str())
            .collect()
    }

    /// Get all plottable field names for a message by its ID.
    pub fn get_plottable_fields_by_id(&self, message_id: u32) -> Vec<&str> {
        self.mavlink_profile
            .messages
            .iter()
            .find(|(_, m)| m.id == message_id)
            .map(|(_, m)| &m.fields)
            .unwrap_or_else(|| {
                panic!("Message ID {} not found in profile", message_id);
            })
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
            .map(|f| f.name.as_str())
            .collect()
    }

    /// Get all field names for a message by its name.
    pub fn get_fields_by_name(&self, message_name: &str) -> Vec<&str> {
        self.mavlink_profile
            .messages
            .iter()
            .find(|(_, m)| m.name == message_name)
            .map(|(_, m)| &m.fields)
            .unwrap_or_else(|| {
                panic!("Message {} not found in profile", message_name);
            })
            .iter()
            .map(|f| f.name.as_str())
            .collect()
    }
}
