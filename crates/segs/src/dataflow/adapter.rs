use std::time::Instant;

use argh::FromArgValue;
use serde::{Deserialize, Serialize};

use crate::dataflow::{
    DataStore,
    mapping::{DataMapping, MappingDescriptor},
    mavlink_adapter::MavlinkAdapter,
    protocol::ProtocolDescriptor,
    transport::DataTransport,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromArgValue)]
pub enum AdapterType {
    #[default]
    #[argh(name = "mavlink")]
    MAVLink,
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
    /// The egui context is provided to allow the adapter to request UI updates when new data is received.
    fn new(
        ctx: egui::Context,
        transport: DataTransport,
        mapping: DataMapping,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    /// Returns a list of available mapping sources for this adapter to be used during adapter configuration
    /// to present options to the user.
    fn get_mapping_sources() -> Vec<MappingDescriptor>
    where
        Self: Sized;

    /// Describe the data protocol that this adapter implements.
    /// The returned structure shall reflect the hierarchical organization of the data as defined by the adapter.
    /// Every leaf node in the structure maps to a data stream via [`DataKey`]. The [`DataKey`] contained in this
    /// descriptor can be used to subscribe to that stream in the central data store for a specific source.
    fn describe_protocol(&self) -> ProtocolDescriptor;

    /// Prepare the data store with any necessary initial structure or metadata before processing begins.
    fn prepare_data_store(&self, _data_store: &mut DataStore) {
        // Default implementation does nothing, can be overridden by specific adapters if needed
    }

    /// Process incoming data and update the data store.
    ///
    /// Returns true if new data was processed, false otherwise.
    fn process_incoming(&mut self, data_store: &mut DataStore) -> bool;

    fn status(&self) -> Status;
}

pub struct Status {
    pub transport: DataTransport,
    pub mapping: DataMapping,
    pub rx: Stats,
    pub tx: Stats,
}

pub struct Stats {
    pub last_time: Instant,
    pub rate: f32,
    pub count: u32,
    pub errors: u32,
}

// TODO: stub implementation, remove once properly implemented with real data in mavlink adapter
impl Default for Stats {
    fn default() -> Self {
        Self {
            last_time: Instant::now(),
            rate: 0.,
            count: 0,
            errors: 0,
        }
    }
}

pub fn get_mapping_sources(adapter: AdapterType) -> Vec<MappingDescriptor> {
    match adapter {
        AdapterType::MAVLink => MavlinkAdapter::get_mapping_sources(),
    }
}
