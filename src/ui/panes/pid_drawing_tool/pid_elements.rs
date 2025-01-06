use egui::{ImageSource, Theme};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PidElement {
    pub pos: (i32, i32),
    pub size: i32,
    pub symbol: PidSymbol,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter, Display)]
pub enum PidSymbol {
    BallValve,
    CheckValve,
    PressurizedVessel,
}

impl PidElement {
    pub fn contains(&self, pos: (i32, i32)) -> bool {
        (pos.0 >= self.pos.0 && pos.0 < (self.pos.0 + self.size))
            && (pos.1 >= self.pos.1 && pos.1 < (self.pos.1 + self.size))
    }

    pub fn get_image(&self, theme: Theme) -> ImageSource {
        match (&self.symbol, theme) {
            (PidSymbol::BallValve, Theme::Light) => {
                egui::include_image!("../../../../icons/ball_valve_light.svg")
            }
            (PidSymbol::BallValve, Theme::Dark) => {
                egui::include_image!("../../../../icons/ball_valve_dark.svg")
            }
            (PidSymbol::CheckValve, Theme::Light) => {
                egui::include_image!("../../../../icons/check_valve_light.svg")
            }
            (PidSymbol::CheckValve, Theme::Dark) => {
                egui::include_image!("../../../../icons/check_valve_dark.svg")
            }
            (PidSymbol::PressurizedVessel, Theme::Light) => {
                egui::include_image!("../../../../icons/pressurized_vessel_light.svg")
            }
            (PidSymbol::PressurizedVessel, Theme::Dark) => {
                egui::include_image!("../../../../icons/pressurized_vessel_dark.svg")
            }
        }
    }
}
