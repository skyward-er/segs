//! Reflection utilities for MAVLink messages.
//!
//! This module provides a reflection context that allows to query information about MAVLink messages
//! and their fields. This is useful for dynamically generating UI elements based on the available
//! messages and fields.

use std::collections::HashMap;

use mavlink_bindgen::parser::{MavProfile, MavType};

use crate::error::ErrInstrument;

use super::MAVLINK_PROFILE_SERIALIZED;

pub use mavlink_bindgen::parser::{MavField, MavMessage};

/// Reflection context for MAVLink messages.
///
/// This struct provides methods to query information about MAVLink messages and their fields.
pub struct ReflectionContext {
    mavlink_profile: MavProfile,
    id_msg_map: HashMap<u32, MavMessage>,
}

impl ReflectionContext {
    /// Create a new reflection context.
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

    /// Get the name of a message by its ID.
    pub fn get_msg(&self, msg: impl MessageLike) -> Option<&MavMessage> {
        msg.to_mav_message(self).ok()
    }

    /// Get all field names for a message by its ID.
    pub fn get_fields(&self, message_id: impl MessageLike) -> Option<Vec<IndexedField<'_>>> {
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

    /// Get all message names in a sorted vector.
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

    /// Get all plottable field names for a message by its ID.
    pub fn get_plottable_fields(
        &self,
        message_id: impl MessageLike,
    ) -> Option<Vec<IndexedField<'_>>> {
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
}

#[derive(Clone)]
pub struct IndexedField<'a> {
    id: usize,
    msg: &'a MavMessage,
    field: &'a MavField,
}

impl<'a> IndexedField<'a> {
    pub fn msg(&self) -> &MavMessage {
        self.msg
    }

    pub fn msg_id(&self) -> u32 {
        self.msg.id
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn field(&self) -> &MavField {
        self.field
    }

    pub fn name(&self) -> &str {
        &self.field.name
    }
}

pub trait MessageLike {
    fn to_mav_message<'a, 'b>(
        &'a self,
        ctx: &'b ReflectionContext,
    ) -> Result<&'b MavMessage, String>;
}

pub trait FieldLike<'a, 'b> {
    fn to_mav_field(
        &'a self,
        msg_id: u32,
        ctx: &'b ReflectionContext,
    ) -> Result<IndexedField<'b>, String>;
}

impl MessageLike for u32 {
    fn to_mav_message<'a, 'b>(
        &'a self,
        ctx: &'b ReflectionContext,
    ) -> Result<&'b MavMessage, String> {
        ctx.id_msg_map
            .get(self)
            .ok_or_else(|| format!("Message {} not found", self))
    }
}

impl MessageLike for &str {
    fn to_mav_message<'a, 'b>(
        &'a self,
        ctx: &'b ReflectionContext,
    ) -> Result<&'b MavMessage, String> {
        ctx.mavlink_profile
            .messages
            .iter()
            .find(|(_, m)| m.name == *self)
            .map(|(_, m)| m)
            .ok_or_else(|| format!("Message {} not found", self))
    }
}

impl<'b> FieldLike<'_, 'b> for &MavField {
    fn to_mav_field(
        &self,
        msg_id: u32,
        ctx: &'b ReflectionContext,
    ) -> Result<IndexedField<'b>, String> {
        ctx.id_msg_map
            .get(&msg_id)
            .and_then(|msg| {
                msg.fields
                    .iter()
                    .enumerate()
                    .find(|(_, f)| f == self)
                    .map(|(i, f)| IndexedField {
                        id: i,
                        msg,
                        field: f,
                    })
            })
            .ok_or_else(|| format!("Field {} not found in message {}", self.name, msg_id))
    }
}

impl<'b> FieldLike<'b, 'b> for IndexedField<'b> {
    fn to_mav_field(
        &self,
        _msg_id: u32,
        _ctx: &ReflectionContext,
    ) -> Result<IndexedField<'_>, String> {
        Ok(IndexedField {
            id: self.id,
            msg: self.msg,
            field: self.field,
        })
    }
}
impl<'b> FieldLike<'_, 'b> for usize {
    fn to_mav_field(
        &self,
        msg_id: u32,
        ctx: &'b ReflectionContext,
    ) -> Result<IndexedField<'b>, String> {
        ctx.id_msg_map
            .get(&msg_id)
            .and_then(|msg| {
                msg.fields.get(*self).map(|f| IndexedField {
                    id: *self,
                    msg,
                    field: f,
                })
            })
            .ok_or_else(|| format!("Field {} not found in message {}", self, msg_id))
    }
}
impl<'b> FieldLike<'_, 'b> for &str {
    fn to_mav_field(
        &self,
        msg_id: u32,
        ctx: &'b ReflectionContext,
    ) -> Result<IndexedField<'b>, String> {
        ctx.id_msg_map
            .get(&msg_id)
            .and_then(|msg| {
                msg.fields
                    .iter()
                    .find(|f| f.name == *self)
                    .map(|f| IndexedField {
                        id: msg.fields.iter().position(|f2| f2 == f).unwrap(),
                        msg,
                        field: f,
                    })
            })
            .ok_or_else(|| format!("Field {} not found in message {}", self, msg_id))
    }
}
