#![allow(dead_code)]

pub mod adapter;
pub mod mapping;
pub mod mavlink_adapter;
pub mod transport;

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataKey {
    pub source_id: u32,
    pub message_id: u32,
    pub field_hash: u32, // Hash to detect changes in the field structure of a message
}

#[derive(Debug, Clone, Copy)]
pub struct DataPoint<T> {
    pub timestamp: f64,
    pub value: T,
}

pub enum DataStream {
    F64(Vec<DataPoint<f64>>),
    I64(Vec<DataPoint<i64>>),
    String(Vec<DataPoint<String>>),
}

pub enum DataValue {
    F64(f64),
    I64(i64),
    Bool(bool),
    String(String),
}

/// Raw message type stored as key-value pairs for maximum flexibility of representation.
pub type RawMessage = HashMap<u32, DataValue>;

pub struct CommandSequence {
    request: RawMessage,
    response: Vec<RawMessage>,
}

/// Central data store that holds all processed data streams, raw messages, and command sequences.
///
/// Data adapters will update this store with new data points as they are processed.
/// UI will read from this store to display information to the user.
#[derive(Default)]
pub struct DataStore {
    pub streams: HashMap<DataKey, DataStream>,
    pub raw_store: Vec<RawMessage>,
    pub commands: Vec<CommandSequence>,
}

impl DataStore {
    pub fn new() -> Self {
        Default::default()
    }
}
