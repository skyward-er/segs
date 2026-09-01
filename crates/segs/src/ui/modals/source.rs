use std::{
    fmt::Display,
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use egui::{Button, Color32, Frame, Id, Label, Response, RichText, TextEdit, Ui, UiBuilder, Vec2, vec2};

use segs_memory::MemoryExt;
use segs_ui::{
    containers::{Modal, ModalResponse},
    widgets::text::ValueEdit,
};

use crate::{
    app::AppContext,
    dataflow::{
        adapter::{
            AdapterType::{self, SkywardMavlink},
            DataAdapter, DataAdapterInstance,
        },
        mapping::DataMapping,
        skyward_mavlink_adapter::SkywardMavlinkAdapter,
        transport::{DataTransport, TransportType},
    },
    ui::components::value_edits,
};

const MODAL_ID: &str = "source_modal";

pub struct SourceModal<'a> {
    appctx: &'a mut AppContext,
}

impl<'a> SourceModal<'a> {
    pub fn new(appctx: &'a mut AppContext) -> Self {
        Self { appctx }
    }

    pub fn show(self, ui: &mut Ui) -> ModalResponse<()> {
        let SourceModal { appctx } = self;

        let adapter_id = ui.id().with("_adapter_index");
        let mapping_id = ui.id().with("_mapping_id");
        let transport_id = ui.id().with("_transport_id");
        let error_id = ui.id().with("_error_id");

        let mut adapter_sel: AdapterType = ui.mem().get_perm_or_default(adapter_id);
        let mut mapping_sel: String = ui.mem().get_perm_or_default(mapping_id);
        let mut transport_sel = ui.mem().get_perm_or_default(transport_id);
        let mut connect_error = ui.mem().get_temp_or_default(error_id);

        let res = Modal::new(Id::new(MODAL_ID), "Source Settings").show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Adapter");
                    egui::ComboBox::from_id_salt(ui.id().with("_adapter_combobox"))
                        .selected_text(adapter_sel.to_string())
                        .show_ui(ui, |ui| {
                            let adapter = AdapterType::SkywardMavlink;
                            ui.selectable_value(&mut adapter_sel, adapter, adapter.to_string());
                        });
                });

                ui.add_space(4.);

                ui.horizontal(|ui| {
                    ui.label("Mapping");
                    ui.add(TextEdit::singleline(&mut mapping_sel).hint_text("Path to mapping file..."));
                });
                let mapping = if !mapping_sel.is_empty() {
                    Some(DataMapping::LocalFile(mapping_sel.clone().into()))
                } else {
                    None
                };

                ui.add_space(8.);

                ui.horizontal(|ui| {
                    ui.label("Transport");
                    egui::ComboBox::from_id_salt(ui.id().with("_transport_combobox"))
                        .selected_text(format!("{:?}", transport_sel))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut transport_sel, TransportType::Ethernet, "Ethernet");
                            ui.selectable_value(&mut transport_sel, TransportType::Serial, "Serial");
                        });
                });

                ui.add_space(4.);

                let transport = match transport_sel {
                    TransportType::Ethernet => {
                        ui.scope_builder(UiBuilder::new().id_salt("ethernet"), show_ethernet_fields)
                            .inner
                    }
                    TransportType::Serial => {
                        ui.scope_builder(UiBuilder::new().id_salt("serial"), show_serial_fields)
                            .inner
                    }
                };

                ui.add_space(8.);

                let connected = appctx.data_adapter.is_some();
                ui.horizontal(|ui| {
                    if ui.add(Button::new("Disconnect").frame(connected)).clicked() && connected {
                        appctx.data_adapter = None;
                    }

                    if ui.add(Button::new("Connect").frame(!connected)).clicked() && !connected {
                        match adapter_sel {
                            SkywardMavlink => match (transport, mapping) {
                                (Some(transport), Some(mapping)) => {
                                    match SkywardMavlinkAdapter::new(ui.ctx().clone(), transport, mapping) {
                                        Ok(adapter) => {
                                            appctx.data_adapter = Some(DataAdapterInstance::new(Box::new(adapter)));
                                            connect_error = false;
                                            ui.close();
                                        }
                                        Err(e) => {
                                            eprintln!("Connect error: {e}");
                                            connect_error = true;
                                        }
                                    }
                                }
                                _ => {
                                    eprintln!("Connect error: incomplete parameters");
                                    connect_error = true;
                                }
                            },
                        }
                    }

                    if connect_error {
                        ui.label(RichText::new("Failed to connect").color(Color32::RED));
                    }
                });
            });
        });

        ui.mem().insert_perm(adapter_id, adapter_sel);
        ui.mem().insert_perm(mapping_id, mapping_sel);
        ui.mem().insert_perm(transport_id, transport_sel);
        ui.mem().insert_temp(error_id, connect_error);

        res
    }
}

fn show_ethernet_fields(ui: &mut Ui) -> Option<DataTransport> {
    ui.horizontal(|ui| {
        let listen_ip_id = ui.id().with("listen_ip");
        let listen_port_id = ui.id().with("listen_port");
        let send_ip_id = ui.id().with("send_ip");
        let send_port_id = ui.id().with("send_port");

        let mut listen_ip = ui.mem().get_perm_or_insert(listen_ip_id, Ipv4Addr::new(0, 0, 0, 0));
        let mut listen_port = ui.mem().get_perm_or_insert(listen_port_id, 42069);
        let mut send_ip = ui
            .mem()
            .get_perm_or_insert(send_ip_id, Ipv4Addr::new(169, 254, 255, 255));
        let mut send_port = ui.mem().get_perm_or_insert(send_port_id, 42070);

        labelled_value_edit(ui, "LISTEN IP", value_edits::ip_value_edit(&mut listen_ip));
        labelled_value_edit(ui, "LISTEN PORT", value_edits::port_value_edit(&mut listen_port));
        labelled_value_edit(ui, "SEND IP", value_edits::ip_value_edit(&mut send_ip));
        labelled_value_edit(ui, "SEND PORT", value_edits::port_value_edit(&mut send_port));

        ui.mem().insert_perm(listen_ip_id, listen_ip);
        ui.mem().insert_perm(listen_port_id, listen_port);
        ui.mem().insert_perm(send_ip_id, send_ip);
        ui.mem().insert_perm(send_port_id, send_port);

        Some(DataTransport::Ethernet {
            recv_socket: SocketAddrV4::new(listen_ip, listen_port),
            send_socket: SocketAddrV4::new(send_ip, send_port),
        })
    })
    .inner
}

fn show_serial_fields(ui: &mut Ui) -> Option<DataTransport> {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(5., 2.);

        let tty_id = ui.id().with("tty");
        let baudrate_id = ui.id().with("baud_rate");

        let mut tty = ui.mem().get_perm_or_insert(tty_id, "/dev/ttyUSB0".into());
        let mut baud_rate = ui.mem().get_perm_or_insert(baudrate_id, 115200);

        labelled_value_edit(ui, "TTY", value_edits::tty_value_edit(&mut tty));
        labelled_value_edit(ui, "BAUD RATE", value_edits::baudrate_value_edit(&mut baud_rate));

        ui.mem().insert_perm(tty_id, tty.clone());
        ui.mem().insert_perm(baudrate_id, baud_rate);

        Some(DataTransport::Serial { tty, baud_rate })
    })
    .inner
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
