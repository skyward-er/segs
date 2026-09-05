#![allow(dead_code)]

pub mod adapter;
pub mod mapping;
pub mod protocol;
pub mod skyward_mavlink_adapter;
pub mod store;
pub mod transport;

use std::{fmt, time::SystemTime};

use egui::ahash::HashMap;
use serde::{Deserialize, Serialize};

/// An opaque handler that uniquely represents a data stream.
/// Adapters are responsible for generating a uniform key space based on the data source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DataKey(u64);

/// An opaque handler that uniquely represents a source of data, such as a specific system or component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceKey(u32);

/// An opaque handler that uniquely represents a protocol message schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageKey(u64);

/// A protocol-independent identifier for a command sequence in the central datastore.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandId(u64);

/// An opaque handler that uniquely represents a data stream coming from a specific source.
/// This is the key that will be used to store and retrieve data in the central [`DataStore`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StreamKey {
    pub source_key: SourceKey,
    pub data_key: DataKey,
}

impl StreamKey {
    /// A stream key that is always available from a mock [`DataStore`].
    pub const fn mock() -> Self {
        Self {
            source_key: SourceKey(0),
            data_key: DataKey(0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DataPoint<T> {
    /// Floating-point timestamp in seconds
    pub timestamp: f64,
    pub value: T,
}

#[derive(Debug)]
pub enum DataType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
}

impl fmt::Display for DataType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Bool => "bool",
            Self::String => "string",
        })
    }
}

pub enum DataStream {
    F64(Vec<DataPoint<f64>>),
    I64(Vec<DataPoint<i64>>),
    String(Vec<DataPoint<String>>),
}

impl DataStream {
    /// Returns the most recent sample in this stream.
    ///
    /// The returned tuple contains the adapter-relative timestamp in seconds
    /// followed by the sample value. `None` means the stream has no samples.
    pub fn last(&self) -> Option<(f64, DataValue)> {
        match self {
            Self::F64(points) => points
                .last()
                .map(|point| (point.timestamp, DataValue::F64(point.value))),
            Self::I64(points) => points
                .last()
                .map(|point| (point.timestamp, DataValue::I64(point.value))),
            Self::String(points) => points
                .last()
                .map(|point| (point.timestamp, DataValue::String(point.value.clone()))),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DataValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
}

impl fmt::Display for DataValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8(value) => write!(formatter, "{}", value),
            Self::U16(value) => write!(formatter, "{}", value),
            Self::U32(value) => write!(formatter, "{}", value),
            Self::U64(value) => write!(formatter, "{}", value),
            Self::I8(value) => write!(formatter, "{}", value),
            Self::I16(value) => write!(formatter, "{}", value),
            Self::I32(value) => write!(formatter, "{}", value),
            Self::I64(value) => write!(formatter, "{}", value),
            Self::F32(value) => write!(formatter, "{:?}", value),
            Self::F64(value) => write!(formatter, "{:?}", value),
            Self::Bool(value) => write!(formatter, "{}", value),
            Self::String(value) => write!(formatter, "{}", value),
        }
    }
}

pub enum CommandStatus {
    /// The command is pending response.
    Pending,
    /// The command did not receive a final response before its deadline.
    TimedOut,
    /// The command was completed successfully.
    Success,
    /// The command was rejected by the target.
    Rejected,
    /// The command encountered a local error.
    LocalError,
}

impl fmt::Display for CommandStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Pending => "Pending",
            Self::TimedOut => "Timed out",
            Self::Success => "Success",
            Self::Rejected => "Rejected",
            Self::LocalError => "Local error",
        })
    }
}

/// Command type stored as key-value pairs for maximum flexibility of representation.
pub struct Command {
    pub key: MessageKey,
    pub target: SourceKey,
    pub timestamp: SystemTime,
    pub fields: HashMap<DataKey, DataValue>,
}

pub struct CommandSequence {
    pub id: CommandId,
    pub status: CommandStatus,
    pub request: Command,
    pub responses: Vec<Command>,
}

/// Testing module exposing functionality that would otherwise be private.
#[cfg(test)]
pub mod testing {
    use super::*;

    pub const fn message_key(value: u64) -> MessageKey {
        MessageKey(value)
    }

    #[test]
    fn float_values_use_adaptive_debug_formatting() {
        assert_eq!(DataValue::F64(9.5e-44).to_string(), "9.5e-44");
        assert_eq!(DataValue::F64(1e30).to_string(), "1e30");
        assert_eq!(DataValue::F32(1.25).to_string(), "1.25");
        assert_eq!(DataValue::F32(1.0).to_string(), "1.0");
    }
}
