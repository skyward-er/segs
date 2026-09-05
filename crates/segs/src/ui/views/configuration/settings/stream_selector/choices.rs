use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use egui::{
    Context,
    cache::{ComputerMut, FrameCache},
};
use segs_ui::widgets::{SearchableComboBoxHierarchy, SearchableComboBoxHierarchyBuilder, SearchableComboBoxList};

use crate::dataflow::{
    DataKey, SourceKey,
    adapter::DataAdapterInstanceToken,
    protocol::{FieldDescriptor, ProtocolDescriptor},
};

/// Retains one adapter's reusable source and field choices in egui's shared cache.
struct CachedChoices {
    _adapter_token: DataAdapterInstanceToken,
    sources: Arc<SearchableComboBoxList<SourceKey>>,
    hierarchy: Arc<SearchableComboBoxHierarchy<DataKey>>,
}

impl CachedChoices {
    /// Builds the source list and flattened field hierarchy for one installed adapter.
    fn build(protocol: &ProtocolDescriptor, adapter_token: &DataAdapterInstanceToken) -> Self {
        let sources =
            SearchableComboBoxList::new(protocol.sources.iter().map(|source| (source.key, source.name.clone())));
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
            sources: Arc::new(sources),
            hierarchy: Arc::new(hierarchy),
        }
    }
}

/// Identifies one adapter-derived selection-choice cache entry.
#[derive(Clone, Copy)]
struct ChoicesRequest<'a> {
    protocol: &'a ProtocolDescriptor,
    adapter_token: &'a DataAdapterInstanceToken,
}

impl Hash for ChoicesRequest<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.adapter_token.hash(state);
    }
}

/// Builds missing adapter choice entries for egui's frame cache.
#[derive(Default)]
struct ChoicesComputer;

impl ComputerMut<ChoicesRequest<'_>, Arc<CachedChoices>> for ChoicesComputer {
    fn compute(&mut self, request: ChoicesRequest<'_>) -> Arc<CachedChoices> {
        Arc::new(CachedChoices::build(request.protocol, request.adapter_token))
    }
}

type ChoicesFrameCache = FrameCache<Arc<CachedChoices>, ChoicesComputer>;

/// Returns the reusable source list and field hierarchy for the installed adapter.
///
/// The first tuple value contains the flat source choices, and the second
/// contains the hierarchical message and field choices. Both values are shared
/// across selectors during the adapter lifecycle and rebuilt after it changes.
pub fn resolve_choices(
    context: &Context,
    protocol: &ProtocolDescriptor,
    adapter_token: &DataAdapterInstanceToken,
) -> (
    Arc<SearchableComboBoxList<SourceKey>>,
    Arc<SearchableComboBoxHierarchy<DataKey>>,
) {
    context.memory_mut(|memory| {
        let choices = memory.caches.cache::<ChoicesFrameCache>().get(ChoicesRequest {
            protocol,
            adapter_token,
        });
        (choices.sources.clone(), choices.hierarchy.clone())
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
