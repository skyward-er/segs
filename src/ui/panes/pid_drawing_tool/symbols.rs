pub mod icons;
mod label;
mod value_display;

use egui::{Theme, Ui};
use enum_dispatch::enum_dispatch;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

use icons::Icon;
use label::Label;
use value_display::ValueDisplay;

use crate::ccsds::TelemetryPacket;

#[derive(Clone, Serialize, Deserialize, PartialEq, EnumIter, Display, Debug)]
#[enum_dispatch]
pub enum Symbol {
    Icon(Icon),
    ValueDisplay(ValueDisplay),
    Label(Label),
}

impl Default for Symbol {
    fn default() -> Self {
        Symbol::Icon(Icon::default())
    }
}

#[enum_dispatch(Symbol)]
pub trait SymbolBehavior {
    fn reset_subscriptions(&mut self);
    fn update(&mut self, packet: &TelemetryPacket);
    fn paint(&mut self, ui: &mut Ui, theme: Theme, pos: Vec2, size: f32, rotation: f32);
    fn subscriptions_ui(&mut self, ui: &mut Ui);
    fn anchor_points(&self) -> Option<Vec<Vec2>>;
    fn size(&self) -> Vec2;

    #[allow(unused_variables)]
    fn context_menu(&mut self, ui: &mut Ui) {}
}
