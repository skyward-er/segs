pub mod connection;
mod core;

use std::collections::BTreeMap;

use bytes::{Buf, BufMut, Bytes};
use mavlink_bindgen::parser::{MavEnum, MavProfile as ProfileInfo, MavType};
pub use mavlink_bindgen::parser::{MavMessage as MessageInfo, parse_profile};
pub use mavlink_core::MavlinkVersion;
use mavlink_core::{MavHeader, error::ParserError, utils::remove_trailing_zeroes};

use crate::core::MAVLinkMessageRaw;

pub struct MavProfile {
    pub enums: BTreeMap<String, MavEnum>,
    pub messages: BTreeMap<u32, MessageInfo>,
}

impl MavProfile {
    pub fn new(profile: ProfileInfo) -> Self {
        Self {
            enums: profile.enums.clone(),
            messages: profile.messages.into_values().map(|msg| (msg.id, msg)).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MavFrame {
    pub version: MavlinkVersion,
    pub header: MavHeader,
    pub message: MavMessage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MavMessage {
    pub id: u32,
    pub fields: Vec<MsgField>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MsgField {
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Char(char),
    Float(f32),
    Double(f64),
    CharArray(String),
    Array(Vec<Self>),
}

impl MavFrame {
    pub fn parse(raw_msg: &MAVLinkMessageRaw, profile_msg: &MessageInfo) -> Result<Self, ParserError> {
        let version = match raw_msg {
            MAVLinkMessageRaw::V1(_) => MavlinkVersion::V1,
            MAVLinkMessageRaw::V2(_) => MavlinkVersion::V2,
        };
        let header = match raw_msg {
            MAVLinkMessageRaw::V1(msg) => MavHeader {
                system_id: msg.system_id(),
                component_id: msg.component_id(),
                sequence: msg.sequence(),
            },
            MAVLinkMessageRaw::V2(msg) => MavHeader {
                system_id: msg.system_id(),
                component_id: msg.component_id(),
                sequence: msg.sequence(),
            },
        };
        let message = match raw_msg {
            MAVLinkMessageRaw::V1(msg) => MavMessage::parse(msg.payload(), profile_msg)?,
            MAVLinkMessageRaw::V2(msg) => MavMessage::parse(msg.payload(), profile_msg)?,
        };
        Ok(Self {
            version,
            header,
            message,
        })
    }
}

impl MavMessage {
    pub fn parse(payload: &[u8], profile_msg: &MessageInfo) -> Result<Self, ParserError> {
        let id = profile_msg.id;
        let mut buf = Bytes::copy_from_slice(payload);
        let fields = profile_msg
            .fields
            .iter()
            .map(|f| MsgField::parse_type(&f.mavtype, &mut buf))
            .collect::<Result<Vec<MsgField>, ParserError>>()?;
        Ok(MavMessage { id, fields })
    }

    pub fn ser(self, version: MavlinkVersion, bytes: &mut [u8]) -> usize {
        let mut buf = &mut *bytes;
        for field in self.fields {
            field.ser(&mut buf);
        }
        match version {
            MavlinkVersion::V1 => bytes.len(),
            MavlinkVersion::V2 => remove_trailing_zeroes(bytes),
        }
    }
}

impl MsgField {
    fn parse_type(mav_type: &MavType, buf: &mut Bytes) -> Result<Self, ParserError> {
        Ok(match mav_type {
            MavType::UInt8MavlinkVersion | MavType::UInt8 => Self::UInt8(buf.get_u8()),
            MavType::UInt16 => Self::UInt16(buf.get_u16_le()),
            MavType::UInt32 => Self::UInt32(buf.get_u32_le()),
            MavType::UInt64 => Self::UInt64(buf.get_u64_le()),
            MavType::Int8 => Self::Int8(buf.get_i8()),
            MavType::Int16 => Self::Int16(buf.get_i16_le()),
            MavType::Int32 => Self::Int32(buf.get_i32_le()),
            MavType::Int64 => Self::Int64(buf.get_i64_le()),
            MavType::Char => Self::Char(buf.get_u8() as char),
            MavType::Float => Self::Float(buf.get_f32_le()),
            MavType::Double => Self::Double(buf.get_f64_le()),
            MavType::CharArray(len) => {
                let mut chars = Vec::new();
                for _ in 0..*len {
                    chars.push(buf.get_u8() as char);
                }
                Self::CharArray(chars.into_iter().collect())
            }
            MavType::Array(mav_type, len) => {
                let mut array = Vec::new();
                for _ in 0..*len {
                    array.push(Self::parse_type(mav_type, buf)?);
                }
                Self::Array(array)
            }
        })
    }

    fn ser<B: BufMut>(self, buf: &mut B) {
        match self {
            Self::UInt8(v) => buf.put_u8(v),
            Self::UInt16(v) => buf.put_u16_le(v),
            Self::UInt32(v) => buf.put_u32_le(v),
            Self::UInt64(v) => buf.put_u64_le(v),
            Self::Int8(v) => buf.put_i8(v),
            Self::Int16(v) => buf.put_i16_le(v),
            Self::Int32(v) => buf.put_i32_le(v),
            Self::Int64(v) => buf.put_i64_le(v),
            Self::Char(c) => buf.put_u8(c as u8),
            Self::Float(f) => buf.put_f32_le(f),
            Self::Double(d) => buf.put_f64_le(d),
            Self::CharArray(s) => buf.put_slice(s.as_bytes()),
            Self::Array(arr) => arr.into_iter().for_each(|item| item.ser(buf)),
        }
    }
}
