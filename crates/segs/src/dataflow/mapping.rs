use std::path::PathBuf;

pub enum MappingType {
    BuiltIn,
    LocalFile,
}

/// Enum representing the different types of data mapping sources
/// that can be used to define how raw data should be interpreted by the data adapter.
#[derive(Debug)]
pub enum DataMapping {
    /// Mapping that is built into the adapter implementation
    BuiltIn(i32), // ID of the built-in mapping
    /// Path to a local file containing the mapping
    LocalFile(PathBuf),
}

pub struct MappingDescriptor {
    pub method: MappingType,
    pub description: String,
}
