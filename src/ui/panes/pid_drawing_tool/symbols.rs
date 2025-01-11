use egui::{ImageSource, Theme};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

#[derive(Clone, Serialize, Deserialize, PartialEq, EnumIter, Display)]
pub enum Symbol {
    BallValve,
    CheckValve,
    PressurizedVessel,
}

impl Symbol {
    pub fn get_image(&self, theme: Theme) -> ImageSource {
        match (&self, theme) {
            (Symbol::BallValve, Theme::Light) => {
                egui::include_image!("../../../../icons/ball_valve_light.svg")
            }
            (Symbol::BallValve, Theme::Dark) => {
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
}
