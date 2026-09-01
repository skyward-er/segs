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
    widgets::{
        ExpandableSelector,
        buttons::{CheckState, Checkbox, RadioButton},
        text::TextEdit,
    },
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

/// Adapts exclusive and multiple field storage to the shared picker.
enum FieldSelection<'a> {
    Single(&'a mut Option<DataKey>),
    Multiple {
        selected_nodes: &'a mut [bool],
        unavailable_count: usize,
    },
}

/// Prefix counts used to derive structure checkbox states in constant time.
struct SelectionCounts {
    fields: Vec<usize>,
    selected: Vec<usize>,
}

/// Owns the shared context and mutable state used while rendering field-tree rows.
struct FieldTreeRenderer<'tree, 'selection> {
    index: &'tree DescriptorIndex,
    rows: &'tree [usize],
    counts: SelectionCounts,
    filtered: bool,
    expanded: &'tree mut HashSet<usize>,
    selection: &'tree mut FieldSelection<'selection>,
    changed: bool,
    row_height: f32,
}

/// Renders the source and stream controls for a widget data setting.
pub fn show(
    ui: &mut Ui,
    label: &str,
    stream: &mut Option<StreamKey>,
    protocol: Option<(&ProtocolDescriptor, &DataAdapterInstanceToken)>,
) {
    show_selection(ui, label, StreamSelection::Single(stream), protocol);
}

/// Renders the source and multiple-field controls for a widget data setting.
pub fn show_multiple(
    ui: &mut Ui,
    label: &str,
    streams: &mut Vec<StreamKey>,
    protocol: Option<(&ProtocolDescriptor, &DataAdapterInstanceToken)>,
) {
    show_selection(ui, label, StreamSelection::Multiple(streams), protocol);
}

/// The widget-owned stream storage accepted by the shared selector.
enum StreamSelection<'a> {
    Single(&'a mut Option<StreamKey>),
    Multiple(&'a mut Vec<StreamKey>),
}

fn show_selection(
    ui: &mut Ui,
    label: &str,
    selection: StreamSelection<'_>,
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
    if protocol.stream_messages.is_empty() {
        ui.weak("No streams available.");
        return;
    }

    // Render both selectors from the widget value or their temporary state
    let selected_source_key = match &selection {
        StreamSelection::Single(stream) => stream.as_ref().map(|stream| stream.source_key),
        StreamSelection::Multiple(streams) => streams.first().map(|stream| stream.source_key),
    };
    let source_key = show_source_selection(ui, &protocol.sources, selected_source_key);

    match selection {
        StreamSelection::Single(stream) => {
            let data_key = show_single_field_selection(
                ui,
                label,
                protocol,
                adapter_token,
                stream.as_ref().map(|stream| stream.data_key),
            );
            *stream = data_key.map(|data_key| StreamKey { source_key, data_key });
        }
        StreamSelection::Multiple(streams) => {
            for stream in streams.iter_mut() {
                stream.source_key = source_key;
            }
            show_multiple_field_selection(ui, label, protocol, adapter_token, source_key, streams);
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
fn show_single_field_selection(
    ui: &mut Ui,
    label: &str,
    protocol: &ProtocolDescriptor,
    adapter_token: &DataAdapterInstanceToken,
    stream_data_key: Option<DataKey>,
) -> Option<DataKey> {
    let selection_id = ui.id().with("data");
    let index = descriptor_index(ui, protocol, adapter_token);

    // Prefer the widget value before temporary incomplete selection state
    let mut selected_data_key =
        stream_data_key.or_else(|| ui.mem().get_temp::<Option<DataKey>>(selection_id).flatten());
    show_field_picker(
        ui,
        label,
        &index,
        adapter_token,
        FieldSelection::Single(&mut selected_data_key),
    );
    ui.mem().insert_temp(selection_id, selected_data_key);
    selected_data_key
}

/// Renders the shared picker for a widget's multiple stream keys.
fn show_multiple_field_selection(
    ui: &mut Ui,
    label: &str,
    protocol: &ProtocolDescriptor,
    adapter_token: &DataAdapterInstanceToken,
    source_key: SourceKey,
    streams: &mut Vec<StreamKey>,
) {
    let index = descriptor_index(ui, protocol, adapter_token);
    let mut selected_nodes = vec![false; index.nodes().len()];
    let mut unavailable_count = 0;
    for stream in streams.iter().copied() {
        if let Some(node_index) = index.field_node(stream.data_key) {
            selected_nodes[node_index] = true;
        } else {
            unavailable_count += 1;
        }
    }

    let changed = show_field_picker(
        ui,
        label,
        &index,
        adapter_token,
        FieldSelection::Multiple {
            selected_nodes: &mut selected_nodes,
            unavailable_count,
        },
    );
    if !changed {
        return;
    }

    streams.clear();
    streams.extend(index.nodes().iter().enumerate().filter_map(|(node_index, node)| {
        if !selected_nodes[node_index] {
            return None;
        }
        let IndexedNodeKind::Field { data_key } = node.kind else {
            return None;
        };
        Some(StreamKey { source_key, data_key })
    }));
}

/// Renders the compact summary and expandable hierarchy for either selection mode.
fn show_field_picker(
    ui: &mut Ui,
    label: &str,
    index: &Arc<DescriptorIndex>,
    adapter_token: &DataAdapterInstanceToken,
    mut selection: FieldSelection<'_>,
) -> bool {
    let picker_state_id = ui.id().with("field_picker_state");
    let query_id = ui.id().with("field_picker_query");
    let search_input_id = ui.id().with("field_picker_search_input");
    let mut picker_state = ui.mem().get_temp_or_default::<PickerState>(picker_state_id);
    let same_index = picker_state
        .index
        .upgrade()
        .is_some_and(|cached| Arc::ptr_eq(&cached, index));
    if !same_index {
        // Expansion rows belong to one descriptor traversal order
        picker_state.index = Arc::downgrade(index);
        picker_state.expanded.clear();
    }

    let summary_clicked = show_selection_summary(ui, label, index, &selection, &mut picker_state.open);
    let mut changed = false;

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

        changed |= show_field_tree(
            ui,
            index,
            adapter_token,
            &query,
            &mut picker_state.expanded,
            &mut selection,
        );
        ui.mem().insert_temp(query_id, query);
    }

    ui.mem().insert_temp(picker_state_id, picker_state);
    changed
}

/// Renders the field label and current selection as one expandable control,
///
/// Returns whether the control was clicked.
fn show_selection_summary(
    ui: &mut Ui,
    label: &str,
    index: &DescriptorIndex,
    selection: &FieldSelection<'_>,
    open: &mut bool,
) -> bool {
    let content_margin = ui.spacing().window_margin;

    let (summary, available) = match selection {
        FieldSelection::Single(selected_data_key) => match selected_data_key {
            Some(data_key) => match index.field_path(*data_key) {
                Some(path) => (path.to_owned(), true),
                None => ("Unavailable field".to_owned(), false),
            },
            None => ("No field selected".to_owned(), false),
        },
        FieldSelection::Multiple {
            selected_nodes,
            unavailable_count,
        } => {
            let count = selected_nodes.iter().filter(|selected| **selected).count() + unavailable_count;
            let summary = match count {
                0 => "No fields selected".to_owned(),
                1 => "1 field selected".to_owned(),
                count => format!("{count} fields selected"),
            };
            (summary, count > 0)
        }
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
    selection: &mut FieldSelection<'_>,
) -> bool {
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
        return false;
    }

    let counts = SelectionCounts::build(index, rows, filtered, selection);
    let row_height = ui.spacing().interact_size.y;
    let mut renderer = FieldTreeRenderer {
        index,
        rows,
        counts,
        filtered,
        expanded,
        selection,
        changed: false,
        row_height,
    };
    ScrollArea::vertical()
        .id_salt("field_tree_scroll")
        .max_height(field_tree_height(ui))
        .auto_shrink([false, true])
        .show_rows(ui, row_height, rows.len(), |ui, range| {
            for visible_index in range {
                renderer.show_descriptor_row(ui, visible_index);
            }
        });

    renderer.changed
}

impl FieldTreeRenderer<'_, '_> {
    /// Renders one structure or field row and applies its interaction.
    fn show_descriptor_row(&mut self, ui: &mut Ui, visible_index: usize) {
        let node_index = self.rows[visible_index];
        let node_kind = self.index.nodes()[node_index].kind;
        let animation_id = ui.id().with(("descriptor_node", node_index));
        let clickable = !self.filtered || matches!(node_kind, IndexedNodeKind::Field { .. });
        let sense = if clickable { Sense::click() } else { Sense::hover() };
        let (_, row_rect) = ui.allocate_space(vec2(ui.available_width(), self.row_height));
        // Virtualized interactions stay with row positions while animations stay with descriptor nodes
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

        match node_kind {
            IndexedNodeKind::Structure { subtree_end } => {
                self.show_structure_row(ui, row, visible_index, node_index, subtree_end);
            }
            IndexedNodeKind::Field { data_key } => {
                self.show_field_row(ui, row, node_index, data_key);
            }
        }
    }

    /// Renders a structure row and toggles it during normal browsing.
    fn show_structure_row(
        &mut self,
        ui: &mut Ui,
        row: DescriptorRow,
        visible_index: usize,
        node_index: usize,
        subtree_end: usize,
    ) {
        let node = &self.index.nodes()[node_index];
        let open = self.filtered || self.expanded.contains(&node_index);
        let indent = ui.spacing().indent * node.depth as f32;
        let caret_rect = Rect::from_center_size(
            pos2(row.rect.left() + indent + TREE_CARET_SIZE * 0.5, row.rect.center().y),
            Vec2::splat(TREE_CARET_SIZE),
        );
        let mut text_left = caret_rect.right() + ui.spacing().item_spacing.x;

        let selection_clicked = if matches!(self.selection, FieldSelection::Multiple { .. }) {
            let state = self
                .counts
                .structure_state(visible_index, node_index, subtree_end, self.filtered, self.rows);
            let checkbox_rect = Rect::from_center_size(
                pos2(text_left + Checkbox::SIZE.x * 0.5, row.rect.center().y),
                Checkbox::SIZE,
            );
            let checkbox_response = ui.interact(
                checkbox_rect,
                row.response.id.with("structure_selection"),
                Sense::click(),
            );
            let checkbox_response = Checkbox::show_state_at_with_selection_id(
                ui,
                state,
                checkbox_rect,
                checkbox_response,
                row.animation_id.with("structure_selection"),
            );
            checkbox_response.widget_info(|| match state {
                CheckState::Partial => WidgetInfo::labeled(WidgetType::Checkbox, ui.is_enabled(), &node.name),
                CheckState::Unchecked | CheckState::Checked => WidgetInfo::selected(
                    WidgetType::Checkbox,
                    ui.is_enabled(),
                    state == CheckState::Checked,
                    &node.name,
                ),
            });
            text_left = checkbox_rect.right() + ui.spacing().item_spacing.x;

            if checkbox_response.clicked() {
                let selected = state != CheckState::Checked;
                if self.filtered {
                    let range = structure_scope_range(visible_index, node_index, subtree_end, true, self.rows);
                    for &descendant_index in &self.rows[range] {
                        if let IndexedNodeKind::Field { data_key } = self.index.nodes()[descendant_index].kind {
                            self.changed |= self.selection.set_field(descendant_index, data_key, selected);
                        }
                    }
                } else {
                    for descendant_index in node_index + 1..subtree_end {
                        if let IndexedNodeKind::Field { data_key } = self.index.nodes()[descendant_index].kind {
                            self.changed |= self.selection.set_field(descendant_index, data_key, selected);
                        }
                    }
                }
            }
            checkbox_response.clicked()
        } else {
            false
        };

        if !self.filtered && row.response.clicked() && !selection_clicked && !self.expanded.remove(&node_index) {
            self.expanded.insert(node_index);
        }
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
    fn show_field_row(&mut self, ui: &mut Ui, row: DescriptorRow, node_index: usize, data_key: DataKey) {
        let node = &self.index.nodes()[node_index];
        let mut selected = self.selection.is_selected(node_index, data_key);
        let indent = ui.spacing().indent * node.depth as f32;
        let indicator_size = match self.selection {
            FieldSelection::Single(_) => RadioButton::SIZE,
            FieldSelection::Multiple { .. } => Checkbox::SIZE,
        };
        let indicator_rect = Rect::from_center_size(
            pos2(row.rect.left() + indent + indicator_size.x * 0.5, row.rect.center().y),
            indicator_size,
        );
        let response = match self.selection {
            FieldSelection::Single(_) => RadioButton::show_at_with_selection_id(
                ui,
                &mut selected,
                indicator_rect,
                row.response,
                row.animation_id.with("field_selection"),
            ),
            FieldSelection::Multiple { .. } => Checkbox::show_at_with_selection_id(
                ui,
                &mut selected,
                indicator_rect,
                row.response,
                row.animation_id.with("field_selection"),
            ),
        };
        let text_left = indicator_rect.right() + ui.spacing().item_spacing.x;
        paint_descriptor_text(ui, row.rect, text_left, &node.name, false);

        if response.clicked() {
            self.changed |= self.selection.set_field(node_index, data_key, selected);
        }
        let widget_type = match self.selection {
            FieldSelection::Single(_) => WidgetType::RadioButton,
            FieldSelection::Multiple { .. } => WidgetType::Checkbox,
        };
        response.widget_info(|| WidgetInfo::selected(widget_type, ui.is_enabled(), selected, &node.name));
    }
}

impl FieldSelection<'_> {
    fn is_selected(&self, node_index: usize, data_key: DataKey) -> bool {
        match self {
            Self::Single(selected_data_key) => **selected_data_key == Some(data_key),
            Self::Multiple { selected_nodes, .. } => selected_nodes[node_index],
        }
    }

    fn set_field(&mut self, node_index: usize, data_key: DataKey, selected: bool) -> bool {
        match self {
            Self::Single(selected_data_key) => {
                let next = selected.then_some(data_key);
                let changed = **selected_data_key != next;
                **selected_data_key = next;
                changed
            }
            Self::Multiple { selected_nodes, .. } => {
                let changed = selected_nodes[node_index] != selected;
                selected_nodes[node_index] = selected;
                changed
            }
        }
    }
}

impl SelectionCounts {
    fn build(index: &DescriptorIndex, rows: &[usize], filtered: bool, selection: &FieldSelection<'_>) -> Self {
        let count = if filtered { rows.len() } else { index.nodes().len() };
        let mut fields = Vec::with_capacity(count + 1);
        let mut selected = Vec::with_capacity(count + 1);
        fields.push(0);
        selected.push(0);

        if filtered {
            for &node_index in rows {
                Self::push_node(
                    &mut fields,
                    &mut selected,
                    node_index,
                    &index.nodes()[node_index],
                    selection,
                );
            }
        } else {
            for (node_index, node) in index.nodes().iter().enumerate() {
                Self::push_node(&mut fields, &mut selected, node_index, node, selection);
            }
        }

        Self { fields, selected }
    }

    fn push_node(
        fields: &mut Vec<usize>,
        selected: &mut Vec<usize>,
        node_index: usize,
        node: &IndexedNode,
        selection: &FieldSelection<'_>,
    ) {
        let IndexedNodeKind::Field { data_key } = node.kind else {
            fields.push(*fields.last().expect("prefix counts have an initial value"));
            selected.push(*selected.last().expect("prefix counts have an initial value"));
            return;
        };
        fields.push(fields.last().expect("prefix counts have an initial value") + 1);
        selected.push(
            selected.last().expect("prefix counts have an initial value")
                + usize::from(selection.is_selected(node_index, data_key)),
        );
    }

    fn structure_state(
        &self,
        visible_index: usize,
        node_index: usize,
        subtree_end: usize,
        filtered: bool,
        rows: &[usize],
    ) -> CheckState {
        let range = structure_scope_range(visible_index, node_index, subtree_end, filtered, rows);
        let fields = self.fields[range.end] - self.fields[range.start];
        let selected = self.selected[range.end] - self.selected[range.start];
        match selected {
            0 => CheckState::Unchecked,
            selected if selected == fields => CheckState::Checked,
            _ => CheckState::Partial,
        }
    }
}

fn structure_scope_range(
    visible_index: usize,
    node_index: usize,
    subtree_end: usize,
    filtered: bool,
    rows: &[usize],
) -> std::ops::Range<usize> {
    if filtered {
        let start = visible_index + 1;
        let end = rows.partition_point(|row_index| *row_index < subtree_end);
        start..end
    } else {
        node_index + 1..subtree_end
    }
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
    let index = Arc::new(DescriptorIndex::build(protocol));
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
