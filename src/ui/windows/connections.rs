use egui::{Align2, Button, ComboBox, Context, RichText, Vec2};
use egui_extras::{Size, StripBuilder};
use tracing::{error, warn};

use crate::{
    communication::{
        ConnectionError, EthernetConfiguration, SerialConfiguration,
        ethernet::DEFAULT_ETHERNET_BROADCAST_IP,
        serial::{
            DEFAULT_BAUD_RATE,
            cached::{cached_first_stm32_port, cached_list_all_usb_ports},
        },
    },
    error::ErrInstrument,
    mavlink::{DEFAULT_RCV_ETHERNET_PORT, DEFAULT_SEND_ETHERNET_PORT},
    message_broker::{ConnectionConfig, MessageBroker},
};

#[derive(Default)]
pub struct ConnectionsWindow {
    pub visible: bool,
    connection_kind: ConnectionKind,
    connection_config: ConnectionSetting,
}

impl ConnectionsWindow {
    #[profiling::function]
    pub fn show(&mut self, ui: &mut egui::Ui, message_broker: &mut MessageBroker) {
        let mut window_is_open = self.visible;
        let mut can_be_closed = false;
        egui::Window::new("Sources")
            .id(ui.id())
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .max_width(200.0)
            .collapsible(false)
            .resizable(false)
            .open(&mut window_is_open)
            .show(ui.ctx(), |ui| {
                self.ui(ui, &mut can_be_closed, message_broker);
            });
        self.visible = window_is_open && !can_be_closed;
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        can_be_closed: &mut bool,
        message_broker: &mut MessageBroker,
    ) {
        let ConnectionsWindow {
            connection_kind,
            connection_config,
            ..
        } = self;
        ui.label("Select Source:");
        ui.horizontal_top(|ui| {
            ui.radio_value(connection_kind, ConnectionKind::Ethernet, "Ethernet");
            ui.radio_value(connection_kind, ConnectionKind::Serial, "Serial");
        });

        ui.separator();

        match (connection_kind, &connection_config) {
            (ConnectionKind::Ethernet, ConnectionSetting::Ethernet(_)) => {}
            (ConnectionKind::Serial, ConnectionSetting::Serial(_)) => {}
            (ConnectionKind::Ethernet, _) => {
                *connection_config = ConnectionSetting::Ethernet(default_ethernet());
            }
            (ConnectionKind::Serial, _) => {
                *connection_config = ConnectionSetting::Serial(
                    default_serial(ui.ctx()).log_expect("USER ERROR: issues with serail ports"),
                );
            }
        }

        match connection_config {
            ConnectionSetting::Ethernet(EthernetConfiguration {
                ip_address,
                send_port,
                receive_port,
            }) => {
                ui.vertical(|ui| {
                    let mut ip_str = ui.ctx().memory(|m| {
                        m.data
                            .get_temp(ui.id().with("ip_str"))
                            .unwrap_or(ip_address.to_string())
                    });

                    // Validate the IP address format and update the IP address
                    let mut valid_parse = false;
                    if let Ok(parsed_ip) = ip_str.parse::<std::net::IpAddr>() {
                        *ip_address = parsed_ip;
                        valid_parse = true;
                    }

                    // Create a TextEdit for the IP address input
                    let mut textedit = egui::TextEdit::singleline(&mut ip_str)
                        .hint_text("e.g. 0.0.0.0")
                        .desired_width(100.0);
                    if !valid_parse {
                        textedit = textedit.text_color(ui.style().visuals.error_fg_color);
                    }

                    // Display the IP address input field
                    ui.horizontal(|ui| {
                        ui.label("IP Address:");
                        ui.add(textedit);
                    });
                    // Store the IP address in the UI context memory
                    ui.ctx().memory_mut(|m| {
                        m.data.insert_temp(ui.id().with("ip_str"), ip_str.clone());
                    });
                    // Display the send and receive ports
                    ui.horizontal(|ui| {
                        ui.label("Send Port:");
                        ui.add(egui::DragValue::new(send_port).range(0..=65535).speed(10));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Receive Port:");
                        ui.add(
                            egui::DragValue::new(receive_port)
                                .range(0..=65535)
                                .speed(10),
                        );
                    });
                });
            }
            ConnectionSetting::Serial(opt) => {
                egui::Grid::new("grid")
                    .num_columns(2)
                    .spacing([10.0, 5.0])
                    .show(ui, |ui| {
                        ui.label("Serial Port:");
                        match opt.as_mut() {
                            Some(SerialConfiguration {
                                port_name,
                                baud_rate,
                            }) => {
                                ComboBox::from_id_salt("serial_port")
                                    .selected_text(port_name.as_str())
                                    .show_ui(ui, |ui| {
                                        for available_port in
                                            cached_list_all_usb_ports(ui.ctx()).log_unwrap()
                                        {
                                            ui.selectable_value(
                                                port_name,
                                                available_port.port_name.clone(),
                                                available_port.port_name,
                                            );
                                        }
                                    });

                                ui.label("Baud Rate:");
                                ui.add(
                                    egui::DragValue::new(baud_rate)
                                        .range(110..=256000)
                                        .speed(100),
                                );
                                ui.end_row();
                            }
                            None => {
                                // in case of a serial connection missing
                                warn!("USER ERROR: No serial port found");
                                ui.label(RichText::new("No port found").underline().strong());
                                *opt = default_serial(ui.ctx())
                                    .log_expect("USER ERROR: issues with serial ports");
                            }
                        }

                        ui.end_row();
                    });
            }
        };

        ui.separator();

        ui.allocate_ui(Vec2::new(ui.available_width(), 20.0), |ui| {
            StripBuilder::new(ui)
                .sizes(Size::remainder(), 2) // top cell
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        let btn1 = Button::new("Connect");
                        ui.add_enabled_ui(
                            !message_broker.is_connected() & connection_config.is_valid(),
                            |ui| {
                                if ui.add_sized(ui.available_size(), btn1).clicked() {
                                    if let Err(e) =
                                        connection_config.open_connection(message_broker)
                                    {
                                        error!("Failed to open connection: {:?}", e); // TODO: handle user erros
                                    }
                                    *can_be_closed = true;
                                }
                            },
                        );
                    });
                    strip.cell(|ui| {
                        let btn2 = Button::new("Disconnect");
                        ui.add_enabled_ui(message_broker.is_connected(), |ui| {
                            if ui.add_sized(ui.available_size(), btn2).clicked() {
                                message_broker.close_connection();
                            }
                        });
                    });
                });
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ConnectionKind {
    #[default]
    Ethernet,
    Serial,
}

#[derive(Debug, Clone)]
pub enum ConnectionSetting {
    Ethernet(EthernetConfiguration),
    Serial(Option<SerialConfiguration>),
}

fn default_ethernet() -> EthernetConfiguration {
    EthernetConfiguration {
        ip_address: DEFAULT_ETHERNET_BROADCAST_IP,
        send_port: DEFAULT_SEND_ETHERNET_PORT,
        receive_port: DEFAULT_RCV_ETHERNET_PORT,
    }
}

fn default_serial(ctx: &Context) -> Result<Option<SerialConfiguration>, serialport::Error> {
    let port_name =
        cached_first_stm32_port(ctx)?
            .map(|port| port.port_name)
            .or(cached_list_all_usb_ports(ctx)
                .ok()
                .and_then(|ports| ports.first().map(|port| port.port_name.clone())));
    Ok(port_name.map(|port_name| SerialConfiguration {
        port_name,
        baud_rate: DEFAULT_BAUD_RATE,
    }))
}

impl ConnectionSetting {
    fn is_valid(&self) -> bool {
        match self {
            ConnectionSetting::Ethernet(_) => true,
            ConnectionSetting::Serial(Some(_)) => true,
            ConnectionSetting::Serial(None) => false,
        }
    }

    fn open_connection(&self, msg_broker: &mut MessageBroker) -> Result<(), ConnectionError> {
        match self {
            Self::Ethernet(config) => {
                msg_broker.open_connection(ConnectionConfig::Ethernet(config.clone()));
                Ok(())
            }
            Self::Serial(Some(config)) => {
                msg_broker.open_connection(ConnectionConfig::Serial(config.clone()));
                Ok(())
            }
            Self::Serial(None) => Err(ConnectionError::WrongConfiguration(
                "No serial port found".to_string(),
            )),
        }
    }
}

impl Default for ConnectionSetting {
    fn default() -> Self {
        ConnectionSetting::Ethernet(default_ethernet())
    }
}
