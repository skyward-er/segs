use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use egui::{
    Context,
    cache::{ComputerMut, FrameCache},
};
use segs_ui::widgets::{SearchableComboBoxHierarchy, SearchableComboBoxHierarchyBuilder};

use crate::dataflow::{
    DataKey,
    adapter::DataAdapterInstanceToken,
    protocol::{FieldDescriptor, ProtocolDescriptor},
};

/// Retains one adapter's reusable field hierarchy in egui's shared cache.
struct CachedHierarchy {
    _adapter_token: DataAdapterInstanceToken,
    hierarchy: Arc<SearchableComboBoxHierarchy<DataKey>>,
}

impl CachedHierarchy {
    /// Builds the flattened hierarchy for one installed adapter.
    fn build(protocol: &ProtocolDescriptor, adapter_token: &DataAdapterInstanceToken) -> Self {
        let hierarchy = SearchableComboBoxHierarchy::build(|builder| {
            for message_key in &protocol.stream_messages {
                let Some(message) = protocol.message_schemas.get(message_key) else {
                    continue;
                };
                builder.group(&message.name, |builder| add_fields(builder, &message.fields));
            }
        });
        Self {
            _adapter_token: adapter_token.clone(),
            hierarchy: Arc::new(hierarchy),
        }
    }
}

/// Identifies one adapter-derived hierarchy cache entry.
#[derive(Clone, Copy)]
struct HierarchyRequest<'a> {
    protocol: &'a ProtocolDescriptor,
    adapter_token: &'a DataAdapterInstanceToken,
}

impl Hash for HierarchyRequest<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.adapter_token.hash(state);
    }
}

/// Builds missing adapter hierarchy entries for egui's frame cache.
#[derive(Default)]
struct HierarchyComputer;

impl ComputerMut<HierarchyRequest<'_>, Arc<CachedHierarchy>> for HierarchyComputer {
    fn compute(&mut self, request: HierarchyRequest<'_>) -> Arc<CachedHierarchy> {
        Arc::new(CachedHierarchy::build(request.protocol, request.adapter_token))
    }
}

type HierarchyFrameCache = FrameCache<Arc<CachedHierarchy>, HierarchyComputer>;

/// Returns the reusable field hierarchy for the installed adapter.
///
/// The returned hierarchy is shared across every field selector during the
/// adapter lifecycle and rebuilt after the adapter identity changes.
pub fn resolve_hierarchy(
    context: &Context,
    protocol: &ProtocolDescriptor,
    adapter_token: &DataAdapterInstanceToken,
) -> Arc<SearchableComboBoxHierarchy<DataKey>> {
    context.memory_mut(|memory| {
        memory
            .caches
            .cache::<HierarchyFrameCache>()
            .get(HierarchyRequest {
                protocol,
                adapter_token,
            })
            .hierarchy
            .clone()
    })
}

/// Appends protocol fields to the component-owned hierarchy representation.
fn add_fields(builder: &mut SearchableComboBoxHierarchyBuilder<'_, DataKey>, fields: &[FieldDescriptor]) {
    for field in fields {
        match field {
            FieldDescriptor::Structure { name, fields } => {
                builder.group(name, |builder| add_fields(builder, fields));
            }
            FieldDescriptor::Field { name, data_key, .. } | FieldDescriptor::EnumField { name, data_key, .. } => {
                builder.item(*data_key, name)
            }
        }
    }
}
