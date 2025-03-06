use egui::{Align2, Button, Color32, ComboBox, RichText, Vec2};
use egui_extras::{Size, StripBuilder};
use tracing::{error, warn};

use crate::{
    communication::{
        ConnectionError, EthernetConfiguration, SerialConfiguration,
        serial::{DEFAULT_BAUD_RATE, find_first_stm32_port, list_all_usb_ports},
    },
    error::ErrInstrument,
    mavlink::DEFAULT_ETHERNET_PORT,
    message_broker::MessageBroker,
};

#[derive(Debug)]
enum ConnectionConfig {
    Ethernet(EthernetConfiguration),
    Serial(Option<SerialConfiguration>),
}

impl ConnectionConfig {
    fn default_ethernet() -> Self {
        Self::Ethernet(EthernetConfiguration {
            port: DEFAULT_ETHERNET_PORT,
        })
    }

    fn default_serial() -> Self {
        let port_name = find_first_stm32_port()
            .map(|port| port.port_name)
            .or(list_all_usb_ports()
                .ok()
                .and_then(|ports| ports.first().map(|port| port.port_name.clone())));
        let Some(port_name) = port_name else {
            warn!("USER ERROR: No serial port found");
            return Self::Serial(None);
        };
        Self::Serial(Some(SerialConfiguration {
            port_name,
            baud_rate: DEFAULT_BAUD_RATE,
        }))
    }

    fn is_valid(&self) -> bool {
        match self {
            Self::Ethernet(_) => true,
            Self::Serial(Some(_)) => true,
            Self::Serial(None) => false,
        }
    }

    fn open_connection(&self, msg_broker: &mut MessageBroker) -> Result<(), ConnectionError> {
        match self {
            Self::Ethernet(config) => msg_broker.open_connection(config.clone()),
            Self::Serial(Some(config)) => msg_broker.open_connection(config.clone()),
            Self::Serial(None) => Err(ConnectionError::WrongConfiguration(
                "No serial port found".to_string(),
            )),
        }
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self::Ethernet(EthernetConfiguration {
            port: DEFAULT_ETHERNET_PORT,
        })
    }
}

impl PartialEq for ConnectionConfig {
    fn eq(&self, other: &Self) -> bool {
        matches!(self, Self::Ethernet(_)) && matches!(other, Self::Ethernet(_))
            || matches!(self, Self::Serial(_)) && matches!(other, Self::Serial(_))
    }
}

#[derive(Debug, Default)]
pub struct ConnectionsWindow {
    pub visible: bool,
    connection_config: ConnectionConfig,
}

impl ConnectionsWindow {
    #[profiling::function]
    pub fn show_window(&mut self, ui: &mut egui::Ui, message_broker: &mut MessageBroker) {
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
            connection_config, ..
        } = self;
        ui.label("Select Source:");
        ui.horizontal_top(|ui| {
            ui.radio_value(
                connection_config,
                ConnectionConfig::default_ethernet(),
                "Ethernet",
            );
            ui.radio_value(
                connection_config,
                ConnectionConfig::default_serial(),
                "Serial",
            );
        });

        ui.separator();

        match connection_config {
            ConnectionConfig::Ethernet(EthernetConfiguration { port }) => {
                egui::Grid::new("grid")
                    .num_columns(2)
                    .spacing([10.0, 5.0])
                    .show(ui, |ui| {
                        ui.label("Ethernet Port:");
                        ui.add(egui::DragValue::new(port).range(0..=65535).speed(10));
                        ui.end_row();
                    });
            }
            ConnectionConfig::Serial(opt) => {
                egui::Grid::new("grid")
                    .num_columns(2)
                    .spacing([10.0, 5.0])
                    .show(ui, |ui| {
                        ui.label("Serial Port:");
                        match opt {
                            Some(SerialConfiguration {
                                port_name,
                                baud_rate,
                            }) => {
                                ComboBox::from_id_salt("serial_port")
                                    .selected_text(port_name.as_str())
                                    .show_ui(ui, |ui| {
                                        for available_port in list_all_usb_ports().log_unwrap() {
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
                                ui.label(
                                    RichText::new("No port found")
                                        .color(Color32::RED)
                                        .underline()
                                        .strong(),
                                );
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
