use std::{fmt::Display, net::Ipv4Addr, str::FromStr};

use egui::{Align2, Frame, Id, Label, Pos2, Response, RichText, Ui, Vec2, vec2};
use segs_memory::MemoryExt;
use segs_ui::widgets::{
    labels::VerticalSelectableLabel,
    text::{TextEdit, ValueEdit},
};

use crate::ui::{components::value_edits, popups};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum SourceSelection {
    Ethernet,
    Automatic,
    #[default]
    Serial,
}

pub struct ConnectionPopup<'a> {
    source_toggled: &'a mut bool,
    pivot_pos: Pos2,
    pivot_align: Align2,
}

impl<'a> ConnectionPopup<'a> {
    pub fn new(source_toggled: &'a mut bool, pivot_pos: Pos2, pivot_align: Align2) -> Self {
        Self {
            source_toggled,
            pivot_pos,
            pivot_align,
        }
    }

    pub fn show(self, ui: &mut Ui) {
        let Self {
            source_toggled,
            pivot_pos,
            pivot_align,
        } = self;

        let id = Id::new("connection_popup");
        let mut popup = popups::Popup::new(source_toggled, pivot_pos).id(id).pivot(pivot_align);
        if ui.ctx().mem().remove_temp(Id::new("area_resize")).is_some_and(|b| b) {
            popup = popup.forze_sizing_pass()
        }
        popup.show(ui, connection_ui);
    }
}

fn connection_ui(ui: &mut Ui) {
    let id = ui.id().with("_source_selector");
    let mut selector = ui.ctx().mem().get_temp_or_default(id);

    ui.spacing_mut().item_spacing = Vec2::ZERO;

    // Connection Interface configuration
    match selector {
        SourceSelection::Ethernet => {
            Frame::new().inner_margin(2.5).show(ui, |ui| {
                ethernet_conn_ui(ui);
            });
            ui.add(egui::Separator::default().spacing(0.));
        }
        SourceSelection::Serial => {
            Frame::new().inner_margin(2.5).show(ui, |ui| {
                serial_conn_ui(ui);
            });
            ui.add(egui::Separator::default().spacing(0.));
        }
        SourceSelection::Automatic => (),
    }

    // Source selection
    Frame::new().inner_margin(vec2(5., 5.)).show(ui, |ui| {
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        let widget = VerticalSelectableLabel::new(&mut selector)
            .add_variant(SourceSelection::Ethernet, "ethernet")
            .add_variant(SourceSelection::Automatic, "automatic")
            .add_variant(SourceSelection::Serial, "serial");
        if ui.add(widget).clicked() {
            ui.ctx().mem().insert_temp(Id::new("area_resize"), true);
        }
        ui.ctx().mem().insert_temp(id, selector);
    });
}

fn ethernet_conn_ui(ui: &mut Ui) -> Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(3., 2.);

        let id = ui.id().with("ip_address");
        let mut addr: Ipv4Addr = ui.ctx().mem().get_temp_or_insert(id, Ipv4Addr::new(127, 0, 0, 1));
        labelled_value_edit(ui, "IP ADDRESS", value_edits::ip_value_edit(&mut addr));
        ui.ctx().mem().insert_temp(id, addr);

        let id = ui.id().with("recv_port");
        let mut recv_port = ui.ctx().mem().get_temp_or_insert(id, 42069);
        labelled_value_edit(ui, "RECV PORT", value_edits::port_value_edit(&mut recv_port));
        ui.ctx().mem().insert_temp(id, recv_port);

        let id = ui.id().with("send_port");
        let mut send_port = ui.ctx().mem().get_temp_or_insert(id, 42070);
        labelled_value_edit(ui, "SEND PORT", value_edits::port_value_edit(&mut send_port));
        ui.ctx().mem().insert_temp(id, send_port);
    })
    .response
}

fn serial_conn_ui(ui: &mut Ui) {
    // ui.menu_button(atoms, add_contents)

    let id = Id::new("serial_port_menu");
    let enable_id = id.with("_enable");
    let visible_id = id.with("_visible");

    let button = egui::Button::new("click");
    let response = ui.add(button);
    if response.clicked() {
        ui.ctx().mem().insert_temp(enable_id, true);
    }

    let enabled = ui.ctx().mem().get_temp_or_default(enable_id);
    let visible_t =
        ui.ctx()
            .animate_bool_with_time_and_easing(visible_id, enabled, 0.2, egui::emath::easing::cubic_out);

    let pivot = response.rect.center_top() + vec2(0., -2.);
    if visible_t > 0.3 {
        let id = ui.id().with("_area");

        let source_toggled_t = (visible_t - 0.2) / 0.8;
        let style = ui.style();
        let res = egui::Area::new(id)
            .pivot(Align2::CENTER_BOTTOM)
            .fixed_pos(pivot)
            .order(egui::Order::Foreground)
            // .sizing_pass(force_sizing_pass)
            .show(ui.ctx(), |ui| {
                ui.set_opacity(source_toggled_t);
                Frame::new()
                    .corner_radius(style.visuals.menu_corner_radius)
                    .shadow(style.visuals.popup_shadow)
                    .fill(style.visuals.window_fill())
                    .stroke(style.visuals.window_stroke())
                    .show(ui, |ui| {
                        asdasd(ui);
                    });
            })
            .response;

        /* // After a sizing pass, request a discard to avoid showing a frame without the open popup contents
        if force_sizing_pass {
            ui.ctx().request_discard("record popup size after forced sizing pass");
        } */

        // Hide the popup if the user clicks outside of it
        if res.clicked_elsewhere() {
            ui.ctx().mem().insert_temp(enable_id, false);
        }
    }
}

fn asdasd(ui: &mut Ui) {
    Frame::new().inner_margin(2.5).show(ui, |ui| {
        let id_text_edit = ui.id().with("_text_edit");
        let mut text = ui.ctx().mem().get_temp_or_default(id_text_edit);
        TextEdit::new(&mut text).hint_text("Select Variant...").show(ui);
        ui.ctx().mem().insert_temp(id_text_edit, text);
    });

    ui.add(egui::Separator::default().spacing(0.));

    Frame::new().inner_margin(2.5).show(ui, |ui| {
        ui.label("asdasd");
    });
}

fn labelled_value_edit<V: FromStr + Display>(
    ui: &mut Ui,
    label: impl Into<String>,
    value_edit: ValueEdit<'_, V>,
) -> Response {
    ui.vertical(|ui| {
        let response = value_edit.show(ui);
        Frame::new().inner_margin(vec2(5., 0.)).show(ui, |ui| {
            ui.add(Label::new(RichText::new(label).size(8.)).selectable(false))
        });
        response
    })
    .inner
}
