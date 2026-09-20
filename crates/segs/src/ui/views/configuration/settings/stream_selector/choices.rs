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
    DataKey, DataType, DataValue, SourceKey,
    adapter::DataAdapterInstanceToken,
    protocol::{FieldDescriptor, ProtocolDescriptor},
};
use crate::ui::widget_settings::StreamValueFilter;

/// Retains one adapter's reusable source and field choices in egui's shared cache.
struct CachedChoices {
    _adapter_token: DataAdapterInstanceToken,
    sources: Arc<SearchableComboBoxList<SourceKey>>,
    hierarchy: Arc<SearchableComboBoxHierarchy<DataKey>>,
    _filter: StreamValueFilter,
}

impl CachedChoices {
    /// Builds the source list and flattened field hierarchy for one installed adapter.
    fn build(
        protocol: &ProtocolDescriptor,
        adapter_token: &DataAdapterInstanceToken,
        filter: StreamValueFilter,
    ) -> Self {
        let sources =
            SearchableComboBoxList::new(protocol.sources.iter().map(|source| (source.key, source.name.clone())));
        let hierarchy = SearchableComboBoxHierarchy::build(|builder| {
            for message_key in &protocol.stream_messages {
                let Some(message) = protocol.message_schemas.get(message_key) else {
                    continue;
                };
                builder.group(&message.name, |builder| add_fields(builder, &message.fields, filter));
            }
        });
        Self {
            _adapter_token: adapter_token.clone(),
            sources: Arc::new(sources),
            hierarchy: Arc::new(hierarchy),
            _filter: filter,
        }
    }
}

/// Identifies one adapter-derived selection-choice cache entry.
#[derive(Clone, Copy)]
struct ChoicesRequest<'a> {
    protocol: &'a ProtocolDescriptor,
    adapter_token: &'a DataAdapterInstanceToken,
    filter: StreamValueFilter,
}

impl Hash for ChoicesRequest<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.adapter_token.hash(state);
        self.filter.hash(state);
    }
}

/// Builds missing adapter choice entries for egui's frame cache.
#[derive(Default)]
struct ChoicesComputer;

impl ComputerMut<ChoicesRequest<'_>, Arc<CachedChoices>> for ChoicesComputer {
    fn compute(&mut self, request: ChoicesRequest<'_>) -> Arc<CachedChoices> {
        Arc::new(CachedChoices::build(
            request.protocol,
            request.adapter_token,
            request.filter,
        ))
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
    filter: StreamValueFilter,
) -> (
    Arc<SearchableComboBoxList<SourceKey>>,
    Arc<SearchableComboBoxHierarchy<DataKey>>,
) {
    context.memory_mut(|memory| {
        let choices = memory.caches.cache::<ChoicesFrameCache>().get(ChoicesRequest {
            protocol,
            adapter_token,
            filter,
        });
        (choices.sources.clone(), choices.hierarchy.clone())
    })
}

/// Appends protocol fields to the component-owned hierarchy representation.
fn add_fields(
    builder: &mut SearchableComboBoxHierarchyBuilder<'_, DataKey>,
    fields: &[FieldDescriptor],
    filter: StreamValueFilter,
) {
    for field in fields {
        match field {
            FieldDescriptor::Structure { name, fields } => {
                builder.group(name, |builder| add_fields(builder, fields, filter));
            }
            FieldDescriptor::Field {
                name,
                field_type,
                data_key,
            } if filter.accepts_type(*field_type) => builder.item(*data_key, name),
            FieldDescriptor::EnumField {
                name,
                descriptor,
                data_key,
            } if filter.accepts_enum(&descriptor.variants) => builder.item(*data_key, name),
            FieldDescriptor::Field { .. } | FieldDescriptor::EnumField { .. } => {}
        }
    }
}

impl StreamValueFilter {
    /// Returns whether a scalar protocol field is accepted by this filter.
    fn accepts_type(self, data_type: DataType) -> bool {
        match self {
            Self::Any => true,
            Self::Integer => matches!(
                data_type,
                DataType::U8
                    | DataType::U16
                    | DataType::U32
                    | DataType::U64
                    | DataType::I8
                    | DataType::I16
                    | DataType::I32
                    | DataType::I64
            ),
        }
    }

    /// Returns whether a protocol enum's represented values are accepted.
    fn accepts_enum(self, variants: &[(String, DataValue)]) -> bool {
        match self {
            Self::Any => true,
            Self::Integer => variants.iter().all(|(_, value)| {
                matches!(
                    value,
                    DataValue::U8(_)
                        | DataValue::U16(_)
                        | DataValue::U32(_)
                        | DataValue::U64(_)
                        | DataValue::I8(_)
                        | DataValue::I16(_)
                        | DataValue::I32(_)
                        | DataValue::I64(_)
                )
            }),
        }
    }
}
