use super::Modal;

use std::{fmt::Display, net::Ipv4Addr, str::FromStr};

use egui::{Frame, Id, Label, Response, RichText, Ui, Vec2, vec2};
use segs_memory::MemoryExt;
use segs_ui::{
    style::presets,
    widgets::{Separator, labels::VerticalSelectableLabel, text::ValueEdit},
};

use crate::ui::components::value_edits;

pub const ADAPTER_CONFIG_MODAL_ID: &str = "adapter_config_modal";

pub struct AdapterConfigModal<'a> {
    source_toggled: &'a mut bool,
}

impl<'a> AdapterConfigModal<'a> {
    pub fn new(source_toggled: &'a mut bool) -> Self {
        Self { source_toggled }
    }

    pub fn show(self, ui: &mut Ui) {
        Modal::new(self.source_toggled)
            .id(Id::new(ADAPTER_CONFIG_MODAL_ID))
            .show(ui, |ui| {
                connection_ui(ui);
            });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceSelection {
    Ethernet,
    Serial,
}

fn connection_ui(ui: &mut Ui) {
    let id = ui.id().with("_source_selector");
    let mut selector = ui.mem().get_temp_or_default(id);

    ui.spacing_mut().item_spacing = Vec2::ZERO;

    // Connection Interface configuration
    match selector {
        Some(SourceSelection::Ethernet) => {
            Frame::new().inner_margin(2.5).show(ui, |ui| {
                ethernet_conn_ui(ui);
            });
            Separator::default().spacing(0.).ui_with_style(ui, presets::popup_style);
        }
        Some(SourceSelection::Serial) => {
            Frame::new().inner_margin(2.5).show(ui, |ui| {
                serial_conn_ui(ui);
            });
            Separator::default().spacing(0.).ui_with_style(ui, presets::popup_style);
        }
        None => (),
    }

    // Source selection
    Frame::new().inner_margin(vec2(2., 5.)).show(ui, |ui| {
        ui.spacing_mut().item_spacing = Vec2::ZERO;

        let widget = VerticalSelectableLabel::new(&mut selector)
            .add_variant(Some(SourceSelection::Ethernet), "ethernet")
            .add_variant(Some(SourceSelection::Serial), "serial")
            .add_variant(None, "none");

        if ui.add(widget).clicked() {
            ui.mem().insert_temp(Id::new("area_resize"), true);
        }
        ui.mem().insert_temp(id, selector);
    });
}

fn ethernet_conn_ui(ui: &mut Ui) -> Response {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(5., 2.);
            let id = ui.id().with("listen_ip");
            let mut listen_ip = ui.mem().get_temp_or_insert(id, Ipv4Addr::new(0, 0, 0, 0));
            labelled_value_edit(ui, "LISTEN IP", value_edits::ip_value_edit(&mut listen_ip));
            ui.mem().insert_temp(id, listen_ip);

            let id = ui.id().with("listen_port");
            let mut listen_port = ui.mem().get_temp_or_insert(id, 42069);
            labelled_value_edit(ui, "LISTEN PORT", value_edits::port_value_edit(&mut listen_port));
            ui.mem().insert_temp(id, listen_port);
        });

        ui.add_space(8.);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(5., 2.);
            let id = ui.id().with("send_ip");
            let mut send_ip = ui.mem().get_temp_or_insert(id, Ipv4Addr::new(169, 254, 255, 255));
            labelled_value_edit(ui, "SEND IP", value_edits::ip_value_edit(&mut send_ip));
            ui.mem().insert_temp(id, send_ip);

            let id = ui.id().with("send_port");
            let mut send_port = ui.mem().get_temp_or_insert(id, 42070);
            labelled_value_edit(ui, "SEND PORT", value_edits::port_value_edit(&mut send_port));
            ui.mem().insert_temp(id, send_port);
        });
    })
    .response
}

fn serial_conn_ui(ui: &mut Ui) -> Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(5., 2.);

        let id = ui.id().with("tty");
        let mut tty = ui.mem().get_temp_or_insert(id, "/dev/ttyUSB0".into());
        labelled_value_edit(ui, "TTY", value_edits::tty_value_edit(&mut tty));
        ui.mem().insert_temp(id, tty);

        let id = ui.id().with("baud_rate");
        let mut baud_rate = ui.mem().get_temp_or_insert(id, 115200);
        labelled_value_edit(ui, "BAUD RATE", value_edits::baudrate_value_edit(&mut baud_rate));
        ui.mem().insert_temp(id, baud_rate);
    })
    .response
}

fn labelled_value_edit<V: FromStr + Display>(
    ui: &mut Ui,
    label: impl Into<String>,
    value_edit: ValueEdit<'_, V>,
) -> Response {
    ui.vertical(|ui| {
        let label = label.into();
        let response = value_edit.id(ui.id().with(&label)).show(ui);
        Frame::new().inner_margin(vec2(5., 0.)).show(ui, |ui| {
            ui.add(Label::new(RichText::new(label).size(8.)).selectable(false))
        });
        response
    })
    .inner
}
