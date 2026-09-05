mod choices;

use egui::{Grid, Id, Ui, ahash::HashSet};
use segs_memory::MemoryExt;
use segs_ui::widgets::{
    MultipleSelection, SearchableComboBox, SearchableComboBoxHierarchy, SearchableComboBoxList, SingleSelection,
};

use crate::dataflow::{DataKey, SourceKey, StreamKey, adapter::DataAdapterInstance};

use self::choices::resolve_choices;

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
    let (source_choices, hierarchy) = resolve_choices(ui.ctx(), protocol, adapter.token());
    let source_selector_id = ui.make_persistent_id(("stream_source_selector", adapter.token()));
    let field_selector_id = ui.make_persistent_id(("stream_field_selector", adapter.token()));

    // Resolve the selected source from persisted widget state
    let selected_source_key = match &selection {
        StreamSelection::Single(stream) => stream.as_ref().map(|stream| stream.source_key),
        StreamSelection::Multiple { streams, .. } => streams.first().map(|stream| stream.source_key),
    };

    // Align source and stream controls to one shared label column
    Grid::new("stream_selection_grid")
        .num_columns(2)
        .spacing([8., 8.])
        .show(ui, |ui| {
            let source_key = show_source_selection(ui, source_selector_id, &source_choices, selected_source_key);
            ui.end_row();

            // Leave the final row open to avoid reserving trailing row spacing
            match selection {
                StreamSelection::Single(stream) => {
                    let data_key = show_single_field_selection(
                        ui,
                        field_selector_id,
                        label,
                        &hierarchy,
                        stream.as_ref().map(|stream| stream.data_key),
                        source_key.is_some(),
                    );
                    if let Some(source_key) = source_key {
                        *stream = data_key.map(|data_key| StreamKey { source_key, data_key });
                    }
                }
                StreamSelection::Multiple { streams, names } => {
                    if let Some(source_key) = source_key {
                        for stream in streams.iter_mut() {
                            stream.source_key = source_key;
                        }
                    }
                    show_multiple_field_selection(ui, field_selector_id, label, &hierarchy, source_key, streams, names);
                }
            }
        });
}

/// Renders the source picker and returns its selected key.
///
/// Returns `None` until the user selects an available source.
fn show_source_selection(
    ui: &mut Ui,
    combo_id: Id,
    choices: &SearchableComboBoxList<SourceKey>,
    selected_source_key: Option<SourceKey>,
) -> Option<SourceKey> {
    let selection_id = ui.id().with("source");
    let is_available = |key| choices.label_for(&key).is_some();

    // Prefer the widget value then temporary incomplete selection state
    let mut source_key = selected_source_key.filter(|key| is_available(*key)).or_else(|| {
        ui.mem()
            .get_temp::<Option<SourceKey>>(selection_id)
            .flatten()
            .filter(|key| is_available(*key))
    });

    // Render searchable source names while retaining their stable keys
    ui.label("Source");
    ui.add(
        SearchableComboBox::new(combo_id, choices, SingleSelection::new(&mut source_key))
            .empty_selection_text("Select a source")
            .search_hint("Search sources…")
            .empty_results_text("No matching sources."),
    );

    ui.mem().insert_temp(selection_id, source_key);
    source_key
}

/// Renders a hierarchical single-field selector and returns its selected key.
///
/// Returns the persisted or newly selected data key, or `None` when no stream
/// is selected. Selections made without a source remain temporary until a
/// source becomes available.
fn show_single_field_selection(
    ui: &mut Ui,
    combo_id: Id,
    label: &str,
    hierarchy: &SearchableComboBoxHierarchy<DataKey>,
    stream_data_key: Option<DataKey>,
    source_selected: bool,
) -> Option<DataKey> {
    let selection_id = ui.id().with("data");

    // Restore any incomplete selection before the persisted widget value
    let pending = ui.mem().remove_temp::<Option<DataKey>>(selection_id).flatten();
    let mut selected = pending.or(stream_data_key);
    ui.label(label);
    ui.add(
        SearchableComboBox::new(combo_id, hierarchy, SingleSelection::new(&mut selected))
            .empty_selection_text("Select a stream")
            .max_visible_rows(FIELD_SELECTOR_MAX_ROWS)
            .search_hint("Search streams…")
            .empty_results_text("No matching streams."),
    );
    if !source_selected {
        ui.mem().insert_temp(selection_id, selected);
    }
    selected
}

/// Renders a hierarchical multiple-field selector and persists changed values.
fn show_multiple_field_selection(
    ui: &mut Ui,
    combo_id: Id,
    label: &str,
    hierarchy: &SearchableComboBoxHierarchy<DataKey>,
    source_key: Option<SourceKey>,
    streams: &mut Vec<StreamKey>,
    mut names: Option<&mut Vec<String>>,
) {
    if let Some(names) = names.as_mut() {
        synchronize_stream_names(hierarchy, streams, names);
    }

    // Restore incomplete selections before the persisted widget values
    let selection_id = ui.id().with("data");
    let pending = ui.mem().remove_temp::<HashSet<DataKey>>(selection_id);
    let had_pending = pending.is_some();
    let mut selected = pending.unwrap_or_else(|| streams.iter().map(|stream| stream.data_key).collect());
    ui.label(label);
    let response = ui.add(
        SearchableComboBox::new(combo_id, hierarchy, MultipleSelection::new(&mut selected))
            .empty_selection_text("Select streams")
            .max_visible_rows(FIELD_SELECTOR_MAX_ROWS)
            .search_hint("Search streams…")
            .empty_results_text("No matching streams.")
            .selection_nouns("stream", "streams"),
    );
    let Some(source_key) = source_key else {
        ui.mem().insert_temp(selection_id, selected);
        return;
    };
    if !response.changed() && !had_pending {
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
            names.push("Unavailable stream".to_owned());
        }
    }
}
