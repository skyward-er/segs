use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::ui::app::PaneResponse;

use super::PaneBehavior;

mod enums;

#[derive(Clone, PartialEq, Default, Serialize, Deserialize, Debug)]
pub struct ValveControlPane {}

impl PaneBehavior for ValveControlPane {
    fn ui(&mut self, ui: &mut Ui) -> PaneResponse {
        todo!()
    }
}
