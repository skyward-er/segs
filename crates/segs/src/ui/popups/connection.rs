use std::{fmt::Display, net::Ipv4Addr, str::FromStr};

use egui::{Align, Align2, Frame, Id, Label, Pos2, Response, RichText, Ui, Vec2, vec2};
use segs_memory::MemoryExt;
use segs_ui::widgets::{labels::VerticalSelectableLabel, text::ValueEdit};

use crate::ui::popups;

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
    interface_conn_ui(ui, &selector);

    // Separator
    ui.add(egui::Separator::default().spacing(0.));

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

fn interface_conn_ui(ui: &mut Ui, selector: &SourceSelection) {
    Frame::new().inner_margin(2.5).show(ui, |ui| match selector {
        SourceSelection::Ethernet => {
            ethernet_conn_ui(ui);
        }
        SourceSelection::Automatic => {
            ui.label("text");
        } // TODO
        SourceSelection::Serial => {
            ui.label("text");
        } // TODO
    });
}

fn ethernet_conn_ui(ui: &mut Ui) -> Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(3., 2.);

        let id = ui.id().with("ip_address");
        let mut addr: Ipv4Addr = ui.ctx().mem().get_temp_or_insert(id, Ipv4Addr::new(127, 0, 0, 1));
        let value_edit = ValueEdit::new(&mut addr)
            .hint_text("IP Address...")
            .with_width(100.)
            .char_limit(15);
        labelled_value_edit(ui, "IP ADDRESS", value_edit);
        ui.ctx().mem().insert_temp(id, addr);

        let id = ui.id().with("recv_port");
        let mut recv_port = ui.ctx().mem().get_temp_or_insert(id, 42069);
        let value_edit = ValueEdit::new(&mut recv_port)
            .hint_text("Port...")
            .horizontal_align(Align::Center)
            .with_width(50.)
            .char_limit(5);
        labelled_value_edit(ui, "RECV PORT", value_edit);
        ui.ctx().mem().insert_temp(id, recv_port);

        let id = ui.id().with("send_port");
        let mut send_port = ui.ctx().mem().get_temp_or_insert(id, 42070);
        let value_edit = ValueEdit::new(&mut send_port)
            .hint_text("Port...")
            .horizontal_align(Align::Center)
            .with_width(50.)
            .char_limit(5);
        labelled_value_edit(ui, "SEND PORT", value_edit);
        ui.ctx().mem().insert_temp(id, send_port);
    })
    .response
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
