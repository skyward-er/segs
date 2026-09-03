pub mod descriptor_index;

use std::collections::HashMap;

use crate::dataflow::{DataKey, DataType, MessageKey, SourceKey};

/// Describes one structure or field in a message schema.
#[derive(Debug)]
pub enum FieldDescriptor {
    /// A named structure containing nested descriptors.
    Structure { name: String, fields: Vec<FieldDescriptor> },
    /// A named value decoded with the given exact type.
    Field {
        name: String,
        field_type: DataType,
        data_key: DataKey,
    },
}

/// Describes the canonical schema of one protocol message.
#[derive(Debug)]
pub struct MessageDescriptor {
    pub name: String,
    pub fields: Vec<FieldDescriptor>,
}

/// Describes one selectable data source.
#[derive(Debug)]
pub struct SourceDescriptor {
    pub name: String,
    pub key: SourceKey,
}

/// Describes the canonical messages and their protocol roles exposed by a data adapter.
#[derive(Debug)]
pub struct ProtocolDescriptor {
    /// Canonical message schemas keyed by their protocol-independent identity.
    pub message_schemas: HashMap<MessageKey, MessageDescriptor>,
    /// Ordered identities of messages whose fields can be selected as data streams.
    pub stream_messages: Vec<MessageKey>,
    /// Ordered identities of command messages that can be sent to a target.
    pub command_messages: Vec<MessageKey>,
    /// Selectable target data sources.
    pub sources: Vec<SourceDescriptor>,
}
