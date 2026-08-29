use egui::{
    CursorIcon, Response, Sense, TextStyle, TextWrapMode, Ui, Widget, WidgetInfo, WidgetText, WidgetType, pos2, vec2,
};

use crate::style::CtxStyleExt;

const CORNER_RADIUS: f32 = 2.;

/// A selectable label with interaction feedback across the complete available row.
pub struct SelectableRow {
    selected: bool,
    text: WidgetText,
}

impl SelectableRow {
    pub fn new(selected: bool, text: impl Into<WidgetText>) -> Self {
        Self {
            selected,
            text: text.into(),
        }
    }
}

impl Widget for SelectableRow {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { selected, text } = self;
        let label = text.text().to_owned();
        let row_height = ui.spacing().interact_size.y;
        let (id, rect) = ui.allocate_space(vec2(ui.available_width(), row_height));
        let response = ui
            .interact(rect, id, Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        response.widget_info(|| WidgetInfo::selected(WidgetType::SelectableLabel, ui.is_enabled(), selected, &label));

        if ui.is_rect_visible(rect) {
            let app_style = ui.app_style();
            let fill = if response.is_pointer_button_down_on() {
                Some(app_style.widgets.active.bg_fill)
            } else if response.hovered() || response.has_focus() {
                Some(app_style.widgets.hovered.bg_fill)
            } else if selected {
                Some(app_style.accent_fill)
            } else {
                None
            };
            if let Some(fill) = fill {
                ui.painter().rect_filled(rect, CORNER_RADIUS, fill);
            }

            let horizontal_padding = ui.spacing().item_spacing.x;
            let text_width = (rect.width() - horizontal_padding * 2.).max(0.);
            let galley = text.into_galley(ui, Some(TextWrapMode::Truncate), text_width, TextStyle::Body);
            let text_pos = pos2(
                rect.left() + horizontal_padding,
                rect.center().y - galley.size().y * 0.5,
            );
            ui.painter().galley(text_pos, galley, ui.visuals().text_color());
        }

        response
    }
}
