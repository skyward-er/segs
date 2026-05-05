use egui::{Align, CursorIcon, Frame, Id, Layout, Panel, Ui, Vec2};
use segs_assets::icons;
use segs_memory::MemoryExt;
use segs_ui::widgets::buttons::{StatusBarButton, UnpaddedStatusBarButton};

use crate::ui::modals;

/// Shows the status bar as a bottom panel of the application window, displaying information and controls relevant to
/// the current state of the application.
pub fn show_inside(ui: &mut Ui) {
    Panel::bottom("status_bar")
        .show_separator_line(false)
        .frame(Frame::new().fill(ui.style().visuals.panel_fill))
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6., 0.);
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| show_left_side(ui));
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| show_right_side(ui));
            });
        });
}

fn show_left_side(ui: &mut egui::Ui) {
    show_adapter_status(ui);
}

fn show_right_side(ui: &mut egui::Ui) {
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

fn show_adapter_status(ui: &mut Ui) {
    let adapter_id = Id::new(modals::ADAPTER_CONFIG_MODAL_ID);
    let mut modal_visible: bool = ui.mem().get_temp_or_default(adapter_id);

    let button = UnpaddedStatusBarButton::default()
        .add_icon(icons::Antenna::outline())
        .add_text("Disconnected")
        .padded();
    let button_response = ui.add(button).on_hover_cursor(CursorIcon::PointingHand);
    if button_response.clicked() {
        modal_visible = true;
    }

    if modal_visible {
        modals::AdapterConfigModal::new(&mut modal_visible).show(ui);
    }

    ui.mem().insert_temp(adapter_id, modal_visible);
}
