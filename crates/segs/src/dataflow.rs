#![allow(dead_code)]

pub mod adapter;
pub mod mapping;
pub mod mavlink_adapter;
pub mod protocol;
pub mod transport;

use std::collections::HashMap;

/// An opaque handler that uniquely represents a data stream.
/// Adapters are responsible for generating a uniform key space based on the data source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataKey(u64);

/// An opaque handler that uniquely represents a source of data, such as a specific system or component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceKey(u32);

/// An opaque handler that uniquely represents a data stream coming from a specific source.
/// This is the key that will be used to store and retrieve data in the central [`DataStore`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StreamKey {
    source_key: SourceKey,
    data_key: DataKey,
}

#[derive(Debug, Clone, Copy)]
pub struct DataPoint<T> {
    pub timestamp: f64,
    pub value: T,
}

#[derive(Debug)]
pub enum DataType {
    F64,
    I64,
    String,
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

/// Command type stored as key-value pairs for maximum flexibility of representation.
pub type Command = HashMap<DataKey, DataValue>;

pub struct CommandSequence {
    source: SourceKey,
    request: Command,
    response: Vec<Command>,
}

/// Central data store that holds all processed data streams, raw messages, and command sequences.
///
/// Data adapters will update this store with new data points as they are processed.
/// UI will read from this store to display information to the user.
#[derive(Default)]
pub struct DataStore {
    pub streams: HashMap<StreamKey, DataStream>,
    pub commands: Vec<CommandSequence>,
}

impl DataStore {
    pub fn new() -> Self {
        Default::default()
    }
}
