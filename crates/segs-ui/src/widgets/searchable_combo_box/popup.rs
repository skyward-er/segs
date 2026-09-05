use std::{collections::HashSet, f32::consts::PI, hash::Hash, sync::Arc};

use egui::{
    FocusDirection, Frame, Galley, Id, Key, Margin, Modifiers, Popup, PopupCloseBehavior, Rect, Response, Sense,
    StrokeKind, TextStyle, TextWrapMode, Ui, Vec2, WidgetInfo, WidgetText, WidgetType, pos2, vec2,
};
use segs_assets::icons::{CaretDown, Icon};

use crate::widgets::{Separator, buttons::Checkbox, text::TextEdit};

use super::{
    INDICATOR_ANIMATION_DURATION_FACTOR, INDICATOR_RIGHT_PADDING, INDICATOR_SIZE, INDICATOR_TEXT_SPACING,
    SEARCH_VERTICAL_MARGIN, SearchableComboBox,
    choices::ChoiceSource,
    rows::{RowAction, Rows, RowsLayout, RowsNavigation, RowsState, resolve_visible_rows},
    selection::SelectionState,
};

#[derive(Clone, Debug, Default)]
struct ComboBoxState {
    query: String,
    visible_rows: Vec<usize>,
    rows_valid: bool,
    highlighted_row: Option<usize>,
    list_scroll_offset: f32,
    list_viewport_height: f32,
    expanded: HashSet<usize>,
}

impl ComboBoxState {
    fn reset_transient(&mut self) {
        self.query.clear();
        self.visible_rows.clear();
        self.rows_valid = false;
        self.highlighted_row = None;
        self.list_scroll_offset = 0.;
        self.list_viewport_height = 0.;
    }
}

#[derive(Clone, Debug)]
struct ComboBoxWidthCache {
    style_revision: Id,
    multiple: bool,
    hierarchical: bool,
    content_width: f32,
}

struct ComboBoxTrigger {
    response: Response,
    galley: Arc<Galley>,
    text: String,
}

/// Renders a searchable combo box for one supported choice and selection strategy.
pub(super) fn show_combo_box<C, S>(ui: &mut Ui, combo: SearchableComboBox<'_, C, S>, hierarchical: bool) -> Response
where
    C: ChoiceSource,
    C::Value: Clone + Eq + Hash + 'static,
    S: SelectionState<C::Value>,
{
    let SearchableComboBox {
        id,
        choices,
        mut selection,
        empty_selection_text,
        max_visible_rows,
        search_hint,
        empty_results_text,
        singular_selection_noun,
        plural_selection_noun,
    } = combo;

    // Resolve trigger text entirely from the reusable choices and selection
    let selected_text = if S::MULTIPLE {
        match selection.selected_count() {
            0 => None,
            1 => Some(format!("1 {singular_selection_noun} selected")),
            count => Some(format!("{count} {plural_selection_noun} selected")),
        }
    } else {
        selection
            .single_value()
            .and_then(|value| choices.selected_text(value))
            .map(str::to_owned)
    };
    let empty_selection = selected_text.is_none();
    let trigger_text = selected_text.map_or(empty_selection_text, WidgetText::from);
    let trigger = allocate_trigger(ui, trigger_text);
    let popup_id = id.with("popup");
    let state_id = popup_id.with("state");
    let was_open = Popup::is_id_open(ui.ctx(), popup_id);
    let opening = trigger.response.clicked() && !was_open;
    let displayed_open = if trigger.response.clicked() {
        !was_open
    } else {
        was_open
    };

    // Paint the trigger using the state that this frame's click will produce
    paint_trigger(ui, &trigger, empty_selection, displayed_open);
    let accessibility_label = format!(
        "{}, {}",
        trigger.text,
        if displayed_open { "expanded" } else { "collapsed" }
    );
    trigger.response.widget_info(|| {
        let mut info = WidgetInfo::labeled(WidgetType::ComboBox, ui.is_enabled(), &accessibility_label);
        info.current_text_value = Some(trigger.text.clone());
        info
    });

    // Restore state owned by this component and choice revision
    let mut state = ui.data_mut(|data| data.remove_temp::<ComboBoxState>(state_id).unwrap_or_default());
    if opening {
        state.reset_transient();
    }

    // Render searchable popup chrome and the internally virtualized rows
    let popup_width = trigger.response.rect.width().max(0.);
    let popup_frame = Frame::popup(ui.style()).inner_margin(Margin::ZERO);
    let popup = Popup::menu(&trigger.response)
        .id(popup_id)
        .width(popup_width)
        .frame(popup_frame)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            let row_spacing = ui.spacing().item_spacing.y;
            let row_text_inset = ui.spacing().button_padding.x.round().clamp(0., i8::MAX as f32) as i8;
            let list_margin = ui.spacing().menu_margin;
            let text_horizontal_margin = Margin {
                left: list_margin.left.saturating_add(row_text_inset),
                right: list_margin.right.saturating_add(row_text_inset),
                top: 0,
                bottom: 0,
            };

            // Expand from the trigger width using one cached full-list measurement
            apply_cached_content_width(ui, choices, popup_id, S::MULTIPLE, hierarchical);
            ui.spacing_mut().item_spacing.y = 0.;

            // Reserve navigation keys before the focused search editor handles them
            let navigation_input = read_navigation_input(ui, S::MULTIPLE);
            if navigation_input.move_up > 0 || navigation_input.move_down > 0 {
                ui.memory_mut(|memory| memory.move_focus(FocusDirection::None));
            }

            // Align the search field with unindented row labels
            let query_before = state.query.clone();
            let search_response = ui.add(
                TextEdit::singleline(&mut state.query)
                    .frameless()
                    .margin(Margin {
                        top: SEARCH_VERTICAL_MARGIN,
                        bottom: SEARCH_VERTICAL_MARGIN,
                        ..text_horizontal_margin
                    })
                    .id_salt(popup_id.with("search"))
                    .hint_text(search_hint)
                    .desired_width(ui.available_width()),
            );
            if !search_response.has_focus() {
                search_response.request_focus();
            }
            ui.add(Separator::default().spacing(1.));

            // Rebuild filtered or expanded row indices only when their inputs change
            let query_changed = state.query != query_before;
            if query_changed {
                state.rows_valid = false;
            }
            if !state.rows_valid {
                resolve_visible_rows(
                    choices,
                    hierarchical,
                    &state.query,
                    &state.expanded,
                    &mut state.visible_rows,
                );
                state.rows_valid = true;
            }
            let action = if navigation_input.toggle_selection {
                RowAction::ToggleSelection
            } else if navigation_input.activate {
                RowAction::Activate
            } else {
                RowAction::None
            };

            let changes = Rows {
                choices,
                selection: &mut selection,
                component_id: popup_id,
                hierarchical,
                visible: &state.visible_rows,
                filtered: !state.query.trim().is_empty(),
                empty_results_text: &empty_results_text,
                layout: RowsLayout {
                    max_visible_rows,
                    row_spacing,
                    list_margin,
                    text_horizontal_margin,
                },
                navigation: RowsNavigation {
                    reset_highlight: opening || query_changed,
                    move_delta: navigation_input.move_down as isize - navigation_input.move_up as isize,
                    action,
                },
                state: RowsState {
                    expanded: &mut state.expanded,
                    highlighted_row: &mut state.highlighted_row,
                    scroll_offset: &mut state.list_scroll_offset,
                    viewport_height: &mut state.list_viewport_height,
                },
            }
            .show(ui);
            if changes.expansion {
                state.rows_valid = false;
            }
            if changes.selection_event && !S::MULTIPLE {
                ui.memory_mut(|memory| {
                    memory.surrender_focus(search_response.id);
                    memory.move_focus(FocusDirection::None);
                });
                ui.close();
            }
            changes.selection
        });

    // Keep hierarchy expansion but clear transient popup state after closing
    let changed = popup.is_some_and(|response| response.inner);
    if !Popup::is_id_open(ui.ctx(), popup_id) {
        state.reset_transient();
    }
    ui.data_mut(|data| data.insert_temp(state_id, state));

    let mut response = trigger.response;
    if changed {
        response.mark_changed();
    }
    response
}

struct NavigationInput {
    move_up: usize,
    move_down: usize,
    activate: bool,
    toggle_selection: bool,
}

fn read_navigation_input(ui: &mut Ui, multiple: bool) -> NavigationInput {
    ui.input_mut(|input| NavigationInput {
        toggle_selection: multiple && input.consume_key(Modifiers::COMMAND, Key::Space),
        move_up: input.count_and_consume_key(Modifiers::NONE, Key::ArrowUp),
        move_down: input.count_and_consume_key(Modifiers::NONE, Key::ArrowDown),
        activate: input.consume_key(Modifiers::NONE, Key::Enter),
    })
}

fn apply_cached_content_width<C>(ui: &mut Ui, choices: &C, popup_id: Id, multiple: bool, hierarchical: bool)
where
    C: ChoiceSource,
{
    let width_cache_id = popup_id.with("content_width");
    let style_revision = width_style_revision(ui);
    let cached_width = ui.data(|data| data.get_temp::<ComboBoxWidthCache>(width_cache_id));
    let content_width = match cached_width {
        Some(cache)
            if cache.style_revision == style_revision
                && cache.multiple == multiple
                && cache.hierarchical == hierarchical =>
        {
            cache.content_width
        }
        _ => measure_and_cache_content_width(ui, choices, width_cache_id, style_revision, multiple, hierarchical),
    };
    let list_margin = ui.spacing().menu_margin;
    let horizontal_margin = f32::from(list_margin.left) + f32::from(list_margin.right);
    ui.set_min_width(ui.available_width().max(content_width + horizontal_margin));
}

fn measure_and_cache_content_width<C>(
    ui: &mut Ui,
    choices: &C,
    cache_id: Id,
    style_revision: Id,
    multiple: bool,
    hierarchical: bool,
) -> f32
where
    C: ChoiceSource,
{
    let content_width = measure_content_width(ui, choices, multiple, hierarchical);
    ui.data_mut(|data| {
        data.insert_temp(
            cache_id,
            ComboBoxWidthCache {
                style_revision,
                multiple,
                hierarchical,
                content_width,
            },
        );
    });
    content_width
}

fn measure_content_width<C>(ui: &Ui, choices: &C, multiple: bool, hierarchical: bool) -> f32
where
    C: ChoiceSource,
{
    let padding = ui.spacing().button_padding.x;
    let item_spacing = ui.spacing().item_spacing.x;

    // Measure stable full-list geometry independently of filtering and expansion
    (0..choices.len())
        .map(|index| {
            let group = choices.group_end(index).is_some();
            let text = if group {
                WidgetText::from(egui::RichText::new(choices.label(index)).strong())
            } else {
                WidgetText::from(choices.label(index))
            };
            let text_width = text
                .into_galley(ui, Some(TextWrapMode::Extend), f32::INFINITY, TextStyle::Body)
                .size()
                .x;
            let indent = if hierarchical {
                ui.spacing().indent * choices.depth(index) as f32
            } else {
                0.
            };
            let leading = if multiple && hierarchical {
                Checkbox::SIZE.x + item_spacing
            } else {
                0.
            };
            let trailing = if group {
                item_spacing + INDICATOR_SIZE
            } else if !hierarchical {
                item_spacing + if multiple { Checkbox::SIZE.x } else { INDICATOR_SIZE }
            } else if !multiple && choices.depth(index) == 0 {
                item_spacing + INDICATOR_SIZE
            } else {
                0.
            };
            padding + indent + leading + text_width + trailing + padding
        })
        .fold(0., f32::max)
}

fn width_style_revision(ui: &Ui) -> Id {
    let spacing = ui.spacing();
    let body_font = TextStyle::Body.resolve(ui.style());
    Id::new((
        spacing.button_padding.x.to_bits(),
        spacing.item_spacing.x.to_bits(),
        spacing.indent.to_bits(),
        body_font.size.to_bits(),
        body_font.family,
        ui.ctx().pixels_per_point().to_bits(),
    ))
}

fn allocate_trigger(ui: &mut Ui, text: WidgetText) -> ComboBoxTrigger {
    // Preserve plain text before galley conversion consumes it
    let plain_text = text.text().to_owned();
    let width = ui.available_width().max(0.);
    let horizontal_padding = ui.spacing().button_padding.x;
    let text_width =
        (width - horizontal_padding - INDICATOR_SIZE - INDICATOR_RIGHT_PADDING - INDICATOR_TEXT_SPACING).max(0.);
    let galley = text.into_galley(ui, Some(TextWrapMode::Truncate), text_width, TextStyle::Button);

    // Match the standard one-line control height
    let height = ui
        .spacing()
        .interact_size
        .y
        .max(galley.size().y + ui.spacing().button_padding.y * 2.);
    let (id, rect) = ui.allocate_space(vec2(width, height));
    let response = ui.interact(rect, id, Sense::click());
    ComboBoxTrigger {
        response,
        galley,
        text: plain_text,
    }
}

fn paint_trigger(ui: &Ui, trigger: &ComboBoxTrigger, empty_selection: bool, expanded: bool) {
    let ComboBoxTrigger { response, galley, .. } = trigger;
    if !ui.is_rect_visible(response.rect) {
        return;
    }

    // Paint the persistent standard combo-box frame
    let interaction = if expanded {
        &ui.visuals().widgets.open
    } else {
        ui.style().interact(response)
    };
    ui.painter().rect(
        response.rect.expand(interaction.expansion),
        interaction.corner_radius,
        interaction.weak_bg_fill,
        interaction.bg_stroke,
        StrokeKind::Inside,
    );

    // Align the one-line summary and animated trigger indicator
    let text_color = interaction.text_color();
    let horizontal_padding = ui.spacing().button_padding.x;
    let text_pos = pos2(
        response.rect.left() + horizontal_padding,
        response.rect.center().y - galley.size().y * 0.5,
    );
    let indicator_rect = Rect::from_center_size(
        pos2(
            response.rect.right() - INDICATOR_RIGHT_PADDING - INDICATOR_SIZE * 0.5,
            response.rect.center().y,
        ),
        Vec2::splat(INDICATOR_SIZE),
    );
    let summary_color = if empty_selection {
        text_color.gamma_multiply(ui.visuals().weak_text_alpha)
    } else {
        text_color
    };
    ui.painter().galley(text_pos, galley.clone(), summary_color);
    let openness = ui.ctx().animate_bool_with_time_and_easing(
        response.id.with("indicator_openness"),
        expanded,
        ui.style().animation_time * INDICATOR_ANIMATION_DURATION_FACTOR,
        egui::emath::easing::cubic_out,
    );
    CaretDown::solid()
        .to_image()
        .tint(text_color)
        .rotate(PI * openness, Vec2::splat(0.5))
        .fit_to_exact_size(indicator_rect.size())
        .paint_at(ui, indicator_rect);
}
