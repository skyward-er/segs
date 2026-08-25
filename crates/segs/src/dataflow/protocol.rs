use std::collections::HashMap;

use crate::dataflow::{CommandKey, DataKey, DataType, SourceKey};

/// Describes one structure or selectable field in a protocol hierarchy.
#[derive(Debug)]
pub enum FieldDescriptor {
    /// A named structure containing nested descriptors.
    Structure { name: String, fields: Vec<FieldDescriptor> },
    Field {
        name: String,
        field_type: DataType,
        data_key: DataKey,
    },
}

/// Describes a command that can be sent to remote systems through adapters.
#[derive(Debug)]
pub struct CommandDescriptor {
    pub name: String,
    pub key: CommandKey,
    pub fields: Vec<FieldDescriptor>,
}

/// Describes one selectable data source.
#[derive(Debug)]
pub struct SourceDescriptor {
    pub name: String,
    pub key: SourceKey,
}

/// Describes the sources and message hierarchy exposed by a data adapter.
#[derive(Debug)]
pub struct ProtocolDescriptor {
    pub messages: Vec<FieldDescriptor>,
    pub commands: HashMap<CommandKey, CommandDescriptor>,
    pub sources: Vec<SourceDescriptor>,
}
