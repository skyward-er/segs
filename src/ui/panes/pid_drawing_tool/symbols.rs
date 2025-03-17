pub mod icons;
mod labels;

use egui::{Theme, Ui};
use enum_dispatch::enum_dispatch;
use glam::Vec2;
use icons::Icon;
use labels::Label;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

use crate::mavlink::MavMessage;

#[derive(Clone, Serialize, Deserialize, PartialEq, EnumIter, Display, Debug)]
#[enum_dispatch]
pub enum Symbol {
    Icon(Icon),
    Label(Label),
}

impl Default for Symbol {
    fn default() -> Self {
        Symbol::Icon(Icon::default())
    }
}

#[enum_dispatch(Symbol)]
pub trait SymbolBehavior {
    fn paint(&mut self, ui: &Ui, theme: Theme, pos: Vec2, size: f32, rotation: f32);

    /// Anchor point in grid coordinates relative to the element's center
    ///
    /// These vectors include the current rotation of the element.
    /// They are cached to avoid recomputing the rotation.
    fn anchor_points(&self) -> Option<Vec<Vec2>>;

    /// Symbol size in grid coordinates
    fn size(&self) -> Vec2;

    fn update(&mut self, message: &MavMessage);

    #[allow(unused_variables)]
    fn context_menu(&mut self, ui: &mut Ui) {}
}
