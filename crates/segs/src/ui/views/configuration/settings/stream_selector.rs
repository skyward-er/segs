use egui::{CollapsingHeader, ComboBox, Ui};
use segs_memory::MemoryExt;
use segs_ui::widgets::UiWidgetExt;

use crate::dataflow::{
    DataKey, SourceKey, StreamKey,
    protocol::{FieldDescriptor, ProtocolDescriptor, SourceDescriptor},
};

/// Renders the source and stream controls for a widget data setting.
pub fn show(ui: &mut Ui, label: &str, stream: &mut Option<StreamKey>, protocol: Option<&ProtocolDescriptor>) {
    // Validate the protocol before rendering controls that index its descriptors
    let Some(protocol) = protocol else {
        ui.weak("No data source configured.");
        return;
    };
    if protocol.sources.is_empty() {
        ui.weak("No data sources available.");
        return;
    }
    if protocol.messages.is_empty() {
        ui.weak("No streams available.");
        return;
    }

    // Render both key selectors from the widget value or their temporary state
    let source_key = show_source_selection(ui, &protocol.sources, stream.as_ref().map(|stream| stream.source_key));
    ui.label(label);
    let data_key = show_fields(ui, &protocol.messages, stream.as_ref().map(|stream| stream.data_key));

    // Apply the complete selection to the widget
    *stream = data_key.map(|data_key| StreamKey { source_key, data_key });
}

/// Renders the source picker and returns its selected key.
///
/// A completed stream takes precedence over temporary UI state.
fn show_source_selection(
    ui: &mut Ui,
    sources: &[SourceDescriptor],
    selected_source_key: Option<SourceKey>,
) -> SourceKey {
    let selection_id = ui.id().with("source");
    // Lambda to check if the source is still available in the protocol's sources
    let is_available = |key| sources.iter().any(|source| source.key == key);

    // Get the source from the stream then UI memory then the first descriptor
    let mut source_key = selected_source_key
        .filter(|key| is_available(*key))
        .or_else(|| {
            ui.mem()
                .get_temp::<SourceKey>(selection_id)
                .filter(|key| is_available(*key))
        })
        .unwrap_or(sources[0].key);
    let source_name = &sources
        .iter()
        .find(|source| source.key == source_key)
        .expect("selected source must exist")
        .name;

    // Render the source names while retaining their stable keys in UI memory
    ui.horizontal(|ui| {
        ui.label("Source");
        ComboBox::from_id_salt(selection_id)
            .width(ui.available_width())
            .truncate()
            .selected_text(source_name)
            .show_ui(ui, |ui| {
                for source in sources {
                    ui.selectable_value(&mut source_key, source.key, &source.name);
                }
            });
    });

    // Retain the source independently from the complete stream
    ui.mem().insert_temp(selection_id, source_key);
    source_key
}

/// Renders a descriptor hierarchy and returns its selected data key.
fn show_fields(ui: &mut Ui, descriptors: &[FieldDescriptor], stream_data_key: Option<DataKey>) -> Option<DataKey> {
    // Resolve the data key from the stream then temporary UI state
    let selection_id = ui.id().with("data");
    let mut selected_data_key =
        stream_data_key.or_else(|| ui.mem().get_temp::<Option<DataKey>>(selection_id).flatten());

    // Recurse into structures and render leaf fields as checkboxes
    for descriptor in descriptors {
        show_field(ui, descriptor, &mut selected_data_key);
    }

    // Retain the field selection independently from the complete stream
    ui.mem().insert_temp(selection_id, selected_data_key);
    selected_data_key
}

/// Renders one field descriptor and updates the selected data key.
fn show_field(ui: &mut Ui, descriptor: &FieldDescriptor, selected_data_key: &mut Option<DataKey>) {
    match descriptor {
        FieldDescriptor::Structure { name, fields } => {
            // Group nested descriptors under a stable collapsible heading
            CollapsingHeader::new(name).id_salt(name).show(ui, |ui| {
                for field in fields {
                    show_field(ui, field, selected_data_key);
                }
            });
        }
        FieldDescriptor::Field { name, data_key, .. } => {
            let mut selected = *selected_data_key == Some(*data_key);

            // Keep the checkbox and field name on one row
            let clicked = ui
                .horizontal(|ui| {
                    let response = ui.check(&mut selected);
                    ui.label(name);
                    response
                })
                .inner
                .clicked();

            if clicked {
                *selected_data_key = selected.then_some(*data_key);
            }
        }
    }
}
