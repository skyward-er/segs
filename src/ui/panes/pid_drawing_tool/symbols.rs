use egui::{ImageSource, Theme};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

#[derive(Clone, Serialize, Deserialize, PartialEq, EnumIter, Display, Debug)]
pub enum Symbol {
    ManualValve,
    CheckValve,
    // ReliefValve,
    MotorValve,
    // ThreeWayValve,
    // PressureRegulator,
    // BurstDisk,
    // QuickConnector,
    // PressureTransducer,
    // PressureGauge,
    // FlexibleConnection,
    // PressurizedVessel,
}

impl Symbol {
    pub fn get_image(&self, theme: Theme) -> ImageSource {
        match (&self, theme) {
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
            (Symbol::MotorValve, Theme::Light) => {
                egui::include_image!("../../../../icons/pid_symbols/light/motor_valve.svg")
            }
            (Symbol::MotorValve, Theme::Dark) => {
                egui::include_image!("../../../../icons/pid_symbols/light/motor_valve.svg")
            }
        }
    }

    /// Symbol size in grid coordinates
    pub fn size(&self) -> Vec2 {
        match self {
            Symbol::ManualValve => Vec2::new(10.0, 5.0),
            Symbol::CheckValve => Vec2::new(10.0, 5.0),
            Symbol::MotorValve => Vec2::new(10.0, 7.5),
        }
    }

    /// Anchor point position relative to top right corner in grid units
    pub fn anchor_points(&self) -> Vec<Vec2> {
        match self {
            Symbol::ManualValve => [Vec2::new(0.0, 2.5), Vec2::new(10.0, 2.5)].into(),
            Symbol::CheckValve => [Vec2::new(0.0, 2.5), Vec2::new(10.0, 2.5)].into(),
            Symbol::MotorValve => [Vec2::new(0.0, 5.0), Vec2::new(10.0, 5.0)].into(),
        }
    }
}
