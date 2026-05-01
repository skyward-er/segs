use egui::{Align, CursorIcon, Frame, Layout, Panel, Ui, Vec2};
use segs_assets::icons;
use segs_memory::MemoryExt;
use segs_ui::widgets::buttons::{StatusBarButton, UnpaddedStatusBarButton};

use crate::{App, ui::popups};

/// Shows the status bar as a bottom panel of the application window, displaying information and controls relevant to
/// the current state of the application.
pub fn show_inside(ui: &mut Ui, app: &App) {
    Panel::bottom("status_bar")
        .show_separator_line(false)
        .frame(Frame::new().fill(ui.style().visuals.panel_fill))
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6., 0.);
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| show_left_side(app, ui));
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| show_right_side(app, ui));
            });
        });
}

fn show_left_side(_app: &App, ui: &mut egui::Ui) {
    let source_id = ui.id().with("status_bar_source");
    let mut source_selection: bool = ui.mem().get_temp_or_default(source_id);

    let icon = if source_selection {
        icons::Antenna::solid()
    } else {
        icons::Antenna::outline()
    };
    let btn = UnpaddedStatusBarButton::default()
        .add_icon(icon)
        .add_text("Sources")
        .padded();
    let res = ui.add(btn).on_hover_cursor(CursorIcon::PointingHand);
    if res.clicked() {
        source_selection = !source_selection;
    }
    ui.mem().insert_temp(source_id, source_selection);

    if source_selection {
        popups::ConnectionPopup::new(&mut source_selection, res.rect.left_top(), egui::Align2::LEFT_BOTTOM).show(ui);
    }
}

fn show_right_side(_app: &App, ui: &mut egui::Ui) {
    let notifications_id = ui.id().with("status_bar_notifications");
    let mut notifications_visible: bool = ui.mem().get_temp_or_default(notifications_id);

    let bell_icon = if notifications_visible {
        icons::Bell::solid()
    } else {
        icons::Bell::outline()
    };
    let btn = UnpaddedStatusBarButton::default().add_icon(bell_icon).add_space(4.);
    let res = ui.add(btn);
    if res.on_hover_cursor(CursorIcon::PointingHand).clicked() {
        notifications_visible = !notifications_visible;
    }
    ui.mem().insert_temp(notifications_id, notifications_visible);

    let btn = UnpaddedStatusBarButton::default()
        .padded()
        .add_icon(icons::Lightning)
        .add_text("Quick Commands");
    ui.add(btn);
}
