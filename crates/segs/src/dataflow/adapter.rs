use std::{
    fmt,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Instant,
};

use argh::FromArgValue;
use serde::{Deserialize, Serialize};

use crate::dataflow::{
    mapping::{DataMapping, MappingDescriptor},
    protocol::{ProtocolDescriptor, descriptor_index::DescriptorIndex},
    skyward_mavlink_adapter::SkywardMavlinkAdapter,
    store::DataStore,
    transport::DataTransport,
};

#[derive(Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromArgValue)]
pub enum AdapterType {
    #[default]
    #[argh(name = "skyward-mavlink")]
    SkywardMavlink,
    // Future adapter types can be added here
}

impl fmt::Display for AdapterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SkywardMavlink => f.write_str("Skyward MAVLink"),
        }
    }
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
    fn describe_protocol(&self) -> &ProtocolDescriptor;

    /// Process incoming data and update the data store.
    ///
    /// Returns true if new data was processed, false otherwise.
    fn process_incoming(&mut self, data_store: &mut DataStore) -> bool;

    /// Process outgoing data from the data store.
    fn process_outgoing(&mut self, data_store: &mut DataStore);

    fn status(&self) -> Status;
}

pub struct Status {
    pub transport: DataTransport,
    pub mapping: DataMapping,
    pub rx: Stats,
    pub tx: Stats,
}

/// A snapshot of successful frame traffic and I/O errors for one direction.
#[derive(Clone, Copy)]
pub struct Stats {
    /// Time of the most recent successful frame.
    pub last_time: Instant,
    /// Successful frames per second during the preceding rolling statistics window.
    pub rate: f32,
    /// Cumulative number of successfully transferred frames.
    pub count: u32,
    /// Cumulative number of frame parsing or transport I/O errors.
    pub errors: u32,
}

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

pub fn try_new(
    adapter_type: Option<AdapterType>,
    transport: Option<DataTransport>,
    mapping: Option<DataMapping>,
    ctx: egui::Context,
) -> Option<Box<dyn DataAdapter>> {
    match (adapter_type, transport, mapping) {
        (Some(AdapterType::SkywardMavlink), Some(transport), Some(mapping)) => {
            println!("Loading Skyward MAVLink adapter\n\tTransport: {transport:?}\n\tMapping: {mapping:?}");
            Some(Box::new(
                SkywardMavlinkAdapter::new(ctx, transport, mapping)
                    .inspect_err(|e| println!("Failed to load Skyward MAVLink adapter: {e}"))
                    .ok()?,
            ))
        }
        _ => None,
    }
}

pub fn get_mapping_sources(adapter: AdapterType) -> Vec<MappingDescriptor> {
    match adapter {
        AdapterType::SkywardMavlink => SkywardMavlinkAdapter::get_mapping_sources(),
    }
}

/// Owns one installed adapter together with its lifecycle identity.
///
/// Derived UI state and caches must be invalidated when an adapter is replaced,
/// even when the replacement exposes an identical protocol. This wrapper creates
/// that identity alongside the adapter so callers cannot forget to update it.
pub struct DataAdapterInstance {
    adapter: Box<dyn DataAdapter>,
    token: DataAdapterInstanceToken,
    descriptor_index: DescriptorIndex,
}

impl DataAdapterInstance {
    /// Wraps an adapter as a newly installed instance.
    pub fn new(adapter: Box<dyn DataAdapter>) -> Self {
        let descriptor_index = DescriptorIndex::build(adapter.describe_protocol());

        Self {
            adapter,
            token: DataAdapterInstanceToken(Arc::new(())),
            descriptor_index,
        }
    }

    /// Returns the identity used to associate derived state with this instance.
    pub fn token(&self) -> &DataAdapterInstanceToken {
        &self.token
    }

    /// Returns the flattened stream descriptor index owned by this adapter instance.
    pub fn descriptor_index(&self) -> &DescriptorIndex {
        &self.descriptor_index
    }
}

impl Deref for DataAdapterInstance {
    type Target = dyn DataAdapter;

    fn deref(&self) -> &Self::Target {
        self.adapter.as_ref()
    }
}

impl DerefMut for DataAdapterInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.adapter.as_mut()
    }
}

/// Cloneable identity for one installed adapter lifecycle.
///
/// Identity follows an allocation shared by all clones, avoiding timestamps,
/// global counters, and manually maintained revisions. Retaining a clone also
/// prevents its address from being reused while derived cached state is alive.
#[derive(Clone, Debug)]
pub struct DataAdapterInstanceToken(Arc<()>);

impl PartialEq for DataAdapterInstanceToken {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for DataAdapterInstanceToken {}

impl Hash for DataAdapterInstanceToken {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
