mod descriptor;
mod search;

use std::{
    collections::HashSet,
    f32::consts::FRAC_PI_2,
    sync::{Arc, Weak},
};

use egui::{
    ComboBox, CursorIcon, Id, Rect, Response, RichText, ScrollArea, Sense, TextStyle, TextWrapMode, Ui, Vec2,
    WidgetInfo, WidgetText, WidgetType, pos2, vec2,
};
use segs_assets::icons::{CaretDown, Icon};
use segs_memory::MemoryExt;
use segs_ui::{
    style::CtxStyleExt,
    widgets::{ExpandableSelector, buttons::Checkbox, text::TextEdit},
};

use crate::dataflow::{
    DataKey, SourceKey, StreamKey,
    adapter::DataAdapterInstanceToken,
    protocol::{ProtocolDescriptor, SourceDescriptor},
};

use self::{
    descriptor::{DescriptorIndex, IndexedNode, IndexedNodeKind},
    search::resolve_search_cache,
};

const DESCRIPTOR_INDEX_ID: &str = "stream_selector_descriptor_index";
const TREE_MIN_HEIGHT: f32 = 96.;
const TREE_MAX_HEIGHT: f32 = 280.;
const TREE_VIEWPORT_FRACTION: f32 = 0.4;
const TREE_CARET_SIZE: f32 = 14.;
const TREE_ROW_CORNER_RADIUS: f32 = 2.;

/// Stores the expandable picker state for one widget data setting.
#[derive(Clone, Debug, Default)]
struct PickerState {
    open: bool,
    index: Weak<DescriptorIndex>,
    expanded: HashSet<usize>,
}

/// Associates the reusable descriptor index with one installed adapter.
#[derive(Clone)]
struct CachedDescriptorIndex {
    adapter_token: DataAdapterInstanceToken,
    index: Arc<DescriptorIndex>,
}

/// Stores one tree row's layout, interaction, and animation identity.
struct DescriptorRow {
    rect: Rect,
    response: Response,
    animation_id: Id,
}

/// Renders the source and stream controls for a widget data setting.
pub fn show(
    ui: &mut Ui,
    label: &str,
    stream: &mut Option<StreamKey>,
    protocol: Option<(&ProtocolDescriptor, &DataAdapterInstanceToken)>,
) {
    // Validate the protocol before rendering controls that index its descriptors
    let Some((protocol, adapter_token)) = protocol else {
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

    // Render both selectors from the widget value or their temporary state
    let source_key = show_source_selection(ui, &protocol.sources, stream.as_ref().map(|stream| stream.source_key));
    let data_key = show_field_selection(
        ui,
        label,
        protocol,
        adapter_token,
        stream.as_ref().map(|stream| stream.data_key),
    );

    // Apply the complete selection to the widget
    *stream = data_key.map(|data_key| StreamKey { source_key, data_key });
}

/// Renders the source picker and returns its selected key.
fn show_source_selection(
    ui: &mut Ui,
    sources: &[SourceDescriptor],
    selected_source_key: Option<SourceKey>,
) -> SourceKey {
    let selection_id = ui.id().with("source");
    // Check temporary and widget keys against the current sources
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

    // Retain source selection before the stream is complete
    ui.mem().insert_temp(selection_id, source_key);
    source_key
}

/// Renders the compact field summary and optional searchable picker.
fn show_field_selection(
    ui: &mut Ui,
    label: &str,
    protocol: &ProtocolDescriptor,
    adapter_token: &DataAdapterInstanceToken,
    stream_data_key: Option<DataKey>,
) -> Option<DataKey> {
    let selection_id = ui.id().with("data");
    let picker_state_id = ui.id().with("field_picker_state");
    let query_id = ui.id().with("field_picker_query");
    let search_input_id = ui.id().with("field_picker_search_input");
    // Reuse the normalized index until the installed adapter changes
    let index = descriptor_index(ui, protocol, adapter_token);

    // Prefer the widget value before temporary incomplete selection state
    let mut selected_data_key =
        stream_data_key.or_else(|| ui.mem().get_temp::<Option<DataKey>>(selection_id).flatten());
    let mut picker_state = ui.mem().get_temp_or_default::<PickerState>(picker_state_id);
    let same_index = picker_state
        .index
        .upgrade()
        .is_some_and(|cached| Arc::ptr_eq(&cached, &index));
    if !same_index {
        // Expansion rows belong to one descriptor traversal order
        picker_state.index = Arc::downgrade(&index);
        picker_state.expanded.clear();
    }

    let summary_clicked = show_selection_summary(ui, label, &index, selected_data_key, &mut picker_state.open);

    if picker_state.open {
        let mut query = ui.mem().get_temp_or_default::<String>(query_id);
        let response = ui.add(
            TextEdit::singleline(&mut query)
                .id_salt(search_input_id)
                .hint_text("Search messages and fields…")
                .desired_width(ui.available_width()),
        );
        if summary_clicked {
            response.request_focus();
        }
        ui.add_space(6.);

        show_field_tree(
            ui,
            &index,
            adapter_token,
            &query,
            &mut picker_state.expanded,
            &mut selected_data_key,
        );
        ui.mem().insert_temp(query_id, query);
    }

    ui.mem().insert_temp(picker_state_id, picker_state);
    ui.mem().insert_temp(selection_id, selected_data_key);
    selected_data_key
}

/// Renders the field label and current selection as one expandable control,
///
/// Returns whether the control was clicked.
fn show_selection_summary(
    ui: &mut Ui,
    label: &str,
    index: &DescriptorIndex,
    selected_data_key: Option<DataKey>,
    open: &mut bool,
) -> bool {
    let content_margin = ui.spacing().window_margin;

    let (summary, available) = match selected_data_key {
        Some(data_key) => match index.field_path(data_key) {
            Some(path) => (path, true),
            None => ("Unavailable field", false),
        },
        None => ("No field selected", false),
    };

    ui.add(
        ExpandableSelector::new(label, summary, open)
            .preview_weak(!available)
            .horizontal_bleed(content_margin),
    )
    .clicked()
}

/// Renders the filtered or browsable hierarchy through fixed-height rows.
fn show_field_tree(
    ui: &mut Ui,
    index: &Arc<DescriptorIndex>,
    adapter_token: &DataAdapterInstanceToken,
    query: &str,
    expanded: &mut HashSet<usize>,
    selected_data_key: &mut Option<DataKey>,
) {
    let filtered = !query.trim().is_empty();
    let visible_rows;
    let cached_search;

    let rows = if filtered {
        cached_search = resolve_search_cache(ui, index, adapter_token, query);
        cached_search.rows()
    } else {
        visible_rows = index.visible_rows(expanded);
        &visible_rows
    };

    if rows.is_empty() {
        ui.weak("No matching messages or fields.");
        return;
    }

    let row_height = ui.spacing().interact_size.y;
    ScrollArea::vertical()
        .id_salt("field_tree_scroll")
        .max_height(field_tree_height(ui))
        .auto_shrink([false, true])
        .show_rows(ui, row_height, rows.len(), |ui, range| {
            for visible_index in range {
                let node_index = rows[visible_index];
                show_descriptor_row(
                    ui,
                    visible_index,
                    node_index,
                    &index.nodes()[node_index],
                    filtered,
                    expanded,
                    selected_data_key,
                    row_height,
                );
            }
        });
}

/// Renders one structure or field row and applies its interaction.
fn show_descriptor_row(
    ui: &mut Ui,
    visible_index: usize,
    node_index: usize,
    node: &IndexedNode,
    filtered: bool,
    expanded: &mut HashSet<usize>,
    selected_data_key: &mut Option<DataKey>,
    row_height: f32,
) {
    let animation_id = ui.id().with(("descriptor_node", node_index));
    let clickable = !filtered || matches!(node.kind, IndexedNodeKind::Field { .. });
    let sense = if clickable { Sense::click() } else { Sense::hover() };
    let (_, row_rect) = ui.allocate_space(vec2(ui.available_width(), row_height));
    // Keep interaction identity tied to its row position
    let interaction_id = ui.id().with(("descriptor_row", visible_index));
    let response = ui.interact(row_rect, interaction_id, sense);
    let response = if clickable {
        response.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        response
    };
    paint_descriptor_row_background(ui, row_rect, &response);
    let row = DescriptorRow {
        rect: row_rect,
        response,
        animation_id,
    };

    match node.kind {
        IndexedNodeKind::Structure { .. } => {
            show_structure_row(ui, row, node_index, node, filtered, expanded);
        }
        IndexedNodeKind::Field { data_key } => {
            show_field_row(ui, row, node, data_key, selected_data_key);
        }
    }
}

/// Renders a structure row and toggles it during normal browsing.
fn show_structure_row(
    ui: &mut Ui,
    row: DescriptorRow,
    node_index: usize,
    node: &IndexedNode,
    filtered: bool,
    expanded: &mut HashSet<usize>,
) {
    if row.response.clicked() && !expanded.remove(&node_index) {
        expanded.insert(node_index);
    }
    let open = filtered || expanded.contains(&node_index);
    let indent = ui.spacing().indent * node.depth as f32;
    let caret_rect = Rect::from_center_size(
        pos2(row.rect.left() + indent + TREE_CARET_SIZE * 0.5, row.rect.center().y),
        Vec2::splat(TREE_CARET_SIZE),
    );
    let text_left = caret_rect.right() + ui.spacing().item_spacing.x;
    paint_descriptor_text(ui, row.rect, text_left, &node.name, true);

    let openness = ui
        .ctx()
        .animate_bool_responsive(row.animation_id.with("caret_openness"), open);
    CaretDown::solid()
        .to_image()
        .tint(ui.visuals().text_color())
        .rotate(-FRAC_PI_2 * (1. - openness), Vec2::splat(0.5))
        .fit_to_exact_size(caret_rect.size())
        .paint_at(ui, caret_rect);
    row.response
        .widget_info(|| WidgetInfo::labeled(WidgetType::CollapsingHeader, ui.is_enabled(), &node.name));
}

/// Renders a field row and toggles its selection from the complete row.
fn show_field_row(
    ui: &mut Ui,
    row: DescriptorRow,
    node: &IndexedNode,
    data_key: DataKey,
    selected_data_key: &mut Option<DataKey>,
) {
    let mut selected = *selected_data_key == Some(data_key);
    let indent = ui.spacing().indent * node.depth as f32;
    let checkbox_rect = Rect::from_center_size(
        pos2(row.rect.left() + indent + Checkbox::SIZE.x * 0.5, row.rect.center().y),
        Checkbox::SIZE,
    );
    let response = Checkbox::show_at(ui, &mut selected, checkbox_rect, row.response);
    let text_left = checkbox_rect.right() + ui.spacing().item_spacing.x;
    paint_descriptor_text(ui, row.rect, text_left, &node.name, false);

    if response.clicked() {
        *selected_data_key = selected.then_some(data_key);
    }
    response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, ui.is_enabled(), selected, &node.name));
}

/// Paints transient interaction feedback across a complete tree row.
fn paint_descriptor_row_background(ui: &Ui, rect: Rect, response: &Response) {
    let fill = if response.is_pointer_button_down_on() {
        Some(ui.app_style().widgets.active.bg_fill)
    } else if response.hovered() || response.has_focus() {
        Some(ui.app_style().widgets.hovered.bg_fill)
    } else {
        None
    };
    if let Some(fill) = fill {
        ui.painter().rect_filled(rect, TREE_ROW_CORNER_RADIUS, fill);
    }
}

/// Paints one truncated tree label without text-selection interaction.
fn paint_descriptor_text(ui: &Ui, row_rect: Rect, text_left: f32, text: &str, strong: bool) {
    let text = if strong {
        WidgetText::from(RichText::new(text).strong())
    } else {
        WidgetText::from(text)
    };
    let width = (row_rect.right() - text_left).max(0.);
    let galley = text.into_galley(ui, Some(TextWrapMode::Truncate), width, TextStyle::Body);
    let text_pos = pos2(text_left, row_rect.center().y - galley.size().y * 0.5);
    ui.painter().galley(text_pos, galley, ui.visuals().text_color());
}

/// Returns the index for the installed adapter's protocol descriptor.
fn descriptor_index(
    ui: &Ui,
    protocol: &ProtocolDescriptor,
    adapter_token: &DataAdapterInstanceToken,
) -> Arc<DescriptorIndex> {
    let id = Id::new(DESCRIPTOR_INDEX_ID);
    if let Some(cached) = ui.mem().get_temp::<CachedDescriptorIndex>(id)
        && &cached.adapter_token == adapter_token
    {
        return cached.index;
    }

    // A new wrapper token always denotes a newly installed adapter
    let index = Arc::new(DescriptorIndex::build(&protocol.messages));
    ui.mem().insert_temp(
        id,
        CachedDescriptorIndex {
            adapter_token: adapter_token.clone(),
            index: index.clone(),
        },
    );
    index
}

/// Computes a responsive cap for the nested field tree.
fn field_tree_height(ui: &Ui) -> f32 {
    let clip_rect = ui.clip_rect();
    let preferred = (clip_rect.height() * TREE_VIEWPORT_FRACTION).clamp(TREE_MIN_HEIGHT, TREE_MAX_HEIGHT);
    let remaining = clip_rect.bottom() - ui.cursor().top();
    preferred.min(remaining.max(TREE_MIN_HEIGHT))
}
