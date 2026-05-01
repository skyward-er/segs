use egui::{
    Align, Align2, Button, FontId, Frame, Id, Layout, PointerButton, Rect, RichText, Sense, Ui, UiBuilder,
    ViewportCommand, vec2,
};
use segs_ui::style::CtxStyleExt;

/// Draw client-side decorations for the main window
pub fn window_frame(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    let panel_frame = Frame::new()
        .fill(ui.style().visuals.panel_fill)
        .corner_radius(10)
        ; // so the stroke is within the bounds

    panel_frame.show(ui, |ui| {
        let app_rect = ui.max_rect();

        ui.expand_to_include_rect(app_rect); // Expand frame to include it all

        let title_bar_height = 32.0;
        let title_bar_rect = {
            let mut rect = app_rect;
            rect.max.y = rect.min.y + title_bar_height;
            rect
        };
        title_bar_ui(ui, title_bar_rect, title);

        // Add the contents:
        let content_rect = {
            let mut rect = app_rect;
            rect.min.y = title_bar_rect.max.y;
            rect
        }
        .shrink(4.0);
        let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect));
        add_contents(&mut content_ui);
    });
}

fn title_bar_ui(ui: &mut Ui, title_bar_rect: Rect, title: &str) {
    let painter = ui.painter();

    let title_bar_response = ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click_and_drag());

    // Paint the title:
    painter.text(
        title_bar_rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(20.0),
        ui.style().visuals.text_color(),
    );

    // Paint the line under the title:
    painter.line_segment(
        [
            title_bar_rect.left_bottom() + vec2(1.0, 0.0),
            title_bar_rect.right_bottom() + vec2(-1.0, 0.0),
        ],
        ui.app_style().main_view_stroke,
    );

    // Interact with the title bar (drag to move window):
    if title_bar_response.double_clicked() {
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        ui.send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
    }

    if title_bar_response.drag_started_by(PointerButton::Primary) {
        ui.send_viewport_cmd(ViewportCommand::StartDrag);
    }

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(title_bar_rect)
            .layout(Layout::right_to_left(Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);
            close_maximize_minimize(ui);
        },
    );
}

/// Show some close/maximize/minimize buttons for the native window.
fn close_maximize_minimize(ui: &mut Ui) {
    let button_height = 12.0;

    let close_response = ui
        .add(Button::new(RichText::new("X").size(button_height)))
        .on_hover_text("Close the window");
    if close_response.clicked() {
        ui.send_viewport_cmd(ViewportCommand::Close);
    }

    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    if is_maximized {
        let maximized_response = ui
            .add(Button::new(RichText::new("🗗").size(button_height)))
            .on_hover_text("Restore window");
        if maximized_response.clicked() {
            ui.send_viewport_cmd(ViewportCommand::Maximized(false));
        }
    } else {
        let maximized_response = ui
            .add(Button::new(RichText::new("🗗").size(button_height)))
            .on_hover_text("Maximize window");
        if maximized_response.clicked() {
            ui.send_viewport_cmd(ViewportCommand::Maximized(true));
        }
    }

    let minimized_response = ui
        .add(Button::new(RichText::new("🗕").size(button_height)))
        .on_hover_text("Minimize the window");
    if minimized_response.clicked() {
        ui.send_viewport_cmd(ViewportCommand::Minimized(true));
    }
}
