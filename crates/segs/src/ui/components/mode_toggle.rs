use std::{ops::Range, sync::Arc};

use aho_corasick::AhoCorasick;
use egui::{
    Align2, Area, CursorIcon, Frame, Id, Margin, Rect, Response, ScrollArea, Sense, Stroke, StrokeKind, TextEdit,
    TextFormat, Ui, UiBuilder, Vec2, text::LayoutJob, vec2,
};
use segs_assets::icons;
use segs_memory::MemoryExt;
use segs_ui::{
    ResponseExt,
    style::{CtxStyleExt, UiStyleExt, presets},
    widgets::{
        Separator,
        atoms::{Atoms, AtomsUi},
    },
};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Configuration,
    Operator(String),
}

pub struct ModeToggle<'a> {
    mode: &'a mut Mode,
    width: f32,
    height: f32,
}

impl<'a> ModeToggle<'a> {
    pub fn new(mode: &'a mut Mode) -> Self {
        Self {
            mode,
            width: 100.,
            height: 22.,
        }
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl ModeToggle<'_> {
    /// Shows the mode toggle button, customizable based on the current mode
    fn show_mode(
        self,
        ui: &mut Ui,
        id: Id,
        non_hovered: impl FnOnce(&mut Ui, Rect, f32),
        hovered: impl FnOnce(&mut Ui, Rect, f32),
        clicked: impl FnOnce(&mut Ui, Id, &mut Mode),
    ) {
        let Self { mode, width, height } = self;
        let hover_t_id = id.with("hover_t");
        let tooltip_t_id = id.with("tooltip_t");

        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
        let response = response.on_hover_cursor(CursorIcon::PointingHand);

        let hover_t = ui.ctx().animate_bool_with_time(hover_t_id, response.hovered(), 0.2);
        let tooltip_t = ui
            .ctx()
            .animate_bool_with_time(tooltip_t_id, response.hovered_for(1.), 0.2);

        let style = &ui.app_style().mode_toggle;
        let border_color = style
            .border_color_inactive
            .lerp_to_gamma(style.border_color_active, hover_t);
        let bg_fill = style
            .bg_fill_color_inactive
            .lerp_to_gamma(style.bg_fill_color_active, hover_t);

        let border = Stroke::new(1., border_color);

        let painter = ui.painter();
        painter.rect(rect, 5, bg_fill, border, StrokeKind::Inside);

        let show_config_t = (((1. - tooltip_t) - 0.5) / 0.5).clamp(0., 1.);
        let show_move_to_op_hint_t = ((tooltip_t - 0.5) / 0.5).clamp(0., 1.);

        let rect = rect.shrink2(vec2(0., 2.));
        if show_config_t > 0.0 {
            ui.scope(|ui| {
                ui.set_opacity(show_config_t);
                non_hovered(ui, rect, hover_t);
            });
        }
        if show_move_to_op_hint_t > 0.0 {
            ui.scope(|ui| {
                ui.set_opacity(show_move_to_op_hint_t);
                hovered(ui, rect, hover_t);
            });
        }

        // Handle click to switch to choose layout mode (internal mode)
        if response.clicked() {
            clicked(ui, id, mode);
            ui.ctx().request_repaint(); // Request repaint to update the UI immediately
        }
    }

    /// Shows the layout choice popup, allowing the user to choose a layout to switch to operator mode
    fn show_layout_choice_area(self, ui: &mut Ui, id: Id) {
        let Self { mode, width, height } = self;
        let area_id = id.with("layout_choice_popup");

        // Small "hack" to remove the "out of focus" animation of the configuration button
        let hover_t_id = id.with("hover_t");
        let _ = ui.ctx().animate_bool_with_time(hover_t_id, false, 0.);

        let pos = ui.cursor().center_top() + vec2(0., 1.);
        let response = Area::new(area_id)
            .pivot(Align2::CENTER_TOP)
            .fixed_pos(pos)
            .show(ui.ctx(), |ui| {
                // ui.set_opacity(source_toggled_t);
                let style = ui.visuals();
                Frame::new()
                    .corner_radius(style.menu_corner_radius)
                    .shadow(style.popup_shadow)
                    .fill(style.window_fill())
                    .stroke(style.window_stroke())
                    .show(ui, |ui| {
                        ui.with_style_override(presets::popup_style, |ui| {
                            ui.set_width(width);
                            ui.set_min_height(height);
                            ui.spacing_mut().item_spacing = Vec2::ZERO;
                            show_layout_filter(ui, id, mode, width);
                        });
                    })
                    .response
            })
            .inner;

        if response.clicked_elsewhere() {
            // If the popup was closed, switch back to configuration mode
            move_to_state(ui, id, ToggleState::Configuration);
        }
    }

    pub fn show(self, ui: &mut Ui) {
        let id = Id::new("mode_toggle");
        let id_status = id.with("status");

        let mut mode_selected: ToggleState = ui.mem().get_temp_or_default(id_status);
        mode_selected.sync(self.mode, ui, id);
        ui.mem().insert_temp(id_status, mode_selected.clone());

        match mode_selected {
            ToggleState::Configuration => {
                ui.scope_builder(UiBuilder::new().id_salt("configuration"), |ui| {
                    self.show_mode(ui, id, show_configuration_mode, show_move_to_op_hint, |ui, id, _| {
                        move_to_state(ui, id, ToggleState::ChooseLayout)
                    })
                });
            }
            ToggleState::ChooseLayout => self.show_layout_choice_area(ui, id),
            ToggleState::Operator(layout_name) => {
                ui.scope_builder(UiBuilder::new().id_salt("operator"), |ui| {
                    self.show_mode(
                        ui,
                        id,
                        |ui, rect, hover_t| {
                            show_operator_mode(ui, layout_name, rect, hover_t);
                        },
                        show_move_to_conf_hint,
                        |_, _, mode| *mode = Mode::Configuration,
                    )
                });
            }
        }
    }
}

fn show_layout_filter(ui: &mut Ui, id: Id, mode: &mut Mode, width: f32) {
    let text_edit_id = id.with("layout_name_text_edit");

    // Show a text input to filter layouts by name
    let mut text: String = ui.mem().get_temp_or_default(text_edit_id);
    let mut margin = Margin::same(5);
    margin.bottom = 5;
    let response = TextEdit::singleline(&mut text)
        .frame(false)
        .margin(margin)
        .hint_text("Search layout names...")
        .desired_width(width)
        .show(ui)
        .response;
    ui.mem().insert_temp(text_edit_id, text.clone());

    // Lock on focus on the input
    response.request_focus();

    // Add a divider between input field and layout list
    Separator::default().spacing(0.).ui(ui);

    let mut layouts = Vec::new();
    for i in 0..30 {
        layouts.push(format!("Layout {}", i + 1));
    }

    Frame::new().inner_margin(Margin::symmetric(0, 3)).show(ui, |ui| {
        ScrollArea::vertical()
            .max_height(300.)
            .min_scrolled_height(300.)
            .show(ui, |ui| {
                Frame::new().inner_margin(Margin::symmetric(4, 0)).show(ui, |ui| {
                    show_layout_selection_list(ui, id, mode, &layouts, text, width);
                });
            });
    });
}

fn show_layout_selection_list(ui: &mut Ui, id: Id, mode: &mut Mode, layouts: &[String], filter: String, width: f32) {
    let finder_id = id.with("text_finder");
    let matches_list_id = id.with("matches_list");
    let finder = ui
        .mem()
        .get_temp_or_insert_with(finder_id, || Arc::new(TextFinder::new(filter.clone())));

    // Get the list of layouts that match the filter, using cached results if the filter hasn't changed since the last time
    let matches = if finder.has_same_pattern(&filter)
        && let Some(matches) = ui.mem().get_temp::<Arc<Vec<(String, TextMatch)>>>(matches_list_id)
    {
        matches
    } else {
        let new_finder = Arc::new(TextFinder::new(filter));
        let matches = Arc::new(
            new_finder
                .iter_find_in(layouts.iter())
                .map(|(l, m)| (l.to_string(), m))
                .collect::<Vec<_>>(),
        );
        ui.mem().insert_temp(finder_id, new_finder);
        ui.mem().insert_temp(matches_list_id, matches.clone());
        matches
    };

    let height = 20.;
    if matches.is_empty() {
        // Show a "no results" text if there are no matching layouts
        let (_, rect) = ui.allocate_space(vec2(width - 8., height));
        let text_color = ui.visuals().text_color().gamma_multiply(0.5);
        let painter = ui.painter();
        let galley = painter.layout_no_wrap(
            String::from("No matching layouts found..."),
            ui.app_style().italic_font_of(13.),
            text_color,
        );
        painter.galley(rect.shrink(2.).min, galley, text_color);
    } else {
        // Show the list of matching layouts, sorted by relevance
        for (layout, matches) in matches.iter() {
            layout_entry_ui(ui, mode, layout, matches, height, width - 8.);
        }
    }
}

/// Shows a single entry in the layout choice popup, highlighting the parts of the layout name that match the search query
fn layout_entry_ui(
    ui: &mut Ui,
    mode: &mut Mode,
    layout: &str,
    matches: &TextMatch,
    height: f32,
    width: f32,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);

    if ui.is_rect_visible(rect) {
        let style = &ui.app_style();

        if response.hovered() {
            let shadow_color = style.shadow_fill;
            ui.painter().rect_filled(rect, 3, shadow_color);
        }

        let text_color = ui.visuals().text_color();
        let job = format_text_search(ui, matches, layout);
        let galley = ui.fonts_mut(|v| v.layout_job(job));
        let text_center_pos = rect.left_center() + vec2(galley.size().x * 0.5 + 5., 0.);
        let text_rect = Rect::from_center_size(text_center_pos, galley.size());
        ui.painter().galley(text_rect.min, galley, text_color);

        if response.clicked() {
            // If a layout is clicked, switch to operator mode with the selected layout
            *mode = Mode::Operator(layout.to_string());
        }
    }

    response
}

/// Formats the layout name text, highlighting the parts that match the search query
fn format_text_search(ui: &Ui, matches: &TextMatch, text: &str) -> LayoutJob {
    let style = &ui.app_style();
    let font_size = 13.;
    let regular_font = style.base_font_of(font_size);
    let match_font = style.semibold_font_of(font_size);
    let regular_color = ui.visuals().text_color();
    let match_color = style.accent_fill;

    let mut job = LayoutJob::default();
    let mut last_index = 0;
    for range in matches.0.iter() {
        if range.start > last_index {
            job.append(
                &text[last_index..range.start],
                0.,
                TextFormat::simple(regular_font.clone(), regular_color),
            );
        }
        job.append(
            &text[range.clone()],
            0.,
            TextFormat::simple(match_font.clone(), match_color),
        );
        last_index = range.end;
    }
    if last_index < text.len() {
        job.append(&text[last_index..], 0., TextFormat::simple(regular_font, regular_color));
    }
    job
}

/// Shows a text indicating that the user is in configuration mode (used in configuration mode)
fn show_configuration_mode(ui: &mut Ui, rect: Rect, hover_t: f32) {
    if ui.is_visible() {
        let style = &ui.app_style().mode_toggle;
        let stroke_color = style
            .stroke_color_inactive
            .lerp_to_gamma(style.stroke_color_active, hover_t);

        let icon_size = rect.height() - 4.;
        let text_size = rect.height() - 5.;
        Atoms::left_to_right().justified().with_pad(2.).place(ui, rect, |ui| {
            ui.add(AtomsUi::icon(icons::Tools, icon_size).with_tint(stroke_color))
                .add(
                    AtomsUi::text("Configuration Mode")
                        .with_color(stroke_color)
                        .with_text_size(text_size),
                );
        });
    }
}

/// Shows a hint to move to operator mode (used in configuration mode)
fn show_move_to_op_hint(ui: &mut Ui, rect: Rect, hover_t: f32) {
    if ui.is_visible() {
        let style = &ui.app_style().mode_toggle;
        let stroke_color = style
            .stroke_color_inactive
            .lerp_to_gamma(style.stroke_color_active, hover_t);

        let icon_size = rect.height() - 4.;
        let text_size = rect.height() - 5.;
        Atoms::left_to_right().justified().with_pad(0.).place(ui, rect, |ui| {
            ui.add(AtomsUi::icon(icons::Tools, icon_size).with_tint(stroke_color))
                .add(AtomsUi::icon(icons::Arrow::narrow_right(), icon_size).with_tint(stroke_color))
                .add(AtomsUi::icon(icons::Gauge, icon_size).with_tint(stroke_color))
                .add_pad(4.)
                .add(
                    AtomsUi::text("Choose layout")
                        .with_color(stroke_color)
                        .with_text_size(text_size),
                );
        });
    }
}

/// Shows a text indicating that the user is in operator mode, with the name of the current layout (used in operator mode)
fn show_operator_mode(ui: &mut Ui, layout_name: String, rect: Rect, hover_t: f32) {
    if ui.is_visible() {
        let style = &ui.app_style().mode_toggle;
        let stroke_color = style
            .stroke_color_inactive
            .lerp_to_gamma(style.stroke_color_active, hover_t);

        let icon_size = rect.height() - 4.;
        let text_size = rect.height() - 5.;
        Atoms::left_to_right().justified().with_pad(2.).place(ui, rect, |ui| {
            ui.add(AtomsUi::icon(icons::Gauge, icon_size).with_tint(stroke_color))
                .add(
                    AtomsUi::text("Operator: ")
                        .with_color(stroke_color)
                        .with_text_size(text_size),
                )
                .add_pad(0.)
                .add(
                    AtomsUi::text(layout_name)
                        .with_color(style.stroke_color_active)
                        .with_text_size(text_size),
                );
        });
    }
}

/// Shows a hint to move back to configuration mode (used in operator mode)
fn show_move_to_conf_hint(ui: &mut Ui, rect: Rect, hover_t: f32) {
    if ui.is_visible() {
        let style = &ui.app_style().mode_toggle;
        let stroke_color = style
            .stroke_color_inactive
            .lerp_to_gamma(style.stroke_color_active, hover_t);

        let icon_size = rect.height() - 4.;
        let text_size = rect.height() - 5.;
        Atoms::left_to_right().justified().with_pad(0.).place(ui, rect, |ui| {
            ui.add(AtomsUi::icon(icons::Gauge, icon_size).with_tint(stroke_color))
                .add(AtomsUi::icon(icons::Arrow::narrow_right(), icon_size).with_tint(stroke_color))
                .add(AtomsUi::icon(icons::Tools, icon_size).with_tint(stroke_color))
                .add_pad(4.)
                .add(
                    AtomsUi::text("Move to Configuration")
                        .with_color(stroke_color)
                        .with_text_size(text_size),
                );
        });
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
enum ToggleState {
    #[default]
    Configuration,
    ChooseLayout,
    Operator(String),
}

fn move_to_state(ui: &mut Ui, id: Id, mode: ToggleState) {
    let status_id = id.with("status");
    let mut state: ToggleState = ui.mem().get_temp(status_id).unwrap_or_default();
    state.force_to(mode, ui, id);
    ui.mem().insert_temp(status_id, state);
}

impl ToggleState {
    fn on_exit(&mut self, ui: &mut Ui, id: Id) {
        match self {
            ToggleState::Configuration => {
                // Small "hack" to remove the "out of focus" animation of the configuration button
                let hover_t_id = id.with("hover_t");
                let tooltip_t_id = id.with("tooltip_t");
                let _ = ui.ctx().animate_bool_with_time(hover_t_id, false, 0.);
                let _ = ui.ctx().animate_bool_with_time(tooltip_t_id, false, 0.);
            }
            ToggleState::ChooseLayout => {
                let text_edit_id = id.with("layout_name_text_edit");
                ui.mem().remove_temp::<String>(text_edit_id);
            }
            ToggleState::Operator(_) => {
                // Small "hack" to remove the "out of focus" animation of the configuration button
                let hover_t_id = id.with("hover_t");
                let tooltip_t_id = id.with("tooltip_t");
                let _ = ui.ctx().animate_bool_with_time(hover_t_id, false, 0.);
                let _ = ui.ctx().animate_bool_with_time(tooltip_t_id, false, 0.);
            }
        }
    }

    fn force_to(&mut self, mode: ToggleState, ui: &mut Ui, id: Id) {
        self.on_exit(ui, id);
        *self = mode;
    }

    fn sync(&mut self, mode: &Mode, ui: &mut Ui, id: Id) {
        match (&self, mode) {
            (ToggleState::Configuration | ToggleState::ChooseLayout, Mode::Configuration) => {}
            (ToggleState::Operator(layout), Mode::Operator(mode_layout)) if layout == mode_layout => {}
            _ => {
                self.force_to(mode.clone().into(), ui, id);
            }
        }
    }
}

impl From<Mode> for ToggleState {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Configuration => ToggleState::Configuration,
            Mode::Operator(layout) => ToggleState::Operator(layout),
        }
    }
}

impl From<ToggleState> for Mode {
    fn from(value: ToggleState) -> Self {
        match value {
            ToggleState::Configuration => Mode::Configuration,
            ToggleState::ChooseLayout => Mode::Configuration,
            ToggleState::Operator(layout) => Mode::Operator(layout),
        }
    }
}

#[derive(Debug, Clone)]
struct TextFinder {
    pattern: String,
    matcher: Option<AhoCorasick>,
}

impl TextFinder {
    fn new(pattern: impl AsRef<str>) -> Self {
        Self {
            pattern: pattern.as_ref().to_string().to_lowercase(),
            matcher: AhoCorasick::new([pattern.as_ref().to_lowercase()]).ok(),
        }
    }

    fn has_same_pattern(&self, pattern: impl AsRef<str>) -> bool {
        self.pattern == pattern.as_ref().to_lowercase()
    }

    fn iter_find_in<I, S>(&self, haystack: I) -> impl Iterator<Item = (S, TextMatch)>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let matcher = &self.matcher;
        let mut matches = haystack
            .into_iter()
            .filter_map(move |text| {
                let matcher = matcher.as_ref()?;
                let ranges: SmallVec<[Range<usize>; 3]> = matcher
                    .find_iter(&text.as_ref().to_lowercase())
                    .map(|m| m.range())
                    .collect();
                if ranges.is_empty() {
                    None
                } else {
                    Some((text, TextMatch(ranges)))
                }
            })
            .collect::<Vec<_>>();
        matches.sort_by(|(_, a), (_, b)| a.cmp(b));
        matches.into_iter()
    }
}

#[derive(Debug, Clone)]
struct TextMatch(SmallVec<[Range<usize>; 3]>);

impl PartialEq for TextMatch {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for TextMatch {}

impl Ord for TextMatch {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            match a.start.cmp(&b.start) {
                std::cmp::Ordering::Equal => continue,
                ord => return ord,
            }
        }
        self.0.len().cmp(&other.0.len())
    }
}

impl PartialOrd for TextMatch {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
