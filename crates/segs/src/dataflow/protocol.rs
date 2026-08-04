use crate::dataflow::{DataKey, DataType, SourceKey};

#[derive(Debug)]
pub enum FieldDescriptor {
    Structure {
        name: String,
        fields: Vec<FieldDescriptor>,
    },
    Field {
        name: String,
        field_type: DataType,
        data_key: DataKey,
    },
}

#[derive(Debug)]
pub struct SourceDescriptor {
    pub name: String,
    pub key: SourceKey,
}

#[derive(Debug)]
pub struct ProtocolDescriptor {
    pub messages: Vec<FieldDescriptor>,
    pub sources: Vec<SourceDescriptor>,
}
