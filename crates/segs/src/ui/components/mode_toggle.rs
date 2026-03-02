use egui::{
    Align2, Color32, CornerRadius, CursorIcon, Frame, Id, Margin, Rect, Response, Sense, Stroke, Ui, UiBuilder, Vec2,
    vec2,
};
use segs_assets::icons::{self, Icon};
use segs_memory::MemoryExt;
use segs_ui::{style::CtxStyleExt, utils::RadioGroup};
use serde::{Deserialize, Serialize};

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
            height: 18.,
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
    fn frame_content(self, ui: &mut Ui, hover_t: f32) {
        let Self { mode, width, height } = self;

        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::empty());

        let id = Id::new("mode_toggle");
        let id_status = id.with("status");
        let id_config_sel = id.with("config_selected");

        let mode_selected: ToggleState = ui.ctx().mem().get_temp_or_default(id_status);
        // let config_sel_t = ui
        //     .ctx()
        //     .animate_bool(id_config_sel, mode_selected == ToggleState::Configuration);

        let show_config_t = (((1. - hover_t) - 0.5) / 0.5).clamp(0., 1.);
        let show_move_to_op_hint_t = ((hover_t - 0.5) / 0.5).clamp(0., 1.);

        if show_config_t > 0.0 {
            ui.scope(|ui| {
                ui.set_opacity(show_config_t);
                show_configuration_mode(ui, rect, hover_t);
            });
        }
        if show_move_to_op_hint_t > 0.0 {
            ui.scope(|ui| {
                ui.set_opacity(show_move_to_op_hint_t);
                show_move_to_op_hint(ui, rect, hover_t);
            });
        }
    }

    pub fn show(self, ui: &mut Ui) {
        let id = Id::new("mode_toggle");
        let hover_id = id.with("hover");
        let hover_t_id = id.with("hover_t");

        let is_hovered: bool = ui.mem().get_temp_or_default(hover_id);
        let hover_t = ui.ctx().animate_bool_with_time(hover_t_id, is_hovered, 0.2);

        let style = &ui.app_style().mode_toggle;
        let border_color = style
            .border_color_inactive
            .lerp_to_gamma(style.border_color_active, hover_t);
        let bg_fill = style
            .bg_fill_color_inactive
            .lerp_to_gamma(style.bg_fill_color_active, hover_t);

        let border = Stroke::new(1., border_color);

        let response = Frame::new()
            .stroke(border)
            .fill(bg_fill)
            .inner_margin(Margin::symmetric(0, 0))
            .outer_margin(Margin::symmetric(0, 2))
            .corner_radius(5)
            .show(ui, |ui| {
                self.frame_content(ui, hover_t);
            })
            .response
            .interact(Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);

        // Update hover state in memory for next frame styling
        ui.mem().insert_temp(hover_id, response.hovered());
    }
}

fn show_configuration_mode(ui: &mut Ui, rect: Rect, hover_t: f32) {
    if ui.is_visible() {
        let style = &ui.app_style().mode_toggle;
        let stroke_color = style
            .stroke_color_inactive
            .lerp_to_gamma(style.stroke_color_active, hover_t);

        let painter = ui.painter();

        // ---- Layout the text
        let galley = painter.layout_no_wrap(
            String::from("Configuration Mode"),
            ui.app_style().base_font_of(rect.height() - 5.),
            stroke_color,
        );

        let icon_size = Vec2::splat(rect.height() - 4.);
        let text_size = galley.size();
        let icon_text_pad = 2.;
        let content_width = icon_size.x + icon_text_pad + text_size.x;
        let offset = (rect.width() - content_width) / 2.;

        let mut cursor = rect.left_center();
        cursor.x += offset;

        // ---- Paint the icon
        cursor.x += icon_size.x / 2.;
        let icon_rect = Rect::from_center_size(cursor, icon_size);
        icons::Tools
            .to_image()
            .tint(stroke_color)
            .fit_to_exact_size(icon_size)
            .paint_at(ui, icon_rect);
        cursor.x = icon_rect.right();

        // ---- Paint the text
        cursor.x += icon_text_pad + text_size.x / 2.;
        let text_rect = Rect::from_center_size(cursor, text_size);
        painter.galley(text_rect.min, galley, stroke_color);
    }
}

fn show_move_to_op_hint(ui: &mut Ui, rect: Rect, hover_t: f32) {
    if ui.is_visible() {
        let style = &ui.app_style().mode_toggle;
        let stroke_color = style
            .stroke_color_inactive
            .lerp_to_gamma(style.stroke_color_active, hover_t);

        let painter = ui.painter();

        // ---- Layout the text
        let galley = painter.layout_no_wrap(
            String::from("Move to Operator"),
            ui.app_style().base_font_of(rect.height() - 5.),
            stroke_color,
        );

        let icon_size = Vec2::splat(rect.height() - 4.);
        let text_size = galley.size();
        let text_icon_pad = 4.;
        let content_width = text_size.x + text_icon_pad + icon_size.x * 3.;
        let offset = (rect.width() - content_width) / 2.;

        let mut cursor = rect.left_center();
        cursor.x += offset;

        // ---- Paint the icons
        cursor.x += icon_size.x / 2.;
        let icons = [
            icons::Tools.to_image(),
            icons::Arrow::narrow_right().to_image(),
            icons::Gauge.to_image(),
        ];
        for image in icons {
            let icon_rect = Rect::from_center_size(cursor, icon_size);
            image
                .tint(stroke_color)
                .fit_to_exact_size(icon_size)
                .paint_at(ui, icon_rect);
            cursor.x += icon_size.x;
        }
        cursor.x += -icon_size.x / 2. + text_icon_pad;

        // ---- Paint the text
        cursor.x += text_size.x / 2.;
        let text_rect = Rect::from_center_size(cursor, text_size);
        painter.galley(text_rect.min, galley, stroke_color);
    }
}

// fn show_configuration_mode(ui: &mut Ui, width: f32, height: f32, hover_t: f32) {
//     if ui.is_visible() {
//         let style = &ui.app_style().mode_toggle;
//         let stroke_color = style
//             .stroke_color_inactive
//             .lerp_to_gamma(style.stroke_color_active, hover_t);

//         // ---- Layout the text
//         let galley = ui.painter().layout_no_wrap(
//             String::from("Configuration Mode"),
//             ui.app_style().base_font_of(height - 5.),
//             stroke_color,
//         );

//         let icon_size = Vec2::splat(height - 4.);
//         let text_size = galley.size();
//         let icon_text_pad = 2.;
//         let content_width = icon_size.x + icon_text_pad + text_size.x;
//         let extra_space = width - content_width;
//         let offset = extra_space / 2.;

//         // ---- Left pad
//         ui.add_space(offset);

//         // ---- Paint the icon
//         let (rect, _) = ui.allocate_exact_size(icon_size, Sense::empty());
//         icons::Tools
//             .to_image()
//             .tint(stroke_color)
//             .fit_to_exact_size(rect.size())
//             .paint_at(ui, rect);

//         // ---- Padding space between icon and text
//         ui.add_space(icon_text_pad);

//         // ---- Paint the text
//         let (rect, _) = ui.allocate_exact_size(vec2(text_size.x, height), Sense::empty());
//         let rect = Rect::from_center_size(rect.center(), text_size);
//         ui.painter().galley(rect.min, galley, stroke_color);

//         // ---- Right pad
//         ui.add_space(offset);
//     }
// }

#[derive(Debug, Clone, Default, PartialEq)]
enum ToggleState {
    #[default]
    Configuration,
    ChooseLayout,
    Operator(String),
}

fn left_corner_radius(cr: u8) -> CornerRadius {
    CornerRadius {
        nw: cr,
        sw: cr,
        ..Default::default()
    }
}

fn right_corner_radius(cr: u8) -> CornerRadius {
    CornerRadius {
        ne: cr,
        se: cr,
        ..Default::default()
    }
}
