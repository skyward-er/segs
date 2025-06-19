//! Conversion traits for MAVLink message and field identifiers.
//!
//! This module defines the [`MessageLike`] and [`FieldLike`] traits, which abstract over
//! different types that can be used to identify MAVLink messages and fields for reflection.

use crate::error::ErrInstrument;

use mavlink_bindgen::parser::{MavField, MavMessage};

use super::{fields::IndexedField, profile::ReflectionContext};

/// Trait for types that can be converted to a MAVLink message definition.
///
/// Implemented for message IDs (`u32`) and message names (`&str`).
pub trait MessageLike {
    /// Converts the type to a reference to a MAVLink message definition.
    ///
    /// # Arguments
    /// * `ctx` - The reflection context.
    ///
    /// # Returns
    /// * `Ok(&MavMessage)` if found, otherwise `Err(String)`.
    fn to_mav_message(
        &self,
        ctx: &'static ReflectionContext,
    ) -> Result<&'static MavMessage, String>;
}

/// Trait for types that can be converted to an [`IndexedField`] within a message.
///
/// Implemented for field indices (`usize`), field names (`&str`), and field references.
pub trait FieldLike {
    /// Converts the type to an [`IndexedField`] for the given message.
    ///
    /// # Arguments
    /// * `msg_id` - The message ID.
    /// * `ctx` - The reflection context.
    ///
    /// # Returns
    /// * `Ok(IndexedField)` if found, otherwise `Err(String)`.
    fn to_mav_field(
        &self,
        msg_id: u32,
        ctx: &'static ReflectionContext,
    ) -> Result<IndexedField, String>;
}

impl MessageLike for u32 {
    /// Converts a message ID to a MAVLink message definition.
    fn to_mav_message<'b>(&self, ctx: &'b ReflectionContext) -> Result<&'b MavMessage, String> {
        ctx.id_msg_map
            .get(self)
            .ok_or_else(|| format!("Message {} not found", self))
    }
}

impl MessageLike for &str {
    /// Converts a message name to a MAVLink message definition.
    fn to_mav_message<'b>(&self, ctx: &'b ReflectionContext) -> Result<&'b MavMessage, String> {
        ctx.mavlink_profile
            .messages
            .iter()
            .find(|(_, m)| m.name == *self)
            .map(|(_, m)| m)
            .ok_or_else(|| format!("Message {} not found", self))
    }
}

impl FieldLike for &MavField {
    /// Converts a field reference to an [`IndexedField`] for the given message.
    fn to_mav_field(
        &self,
        msg_id: u32,
        ctx: &'static ReflectionContext,
    ) -> Result<IndexedField, String> {
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

impl FieldLike for IndexedField {
    /// Returns a clone of the [`IndexedField`].
    fn to_mav_field(&self, _msg_id: u32, _ctx: &ReflectionContext) -> Result<IndexedField, String> {
        Ok(IndexedField {
            id: self.id,
            msg: self.msg,
            field: self.field,
        })
    }
}

impl FieldLike for usize {
    /// Converts a field index to an [`IndexedField`] for the given message.
    fn to_mav_field(
        &self,
        msg_id: u32,
        ctx: &'static ReflectionContext,
    ) -> Result<IndexedField, String> {
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

impl FieldLike for &str {
    /// Converts a field name to an [`IndexedField`] for the given message.
    fn to_mav_field(
        &self,
        msg_id: u32,
        ctx: &'static ReflectionContext,
    ) -> Result<IndexedField, String> {
        ctx.id_msg_map
            .get(&msg_id)
            .and_then(|msg| {
                msg.fields
                    .iter()
                    .find(|f| f.name == *self)
                    .map(|f| IndexedField {
                        id: msg.fields.iter().position(|f2| f2 == f).log_unwrap(),
                        msg,
                        field: f,
                    })
            })
            .ok_or_else(|| format!("Field {} not found in message {}", self, msg_id))
    }
}
