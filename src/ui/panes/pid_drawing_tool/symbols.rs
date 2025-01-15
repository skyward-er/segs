use egui::{ImageSource, Theme};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

#[derive(Clone, Serialize, Deserialize, PartialEq, EnumIter, Display, Debug)]
pub enum Symbol {
    Arrow,
    BurstDisk,
    CheckValve,
    FlexibleConnection,
    ManualValve,
    MotorValve,
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
                egui::include_image!("../../../../icons/pid_symbols/light/arrow.svg")
            }
            (Symbol::BurstDisk, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/burst_disk.svg")
            }
            (Symbol::BurstDisk, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/light/burst_disk.svg")
            }
            (Symbol::ManualValve, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/manual_valve.svg")
            }
            (Symbol::ManualValve, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/light/manual_valve.svg")
            }
            (Symbol::CheckValve, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/check_valve.svg")
            }
            (Symbol::CheckValve, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/light/check_valve.svg")
            }
            (Symbol::ReliefValve, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/relief_valve.svg")
            }
            (Symbol::ReliefValve, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/light/relief_valve.svg")
            }
            (Symbol::MotorValve, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/motor_valve.svg")
            }
            (Symbol::MotorValve, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/light/motor_valve.svg")
            }
            (Symbol::ThreeWayValve, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/three_way_valve.svg")
            }
            (Symbol::ThreeWayValve, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/light/three_way_valve.svg")
            }
            (Symbol::PressureRegulator, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/pressure_regulator.svg")
            }
            (Symbol::PressureRegulator, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/light/pressure_regulator.svg")
            }
            (Symbol::QuickConnector, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/quick_connector.svg")
            }
            (Symbol::QuickConnector, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/light/quick_connector.svg")
            }
            (Symbol::PressureTransducer, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/pressure_transducer.svg")
            }
            (Symbol::PressureTransducer, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/light/pressure_transducer.svg")
            }
            (Symbol::PressureGauge, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/pressure_gauge.svg")
            }
            (Symbol::PressureGauge, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/light/pressure_gauge.svg")
            }
            (Symbol::FlexibleConnection, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/flexible_connection.svg")
            }
            (Symbol::FlexibleConnection, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/light/flexible_connection.svg")
            }
            (Symbol::Vessel, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/vessel.svg")
            }
            (Symbol::Vessel, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/light/vessel.svg")
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
            Symbol::MotorValve => (10.0, 8.0),
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
            Symbol::MotorValve => vec![(0.0, 5.0), (10.0, 5.0)],
            Symbol::PressureGauge => vec![(0.0, 3.5), (7.0, 3.5)],
            Symbol::PressureRegulator => vec![(0.0, 7.0), (10.0, 7.0)],
            Symbol::PressureTransducer => vec![(0.0, 3.5), (7.0, 3.5)],
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
