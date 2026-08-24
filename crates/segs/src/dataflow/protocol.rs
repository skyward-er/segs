use crate::dataflow::{DataKey, DataType, SourceKey};

/// Describes one structure or selectable field in a protocol hierarchy.
#[derive(Debug)]
pub enum FieldDescriptor {
    /// A named structure containing nested descriptors.
    Structure { name: String, fields: Vec<FieldDescriptor> },
    /// A selectable field mapped to a data stream.
    Field {
        name: String,
        field_type: DataType,
        data_key: DataKey,
    },
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
    pub sources: Vec<SourceDescriptor>,
}
