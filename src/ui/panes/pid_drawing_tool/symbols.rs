use egui::{ImageSource, Theme};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

#[derive(Clone, Serialize, Deserialize, PartialEq, EnumIter, Display)]
pub enum Symbol {
    ManualValve,
    CheckValve,
    // ReliefValve,
    // ControlValve,
    // PressureRegulator,
    // BurstDisk,
    // QuickConnector,
    // PressureTransducer,
    // PressureGauge,
    // FlexibleConnection,
    // ThreeWayValve,
    PressurizedVessel,
}

impl Symbol {
    pub fn get_image(&self, theme: Theme) -> ImageSource {
        match (&self, theme) {
            (Symbol::ManualValve, Theme::Light) => {
                egui::include_image!("../../../../icons/ball_valve_light.svg")
            }
            (Symbol::ManualValve, Theme::Dark) => {
                egui::include_image!("../../../../icons/ball_valve_dark.svg")
            }
            (Symbol::CheckValve, Theme::Light) => {
                egui::include_image!("../../../../icons/check_valve_light.svg")
            }
            (Symbol::CheckValve, Theme::Dark) => {
                egui::include_image!("../../../../icons/check_valve_dark.svg")
            }
            (Symbol::PressurizedVessel, Theme::Light) => {
                egui::include_image!("../../../../icons/pressurized_vessel_light.svg")
            }
            (Symbol::PressurizedVessel, Theme::Dark) => {
                egui::include_image!("../../../../icons/pressurized_vessel_dark.svg")
            }
        }
    }

    pub fn get_anchor_points(&self) -> Vec<(f32, f32)> {
        match self {
            Symbol::ManualValve => [(-0.5, 0.0), (0.5, 0.0)].into(),
            Symbol::CheckValve => [(-0.5, 0.0), (0.5, 0.0)].into(),
            Symbol::PressurizedVessel => [(0.0, -0.5), (0.0, 0.5)].into(),
        }
    }
}
