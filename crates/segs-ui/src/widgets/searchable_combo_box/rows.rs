use std::{collections::HashSet, f32::consts::FRAC_PI_2, hash::Hash, ops::Range, sync::Arc};

use egui::{
    Align, CursorIcon, Frame, Id, Layout, Margin, Rect, Response, ScrollArea, Sense, TextStyle, TextWrapMode, Ui, Vec2,
    WidgetInfo, WidgetText, WidgetType, pos2, vec2,
};
use segs_assets::icons::{CaretDown, Check, Icon};

use crate::{
    style::CtxStyleExt,
    widgets::buttons::{CheckState, Checkbox},
};

use super::{
    INDICATOR_SIZE,
    choices::{ChoiceSource, normalize_query},
    selection::SelectionState,
};

/// Describes the keyboard action targeted at the highlighted row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum RowAction {
    /// Leaves the highlighted row unchanged.
    #[default]
    None,
    /// Activates the highlighted row.
    Activate,
    /// Toggles the highlighted multiple-selection row.
    ToggleSelection,
}

/// Reports mutations produced while rendering visible rows.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RowChanges {
    /// Whether the bound selection changed.
    pub(super) selection: bool,
    /// Whether the user attempted a selection action.
    pub(super) selection_event: bool,
    /// Whether hierarchy expansion changed.
    pub(super) expansion: bool,
}

/// Holds immutable sizing values for one visible-list pass.
pub(super) struct RowsLayout {
    /// Maximum number of rows shown without scrolling.
    pub(super) max_visible_rows: usize,
    /// Vertical spacing between adjacent rows.
    pub(super) row_spacing: f32,
    /// Margin around the scroll area's row content.
    pub(super) list_margin: Margin,
    /// Margin used to align the empty-result message with row text.
    pub(super) text_horizontal_margin: Margin,
}

/// Holds keyboard navigation input for one visible-list pass.
pub(super) struct RowsNavigation {
    /// Whether navigation must restart at the first row.
    pub(super) reset_highlight: bool,
    /// Signed movement requested for the highlighted row.
    pub(super) move_delta: isize,
    /// Action requested for the highlighted row.
    pub(super) action: RowAction,
}

/// Borrows the popup state mutated by row rendering.
pub(super) struct RowsState<'a> {
    /// Expanded hierarchy group indices.
    pub(super) expanded: &'a mut HashSet<usize>,
    /// Index of the highlighted entry within the visible-row list.
    pub(super) highlighted_row: &'a mut Option<usize>,
    /// Last observed vertical scroll offset.
    pub(super) scroll_offset: &'a mut f32,
    /// Last observed viewport height.
    pub(super) viewport_height: &'a mut f32,
}

/// Renders the virtualized visible portion of a searchable combo-box list.
pub(super) struct Rows<'a, C, S> {
    /// Immutable choice source rendered by the list.
    pub(super) choices: &'a C,
    /// Mutable selection strategy receiving row actions.
    pub(super) selection: &'a mut S,
    /// Stable identity of the combo-box popup owning these rows.
    pub(super) component_id: Id,
    /// Whether choices contain expandable hierarchy groups.
    pub(super) hierarchical: bool,
    /// Flattened choice indices currently visible after filtering and expansion.
    pub(super) visible: &'a [usize],
    /// Whether the visible rows represent a non-empty search query.
    pub(super) filtered: bool,
    /// Message displayed when no rows match the query.
    pub(super) empty_results_text: &'a str,
    /// Sizing values for the list pass.
    pub(super) layout: RowsLayout,
    /// Keyboard input for the list pass.
    pub(super) navigation: RowsNavigation,
    /// Mutable popup state retained between frames.
    pub(super) state: RowsState<'a>,
}

impl<C, S> Rows<'_, C, S>
where
    C: ChoiceSource,
    C::Value: Clone + Eq + Hash,
    S: SelectionState<C::Value>,
{
    /// Renders the current visible rows and returns their combined mutations.
    pub(super) fn show(self, ui: &mut Ui) -> RowChanges {
        // Preserve one result-area identity while its empty or populated contents change
        let results_id = self.component_id.with("searchable_combo_box_results");
        ui.push_id(results_id, |ui| self.show_contents(ui)).inner
    }

    /// Renders the empty message or virtualized rows inside the stable result area.
    fn show_contents(self, ui: &mut Ui) -> RowChanges {
        let Self {
            choices,
            selection,
            component_id,
            hierarchical,
            visible,
            filtered,
            empty_results_text,
            layout,
            navigation,
            state,
        } = self;

        if visible.is_empty() {
            *state.highlighted_row = None;
            *state.scroll_offset = 0.;
            *state.viewport_height = 0.;
            show_empty_results(
                ui,
                empty_results_text,
                layout.list_margin,
                layout.text_horizontal_margin,
            );
            return RowChanges::default();
        }

        // Apply keyboard navigation to the filtered-row coordinate space
        let reset_highlight =
            navigation.reset_highlight || state.highlighted_row.is_none_or(|row| row >= visible.len());
        if reset_highlight {
            *state.highlighted_row = Some(0);
        }
        if navigation.move_delta != 0 {
            let current = state.highlighted_row.unwrap_or(0);
            *state.highlighted_row = Some(
                current
                    .saturating_add_signed(navigation.move_delta)
                    .min(visible.len() - 1),
            );
        }
        let highlighted = state.highlighted_row.unwrap_or(0);

        // Size the viewport to the row limit and current result count
        let row_height = ui.spacing().interact_size.y;
        let top_margin = f32::from(layout.list_margin.top);
        let bottom_margin = f32::from(layout.list_margin.bottom);
        let visible_count = visible.len().min(layout.max_visible_rows);
        let list_height = row_height * visible_count as f32
            + layout.row_spacing * visible_count.saturating_sub(1) as f32
            + top_margin
            + bottom_margin;
        let mut scroll_area = ScrollArea::vertical()
            .id_salt("searchable_combo_box_list")
            .content_margin(layout.list_margin)
            .max_height(list_height)
            .min_scrolled_height(0.)
            .auto_shrink([false, true]);

        // Move the viewport only when keyboard navigation crosses its leading edge
        if reset_highlight {
            *state.scroll_offset = 0.;
            scroll_area = scroll_area.vertical_scroll_offset(0.);
        } else if navigation.move_delta != 0 {
            let row_pitch = row_height + layout.row_spacing;
            let row_top = top_margin + highlighted as f32 * row_pitch;
            let reveal_top = if highlighted == 0 { 0. } else { row_top };
            let reveal_bottom = row_top
                + row_height
                + if highlighted == visible.len() - 1 {
                    bottom_margin
                } else {
                    0.
                };
            let viewport_height = if *state.viewport_height > 0. {
                state.viewport_height.min(list_height)
            } else {
                list_height
            };
            let vertical_offset = if navigation.move_delta < 0 && reveal_top < *state.scroll_offset {
                reveal_top
            } else if navigation.move_delta > 0 && reveal_bottom > *state.scroll_offset + viewport_height {
                reveal_bottom - viewport_height
            } else {
                *state.scroll_offset
            };
            scroll_area = scroll_area.vertical_scroll_offset(vertical_offset.max(0.));
        }

        // Precompute group counts and render only rows intersecting the viewport
        let counts =
            (hierarchical && S::MULTIPLE).then(|| SelectionCounts::build(choices, selection, visible, filtered));
        let pointer_moved = ui.input(|input| input.pointer.delta() != Vec2::ZERO);
        let list_width = ui.available_width();
        let mut renderer = RowRenderer {
            choices,
            selection,
            component_id,
            hierarchical,
            visible,
            filtered,
            counts: counts.as_ref(),
            expanded: state.expanded,
        };
        let keyboard_changes = renderer.apply_keyboard_action(highlighted, visible[highlighted], navigation.action);
        let list_output = ui
            .allocate_ui_with_layout(vec2(list_width, list_height), Layout::top_down(Align::Min), |ui| {
                // Contain scroll content and remove the shadow-like overflow fade
                ui.visuals_mut().clip_rect_margin = 0.;
                ui.spacing_mut().scroll.fade.strength = 0.;
                ui.spacing_mut().item_spacing.y = layout.row_spacing;
                ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                scroll_area.show_rows(ui, row_height, visible.len(), |ui, range| {
                    let mut changes = RowChanges::default();
                    for visible_index in range {
                        let node_index = visible[visible_index];
                        let is_highlighted = *state.highlighted_row == Some(visible_index);
                        let (row_changes, response) = renderer.show(ui, visible_index, node_index, is_highlighted);
                        if pointer_moved && response.hovered() {
                            *state.highlighted_row = Some(visible_index);
                        }
                        changes.merge(row_changes);
                    }
                    changes
                })
            })
            .inner;
        *state.scroll_offset = list_output.state.offset.y;
        *state.viewport_height = list_output.inner_rect.height();
        let mut changes = keyboard_changes;
        changes.merge(list_output.inner);
        changes
    }
}

impl RowChanges {
    fn merge(&mut self, other: Self) {
        self.selection |= other.selection;
        self.selection_event |= other.selection_event;
        self.expansion |= other.expansion;
    }
}

struct RowRenderer<'a, C, S> {
    choices: &'a C,
    selection: &'a mut S,
    component_id: Id,
    hierarchical: bool,
    visible: &'a [usize],
    filtered: bool,
    counts: Option<&'a SelectionCounts>,
    expanded: &'a mut HashSet<usize>,
}

struct RowUi {
    visible_index: usize,
    node_index: usize,
    rect: Rect,
    response: Response,
    animation_id: Id,
}

impl<C, S> RowRenderer<'_, C, S>
where
    C: ChoiceSource,
    C::Value: Clone + Eq + Hash,
    S: SelectionState<C::Value>,
{
    /// Applies an action to a logical row without requiring that row to be painted.
    fn apply_keyboard_action(&mut self, visible_index: usize, node_index: usize, action: RowAction) -> RowChanges {
        match (self.choices.group_end(node_index), action) {
            (_, RowAction::None) => RowChanges::default(),
            (Some(_), RowAction::Activate) if !self.filtered => self.toggle_group_expansion(node_index),
            (Some(subtree_end), RowAction::ToggleSelection) if S::MULTIPLE => {
                self.toggle_group_selection(visible_index, node_index, subtree_end)
            }
            (None, RowAction::Activate | RowAction::ToggleSelection) => {
                let Some(value) = self.choices.value(node_index).cloned() else {
                    return RowChanges::default();
                };
                self.toggle_item_selection(&value)
            }
            _ => RowChanges::default(),
        }
    }

    fn show(
        &mut self,
        ui: &mut Ui,
        visible_index: usize,
        node_index: usize,
        highlighted: bool,
    ) -> (RowChanges, Response) {
        let row_height = ui.spacing().interact_size.y;
        let row_width = ui.available_width();
        let response = ui
            .allocate_response(vec2(row_width, row_height), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        let rect = response.rect;
        paint_row_background(ui, rect, &response, highlighted);

        let row = RowUi {
            visible_index,
            node_index,
            rect,
            response,
            animation_id: self.component_id.with(("searchable_combo_box_choice", node_index)),
        };
        let changes = if let Some(subtree_end) = self.choices.group_end(node_index) {
            self.show_group(ui, &row, subtree_end)
        } else if let Some(value) = self.choices.value(node_index) {
            self.show_item(ui, &row, value)
        } else {
            RowChanges::default()
        };
        (changes, row.response)
    }

    fn show_group(&mut self, ui: &mut Ui, row: &RowUi, subtree_end: usize) -> RowChanges {
        let label = self.choices.label(row.node_index);
        let indent = ui.spacing().button_padding.x + ui.spacing().indent * self.choices.depth(row.node_index) as f32;
        let caret_rect = Rect::from_center_size(
            pos2(
                row.rect.right() - ui.spacing().button_padding.x - INDICATOR_SIZE * 0.5,
                row.rect.center().y,
            ),
            Vec2::splat(INDICATOR_SIZE),
        );
        let mut text_left = row.rect.left() + indent;
        let mut changes = RowChanges::default();

        // Give multiple-selection groups a distinct leading checkbox target
        if S::MULTIPLE {
            let counts = self.counts.expect("multiple hierarchy rows require selection counts");
            let range = group_scope_range(
                row.visible_index,
                row.node_index,
                subtree_end,
                self.filtered,
                self.visible,
            );
            let state = counts.group_state(range.clone());
            let checkbox_rect = Rect::from_center_size(
                pos2(text_left + Checkbox::SIZE.x * 0.5, row.rect.center().y),
                Checkbox::SIZE,
            );
            let checkbox_response = ui.interact(checkbox_rect, row.response.id.with("group_selection"), Sense::click());
            let checkbox_response = Checkbox::show_state_at_with_selection_id(
                ui,
                state,
                checkbox_rect,
                checkbox_response,
                row.animation_id.with("group_selection"),
            );
            checkbox_response.widget_info(|| match state {
                CheckState::Partial => WidgetInfo::labeled(WidgetType::Checkbox, ui.is_enabled(), label),
                CheckState::Unchecked | CheckState::Checked => WidgetInfo::selected(
                    WidgetType::Checkbox,
                    ui.is_enabled(),
                    state == CheckState::Checked,
                    label,
                ),
            });
            text_left = checkbox_rect.right() + ui.spacing().item_spacing.x;

            if checkbox_response.clicked() {
                changes.merge(self.set_group_selection(range, state != CheckState::Checked));
            }
        }

        // Paint the header label and trailing animated expansion caret
        let galley = WidgetText::from(egui::RichText::new(label).strong()).into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            TextStyle::Body,
        );
        paint_text(ui, row.rect, text_left, &galley);
        let open = self.filtered || self.expanded.contains(&row.node_index);
        let openness = ui
            .ctx()
            .animate_bool_responsive(row.animation_id.with("caret_openness"), open);
        CaretDown::solid()
            .to_image()
            .tint(ui.visuals().text_color())
            .rotate(-FRAC_PI_2 * (1. - openness), Vec2::splat(0.5))
            .fit_to_exact_size(caret_rect.size())
            .paint_at(ui, caret_rect);

        let activated = row.response.clicked();
        if !self.filtered && activated && !changes.selection_event {
            changes.merge(self.toggle_group_expansion(row.node_index));
        }
        row.response
            .widget_info(|| WidgetInfo::labeled(WidgetType::CollapsingHeader, ui.is_enabled(), label));
        changes
    }

    fn show_item(&mut self, ui: &mut Ui, row: &RowUi, value: &C::Value) -> RowChanges {
        let label = self.choices.label(row.node_index);
        let depth = self.choices.depth(row.node_index);
        let indent =
            ui.spacing().button_padding.x + ui.spacing().indent * usize::from(self.hierarchical) as f32 * depth as f32;
        let changes = if row.response.clicked() {
            self.toggle_item_selection(value)
        } else {
            RowChanges::default()
        };
        let selected = self.selection.is_selected(value);

        // Paint hierarchical multiple-selection checkboxes before their labels
        let text_left = if S::MULTIPLE && self.hierarchical {
            let checkbox_rect = Rect::from_center_size(
                pos2(row.rect.left() + indent + Checkbox::SIZE.x * 0.5, row.rect.center().y),
                Checkbox::SIZE,
            );
            paint_checkbox(ui, row, checkbox_rect, selected);
            checkbox_rect.right() + ui.spacing().item_spacing.x
        } else {
            let text_left = row.rect.left() + indent;
            if selected && !S::MULTIPLE {
                paint_single_check(ui, row, text_left, depth, self.hierarchical);
            } else if S::MULTIPLE {
                let checkbox_rect = Rect::from_center_size(
                    pos2(
                        row.rect.right() - ui.spacing().button_padding.x - Checkbox::SIZE.x * 0.5,
                        row.rect.center().y,
                    ),
                    Checkbox::SIZE,
                );
                paint_checkbox(ui, row, checkbox_rect, selected);
            }
            text_left
        };
        let galley =
            WidgetText::from(label).into_galley(ui, Some(TextWrapMode::Extend), f32::INFINITY, TextStyle::Body);
        paint_text(ui, row.rect, text_left, &galley);
        row.response.widget_info(|| {
            WidgetInfo::selected(
                if S::MULTIPLE {
                    WidgetType::Checkbox
                } else {
                    WidgetType::SelectableLabel
                },
                ui.is_enabled(),
                selected,
                label,
            )
        });
        changes
    }

    /// Toggles one selectable item while removing values absent from the choices.
    fn toggle_item_selection(&mut self, value: &C::Value) -> RowChanges {
        let selected = self.selection.is_selected(value);
        let mut available = |value: &C::Value| self.choices.value_index(value).is_some();
        let mut changed = self.selection.retain_available(&mut available);
        changed |= self
            .selection
            .set_selected(value, if S::MULTIPLE { !selected } else { true });
        RowChanges {
            selection: changed,
            selection_event: true,
            expansion: false,
        }
    }

    /// Toggles all selectable descendants of one hierarchy group.
    fn toggle_group_selection(&mut self, visible_index: usize, node_index: usize, subtree_end: usize) -> RowChanges {
        let range = group_scope_range(visible_index, node_index, subtree_end, self.filtered, self.visible);
        let state = self
            .counts
            .expect("multiple hierarchy rows require selection counts")
            .group_state(range.clone());
        self.set_group_selection(range, state != CheckState::Checked)
    }

    /// Sets all selectable rows in a hierarchy range to one state.
    fn set_group_selection(&mut self, range: Range<usize>, selected: bool) -> RowChanges {
        let mut available = |value: &C::Value| self.choices.value_index(value).is_some();
        let mut changed = self.selection.retain_available(&mut available);
        for index in range {
            let descendant_index = if self.filtered { self.visible[index] } else { index };
            if let Some(value) = self.choices.value(descendant_index) {
                changed |= self.selection.set_selected(value, selected);
            }
        }
        RowChanges {
            selection: changed,
            selection_event: true,
            expansion: false,
        }
    }

    /// Toggles one hierarchy group's expanded state.
    fn toggle_group_expansion(&mut self, node_index: usize) -> RowChanges {
        if !self.expanded.remove(&node_index) {
            self.expanded.insert(node_index);
        }
        RowChanges {
            selection: false,
            selection_event: false,
            expansion: true,
        }
    }
}

struct SelectionCounts {
    fields: Vec<usize>,
    selected: Vec<usize>,
}

impl SelectionCounts {
    fn build<C, S>(choices: &C, selection: &S, rows: &[usize], filtered: bool) -> Self
    where
        C: ChoiceSource,
        S: SelectionState<C::Value>,
    {
        let count = if filtered { rows.len() } else { choices.len() };
        let mut fields = Vec::with_capacity(count + 1);
        let mut selected = Vec::with_capacity(count + 1);
        fields.push(0);
        selected.push(0);

        for position in 0..count {
            let node_index = if filtered { rows[position] } else { position };
            let value = choices.value(node_index);
            fields.push(fields[position] + usize::from(value.is_some()));
            selected.push(selected[position] + usize::from(value.is_some_and(|value| selection.is_selected(value))));
        }
        Self { fields, selected }
    }

    fn group_state(&self, range: Range<usize>) -> CheckState {
        let fields = self.fields[range.end] - self.fields[range.start];
        let selected = self.selected[range.end] - self.selected[range.start];
        match selected {
            0 => CheckState::Unchecked,
            selected if selected == fields => CheckState::Checked,
            _ => CheckState::Partial,
        }
    }
}

/// Resolves the choice indices displayed for the current query and expansion state.
pub(super) fn resolve_visible_rows<C>(
    choices: &C,
    hierarchical: bool,
    query: &str,
    expanded: &HashSet<usize>,
    rows: &mut Vec<usize>,
) where
    C: ChoiceSource,
{
    rows.clear();
    let query = normalize_query(query);
    if query.is_empty() {
        let mut index = 0;
        while index < choices.len() {
            rows.push(index);
            index = if hierarchical {
                choices
                    .group_end(index)
                    .filter(|_| !expanded.contains(&index))
                    .unwrap_or(index + 1)
            } else {
                index + 1
            };
        }
        return;
    }

    if !hierarchical {
        rows.extend((0..choices.len()).filter(|index| choices.normalized_label(*index).contains(&query)));
        return;
    }

    // Mark matches, their ancestors, and directly matched group subtrees
    let mut included = vec![false; choices.len()];
    for index in 0..choices.len() {
        if !choices.normalized_label(index).contains(&query) {
            continue;
        }
        let mut ancestor = Some(index);
        while let Some(index) = ancestor {
            included[index] = true;
            ancestor = choices.parent(index);
        }
        if let Some(subtree_end) = choices.group_end(index) {
            included[index..subtree_end].fill(true);
        }
    }
    rows.extend(
        included
            .into_iter()
            .enumerate()
            .filter_map(|(index, included)| included.then_some(index)),
    );
}

fn show_empty_results(ui: &mut Ui, text: &str, list_margin: Margin, text_margin: Margin) {
    // Match the empty message to the shared list and row-label insets
    Frame::NONE
        .inner_margin(Margin {
            left: text_margin.left,
            right: text_margin.right,
            top: list_margin.top,
            bottom: list_margin.bottom,
        })
        .show(ui, |ui| {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
            ui.set_min_height(ui.spacing().interact_size.y);
            ui.weak(text);
        });
}

fn group_scope_range(
    visible_index: usize,
    node_index: usize,
    subtree_end: usize,
    filtered: bool,
    rows: &[usize],
) -> Range<usize> {
    if filtered {
        visible_index + 1..rows.partition_point(|row_index| *row_index < subtree_end)
    } else {
        node_index + 1..subtree_end
    }
}

fn paint_checkbox(ui: &mut Ui, row: &RowUi, rect: Rect, selected: bool) {
    Checkbox::show_state_at_with_selection_id(
        ui,
        if selected {
            CheckState::Checked
        } else {
            CheckState::Unchecked
        },
        rect,
        row.response.clone(),
        row.animation_id.with("item_selection"),
    );
}

fn paint_single_check(ui: &Ui, row: &RowUi, text_left: f32, depth: usize, hierarchical: bool) {
    let center_x = if hierarchical && depth > 0 {
        text_left - ui.spacing().indent * 0.5
    } else {
        row.rect.right() - ui.spacing().button_padding.x - INDICATOR_SIZE * 0.5
    };
    let rect = Rect::from_center_size(pos2(center_x, row.rect.center().y), Vec2::splat(INDICATOR_SIZE));
    Check
        .to_image()
        .tint(ui.visuals().text_color())
        .fit_to_exact_size(rect.size())
        .paint_at(ui, rect);
}

fn paint_row_background(ui: &Ui, rect: Rect, response: &Response, highlighted: bool) {
    let fill = if response.is_pointer_button_down_on() {
        Some(ui.app_style().widgets.active.bg_fill)
    } else if response.hovered() || response.has_focus() || highlighted {
        Some(ui.app_style().widgets.hovered.bg_fill)
    } else {
        None
    };
    if let Some(fill) = fill {
        ui.painter()
            .rect_filled(rect, ui.visuals().widgets.hovered.corner_radius, fill);
    }
}

fn paint_text(ui: &Ui, row_rect: Rect, text_left: f32, galley: &Arc<egui::Galley>) {
    let text_pos = pos2(text_left, row_rect.center().y - galley.size().y * 0.5);
    ui.painter().galley(text_pos, galley.clone(), ui.visuals().text_color());
}
