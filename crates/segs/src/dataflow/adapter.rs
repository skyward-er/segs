use crate::dataflow::{
    DataStore,
    mapping::{DataMapping, MappingDescriptor},
    transport::DataTransport,
};

pub enum AdapterType {
    Mavlink,
    // Future adapter types can be added here
}

/// Trait that defines the interface for data adapters, which are responsible for receiving raw data from various
/// sources, processing it according to defined mappings, and updating the central data store with structured data
/// points.
///
/// This abstraction allows the core application logic to remain decoupled from specific data formats and sources,
/// enabling flexibility and extensibility in how data is ingested and processed.
pub trait DataAdapter {
    /// Create a new adapter instance with the given transport configuration and mapping source.
    fn new(transport: DataTransport, mapping: DataMapping) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    /// Returns a list of available mapping sources for this adapter to be used during adapter configuration
    /// to present options to the user.
    fn get_mapping_sources() -> Vec<MappingDescriptor>
    where
        Self: Sized;

    /// Prepare the data store with any necessary initial structure or metadata before processing begins.
    fn prepare_data_store(&self, _data_store: &mut DataStore) {
        // Default implementation does nothing, can be overridden by specific adapters if needed
    }

    /// Process incoming data and update the data store.
    ///
    /// Returns true if new data was processed, false otherwise.
    fn process_incoming(&mut self, data_store: &mut DataStore) -> bool;
}
