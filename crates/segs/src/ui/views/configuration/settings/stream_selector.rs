mod choices;

use std::collections::HashSet;

use egui::{ComboBox, Id, Ui};
use segs_memory::MemoryExt;
use segs_ui::widgets::{MultipleSelection, SearchableComboBox, SearchableComboBoxHierarchy, SingleSelection};

use crate::dataflow::{DataKey, SourceKey, StreamKey, adapter::DataAdapterInstance, protocol::SourceDescriptor};

use self::choices::resolve_hierarchy;

const FIELD_SELECTOR_MAX_ROWS: usize = 13;

/// Renders the source and stream controls for a widget data setting.
pub fn show(ui: &mut Ui, label: &str, stream: &mut Option<StreamKey>, adapter: Option<&DataAdapterInstance>) {
    show_selection(ui, label, StreamSelection::Single(stream), adapter);
}

/// Renders the source and multiple-field controls for a widget data setting.
pub fn show_multiple(
    ui: &mut Ui,
    label: &str,
    streams: &mut Vec<StreamKey>,
    names: Option<&mut Vec<String>>,
    adapter: Option<&DataAdapterInstance>,
) {
    show_selection(ui, label, StreamSelection::Multiple { streams, names }, adapter);
}

/// The widget-owned stream storage accepted by the shared selector.
enum StreamSelection<'a> {
    Single(&'a mut Option<StreamKey>),
    Multiple {
        streams: &'a mut Vec<StreamKey>,
        names: Option<&'a mut Vec<String>>,
    },
}

fn show_selection(ui: &mut Ui, label: &str, selection: StreamSelection<'_>, adapter: Option<&DataAdapterInstance>) {
    // Validate the adapter before rendering controls backed by its protocol
    let Some(adapter) = adapter else {
        ui.weak("No data source configured.");
        return;
    };
    let protocol = adapter.describe_protocol();
    if protocol.sources.is_empty() {
        ui.weak("No data sources available.");
        return;
    }
    if protocol.stream_messages.is_empty() {
        ui.weak("No streams available.");
        return;
    }
    let hierarchy = resolve_hierarchy(ui.ctx(), protocol, adapter.token());
    let field_selector_id = ui.make_persistent_id(("stream_field_selector", adapter.token()));

    // Render both selectors from persisted widget state
    let selected_source_key = match &selection {
        StreamSelection::Single(stream) => stream.as_ref().map(|stream| stream.source_key),
        StreamSelection::Multiple { streams, .. } => streams.first().map(|stream| stream.source_key),
    };
    let source_key = show_source_selection(ui, &protocol.sources, selected_source_key);

    match selection {
        StreamSelection::Single(stream) => {
            let data_key = show_single_field_selection(
                ui,
                field_selector_id,
                label,
                &hierarchy,
                stream.as_ref().map(|stream| stream.data_key),
            );
            *stream = data_key.map(|data_key| StreamKey { source_key, data_key });
        }
        StreamSelection::Multiple { streams, names } => {
            for stream in streams.iter_mut() {
                stream.source_key = source_key;
            }
            show_multiple_field_selection(ui, field_selector_id, label, &hierarchy, source_key, streams, names);
        }
    }
}

/// Renders the source picker and returns its selected key.
fn show_source_selection(
    ui: &mut Ui,
    sources: &[SourceDescriptor],
    selected_source_key: Option<SourceKey>,
) -> SourceKey {
    let selection_id = ui.id().with("source");
    let is_available = |key| sources.iter().any(|source| source.key == key);

    // Prefer the widget value then temporary state then the first source
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

    // Render source names while retaining their stable keys
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

    ui.mem().insert_temp(selection_id, source_key);
    source_key
}

/// Renders a hierarchical single-field selector and returns its selected key.
fn show_single_field_selection(
    ui: &mut Ui,
    combo_id: Id,
    label: &str,
    hierarchy: &SearchableComboBoxHierarchy<DataKey>,
    stream_data_key: Option<DataKey>,
) -> Option<DataKey> {
    let selection_id = ui.id().with("data");

    // Prefer the widget value before temporary incomplete selection state
    let mut selected = stream_data_key.or_else(|| ui.mem().get_temp::<Option<DataKey>>(selection_id).flatten());
    ui.label(label);
    ui.add(
        SearchableComboBox::new(combo_id, hierarchy, SingleSelection::new(&mut selected))
            .empty_selection_text("Select a field")
            .max_visible_rows(FIELD_SELECTOR_MAX_ROWS)
            .search_hint("Search messages and fields…")
            .empty_results_text("No matching messages or fields."),
    );
    ui.mem().insert_temp(selection_id, selected);
    selected
}

/// Renders a hierarchical multiple-field selector and persists changed values.
fn show_multiple_field_selection(
    ui: &mut Ui,
    combo_id: Id,
    label: &str,
    hierarchy: &SearchableComboBoxHierarchy<DataKey>,
    source_key: SourceKey,
    streams: &mut Vec<StreamKey>,
    mut names: Option<&mut Vec<String>>,
) {
    if let Some(names) = names.as_mut() {
        synchronize_stream_names(hierarchy, streams, names);
    }

    // Preserve unavailable values until the component reports a user edit
    let mut selected = streams.iter().map(|stream| stream.data_key).collect::<HashSet<_>>();
    ui.label(label);
    let response = ui.add(
        SearchableComboBox::new(combo_id, hierarchy, MultipleSelection::new(&mut selected))
            .empty_selection_text("Select fields")
            .max_visible_rows(FIELD_SELECTOR_MAX_ROWS)
            .search_hint("Search messages and fields…")
            .empty_results_text("No matching messages or fields.")
            .selection_nouns("field", "fields"),
    );
    if !response.changed() {
        return;
    }

    // Rebuild persisted streams in stable hierarchy traversal order
    streams.clear();
    if let Some(names) = names.as_mut() {
        names.clear();
    }
    for (data_key, name) in hierarchy.items() {
        if !selected.contains(data_key) {
            continue;
        }
        streams.push(StreamKey {
            source_key,
            data_key: *data_key,
        });
        if let Some(names) = names.as_mut() {
            names.push(name.to_owned());
        }
    }
}

/// Synchronizes saved names without changing stream order or unavailable names.
fn synchronize_stream_names(
    hierarchy: &SearchableComboBoxHierarchy<DataKey>,
    streams: &[StreamKey],
    names: &mut Vec<String>,
) {
    names.truncate(streams.len());

    for (position, stream) in streams.iter().enumerate() {
        if let Some(current_name) = hierarchy.label_for(&stream.data_key) {
            if let Some(saved_name) = names.get_mut(position) {
                current_name.clone_into(saved_name);
            } else {
                names.push(current_name.to_owned());
            }
        } else if position == names.len() {
            names.push("Unavailable field".to_owned());
        }
    }
}
