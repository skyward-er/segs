#![allow(dead_code)]

pub mod adapter;
pub mod mapping;
pub mod mavlink_adapter;
pub mod transport;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;

use nohash::NoHashHasher;

/// An opaque handler that uniquely represents a data stream.
/// Adapters are responsible for generating a uniform key space based on the data source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataKey(u64);

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

/// Command type stored as key-value pairs for maximum flexibility of representation.
pub type Command = HashMap<DataKey, DataValue>;

pub struct CommandSequence {
    request: Command,
    response: Vec<Command>,
}

/// Central data store that holds all processed data streams, raw messages, and command sequences.
///
/// Data adapters will update this store with new data points as they are processed.
/// UI will read from this store to display information to the user.
#[derive(Default)]
pub struct DataStore {
    pub streams: HashMap<DataKey, DataStream, BuildHasherDefault<NoHashHasher<u64>>>,
    pub commands: Vec<CommandSequence>,
}

impl DataStore {
    pub fn new() -> Self {
        Self {
            streams: HashMap::with_hasher(BuildHasherDefault::default()),
            ..Default::default()
        }
    }
}
