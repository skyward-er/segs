//! Reflection utilities for MAVLink messages.
//!
//! This module provides a reflection context that allows to query information about MAVLink messages
//! and their fields. This is useful for dynamically generating UI elements based on the available
//! messages and fields.

use std::collections::HashMap;

use mavlink_bindgen::parser::{MavProfile, MavType};
use serde::ser::SerializeStruct;
use skyward_mavlink::mavlink::Message;

use crate::{MAVLINK_PROFILE, error::ErrInstrument};

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
    pub fn get_msg(&'static self, msg: impl MessageLike) -> Option<&'static MavMessage> {
        msg.to_mav_message(self).ok()
    }

    /// Get all field names for a message by its ID.
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

#[derive(Debug, Clone)]
pub struct IndexedField {
    id: usize,
    msg: &'static MavMessage,
    field: &'static MavField,
}

macro_rules! extract_as_type {
    ($as_type: ty, $func: ident, $($mav_ty: ident, $rust_ty: ty),+) => {
        pub fn $func(&self, message: &impl Message) -> Result<$as_type, String> {
            macro_rules! downcast {
                ($value: expr, $type: ty) => {
                    Ok(*$value
                        .downcast::<$type>()
                        .map_err(|_| "Type mismatch".to_string())? as $as_type)
                };
            }

            let value = message
                .get_field(self.id)
                .ok_or("Field not found".to_string())?;
            match self.field.mavtype {
                $(MavType::$mav_ty => downcast!(value, $rust_ty),)+
                _ => Err("Field type not supported".to_string()),
            }
        }
    };
}

impl IndexedField {
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

    pub fn extract_as_string<T: Message>(&self, message: &T) -> Result<String, String> {
        let value = message
            .get_field(self.id)
            .ok_or("Field not found".to_string())?;
        let str_value = format!("{:?}", value);
        Ok(str_value)
    }
}

/// ### Extractors
/// These methods allow to extract the value of a field from a message, casting
/// it to the desired type.
impl IndexedField {
    #[rustfmt::skip]
    extract_as_type!(f32, extract_as_f32,
        UInt8, u8,
        UInt16, u16,
        UInt32, u32,
        UInt64, u64,
        Int8, i8,
        Int16, i16,
        Int32, i32,
        Int64, i64,
        Float, f32,
        Double, f64
    );

    #[rustfmt::skip]
    extract_as_type!(f64, extract_as_f64,
        UInt8, u8,
        UInt16, u16,
        UInt32, u32,
        UInt64, u64,
        Int8, i8,
        Int16, i16,
        Int32, i32,
        Int64, i64,
        Float, f32,
        Double, f64
    );

    #[rustfmt::skip]
    extract_as_type!(u8, extract_as_u8,
        UInt8, u8,
        Char, char
    );

    #[rustfmt::skip]
    extract_as_type!(u16, extract_as_u16,
        UInt8, u8,
        Int8, i8,
        UInt16, u16
    );

    #[rustfmt::skip]
    extract_as_type!(u32, extract_as_u32,
        UInt8, u8,
        Int8, i8,
        UInt16, u16,
        Int16, i16,
        UInt32, u32
    );

    #[rustfmt::skip]
    extract_as_type!(u64, extract_as_u64,
        UInt8, u8,
        Int8, i8,
        UInt16, u16,
        Int16, i16,
        UInt32, u32,
        Int32, i32,
        UInt64, u64
    );

    #[rustfmt::skip]
    extract_as_type!(i8, extract_as_i8,
        Int8, i8
    );

    #[rustfmt::skip]
    extract_as_type!(i16, extract_as_i16,
        UInt8, u8,
        Int8, i8,
        Int16, i16
    );

    #[rustfmt::skip]
    extract_as_type!(i32, extract_as_i32,
        UInt8, u8,
        Int8, i8,
        UInt16, u16,
        Int16, i16,
        Int32, i32
    );

    #[rustfmt::skip]
    extract_as_type!(i64, extract_as_i64,
        UInt8, u8,
        Int8, i8,
        UInt16, u16,
        Int16, i16,
        UInt32, u32,
        Int32, i32,
        Int64, i64
    );

    #[rustfmt::skip]
    extract_as_type!(char, extract_as_char,
        UInt8, u8,
        Char, char
    );
}

impl std::hash::Hash for IndexedField {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.msg.id.hash(state);
    }
}

impl PartialEq for IndexedField {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.msg.id == other.msg.id
    }
}

impl serde::Serialize for IndexedField {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("IndexedField", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("msg_id", &self.msg.id)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for IndexedField {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct IndexedFieldDe {
            id: usize,
            msg_id: u32,
        }

        let de = IndexedFieldDe::deserialize(deserializer)?;
        let field = de
            .id
            .to_mav_field(de.msg_id, &MAVLINK_PROFILE)
            .map_err(|u| serde::de::Error::custom(format!("Invalid field: {}", u)))?;
        Ok(field)
    }
}

pub trait MessageLike {
    fn to_mav_message(
        &self,
        ctx: &'static ReflectionContext,
    ) -> Result<&'static MavMessage, String>;
}

pub trait FieldLike {
    fn to_mav_field(
        &self,
        msg_id: u32,
        ctx: &'static ReflectionContext,
    ) -> Result<IndexedField, String>;
}

impl MessageLike for u32 {
    fn to_mav_message<'b>(&self, ctx: &'b ReflectionContext) -> Result<&'b MavMessage, String> {
        ctx.id_msg_map
            .get(self)
            .ok_or_else(|| format!("Message {} not found", self))
    }
}

impl MessageLike for &str {
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
    fn to_mav_field(&self, _msg_id: u32, _ctx: &ReflectionContext) -> Result<IndexedField, String> {
        Ok(IndexedField {
            id: self.id,
            msg: self.msg,
            field: self.field,
        })
    }
}

impl FieldLike for usize {
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
