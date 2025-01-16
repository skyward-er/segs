mod motor_valve;

use egui::{ImageSource, Theme};
use glam::Vec2;
use motor_valve::MotorValve;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

#[derive(Clone, Serialize, Deserialize, PartialEq, EnumIter, Display, Debug)]
pub enum Symbol {
    Arrow,
    BurstDisk,
    CheckValve,
    FlexibleConnection,
    ManualValve,
    MotorValve(MotorValve),
    PressureGauge,
    PressureRegulator,
    PressureTransducer,
    QuickConnector,
    ReliefValve,
    ThreeWayValve,
    Vessel,
}

impl Symbol {
    pub fn get_image(&self, theme: Theme) -> ImageSource {
        match (&self, theme) {
            (Symbol::Arrow, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/arrow.svg")
            }
            (Symbol::Arrow, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/dark/arrow.svg")
            }
            (Symbol::BurstDisk, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/burst_disk.svg")
            }
            (Symbol::BurstDisk, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/dark/burst_disk.svg")
            }
            (Symbol::ManualValve, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/manual_valve.svg")
            }
            (Symbol::ManualValve, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/dark/manual_valve.svg")
            }
            (Symbol::CheckValve, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/check_valve.svg")
            }
            (Symbol::CheckValve, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/dark/check_valve.svg")
            }
            (Symbol::ReliefValve, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/relief_valve.svg")
            }
            (Symbol::ReliefValve, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/dark/relief_valve.svg")
            }
            (Symbol::MotorValve(state), Theme::Light) => match state.last_value {
                None => egui::include_image!("../../../../icons/pid_symbols/light/motor_valve.svg"),
                Some(true) => {
                    egui::include_image!(
                        "../../../../icons/pid_symbols/light/motor_valve_green.svg"
                    )
                }
                Some(false) => {
                    egui::include_image!("../../../../icons/pid_symbols/light/motor_valve_red.svg")
                }
            },
            (Symbol::MotorValve(state), Theme::Dark) => match state.last_value {
                None => egui::include_image!("../../../../icons/pid_symbols/dark/motor_valve.svg"),
                Some(true) => {
                    egui::include_image!("../../../../icons/pid_symbols/dark/motor_valve_green.svg")
                }
                Some(false) => {
                    egui::include_image!("../../../../icons/pid_symbols/dark/motor_valve_red.svg")
                }
            },
            (Symbol::ThreeWayValve, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/three_way_valve.svg")
            }
            (Symbol::ThreeWayValve, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/dark/three_way_valve.svg")
            }
            (Symbol::PressureRegulator, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/pressure_regulator.svg")
            }
            (Symbol::PressureRegulator, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/dark/pressure_regulator.svg")
            }
            (Symbol::QuickConnector, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/quick_connector.svg")
            }
            (Symbol::QuickConnector, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/dark/quick_connector.svg")
            }
            (Symbol::PressureTransducer, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/pressure_transducer.svg")
            }
            (Symbol::PressureTransducer, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/dark/pressure_transducer.svg")
            }
            (Symbol::PressureGauge, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/pressure_gauge.svg")
            }
            (Symbol::PressureGauge, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/dark/pressure_gauge.svg")
            }
            (Symbol::FlexibleConnection, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/flexible_connection.svg")
            }
            (Symbol::FlexibleConnection, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/dark/flexible_connection.svg")
            }
            (Symbol::Vessel, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/vessel.svg")
            }
            (Symbol::Vessel, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/dark/vessel.svg")
            }
        }
    }

    /// Symbol size in grid coordinates
    pub fn size(&self) -> Vec2 {
        match self {
            Symbol::Arrow => (4.0, 4.0),
            Symbol::BurstDisk => (4.0, 6.0),
            Symbol::CheckValve => (10.0, 5.0),
            Symbol::FlexibleConnection => (10.0, 6.0),
            Symbol::ManualValve => (10.0, 5.0),
            Symbol::MotorValve(_) => (10.0, 8.0),
            Symbol::PressureGauge => (7.0, 7.0),
            Symbol::PressureRegulator => (10.0, 10.0),
            Symbol::PressureTransducer => (7.0, 7.0),
            Symbol::QuickConnector => (6.0, 5.0),
            Symbol::ReliefValve => (6.0, 10.0),
            Symbol::ThreeWayValve => (10.0, 8.0),
            Symbol::Vessel => (8.2, 15.2),
        }
        .into()
    }

    /// Anchor point position relative to top right corner in grid units
    pub fn anchor_points(&self) -> Vec<Vec2> {
        match self {
            Symbol::Arrow => vec![(0.0, 2.0), (4.0, 2.0)],
            Symbol::BurstDisk => vec![(0.0, 3.0), (4.0, 3.0)],
            Symbol::CheckValve => vec![(0.0, 2.5), (10.0, 2.5)],
            Symbol::FlexibleConnection => vec![(0.0, 3.0), (10.0, 3.0)],
            Symbol::ManualValve => vec![(0.0, 2.5), (10.0, 2.5)],
            Symbol::MotorValve(_) => vec![(0.0, 5.0), (10.0, 5.0)],
            Symbol::PressureGauge => vec![(3.5, 7.0)],
            Symbol::PressureRegulator => vec![(0.0, 7.0), (10.0, 7.0)],
            Symbol::PressureTransducer => vec![(3.5, 7.0)],
            Symbol::QuickConnector => vec![(0.0, 2.5), (6.0, 2.5)],
            Symbol::ReliefValve => vec![(3.0, 10.0)],
            Symbol::ThreeWayValve => vec![(0.0, 3.0), (10.0, 3.0), (5.0, 8.0)],
            Symbol::Vessel => vec![(0.0, 7.6), (8.2, 7.6), (4.1, 0.0), (4.1, 15.1)],
        }
        .iter()
        .map(|&p| p.into())
        .collect()
    }
}

/// Single MavLink value source info
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(from = "SerialMavlinkValue")]
struct MavlinkValue {
    msg_id: u32,
    field: String,

    #[serde(skip)]
    view_id: egui::Id,
}

#[derive(Deserialize)]
struct SerialMavlinkValue {
    msg_id: u32,
    field: String,
}

impl From<SerialMavlinkValue> for MavlinkValue {
    fn from(value: SerialMavlinkValue) -> Self {
        Self {
            msg_id: value.msg_id,
            field: value.field,
            view_id: egui::Id::new(""),
        }
    }
}
