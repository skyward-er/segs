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

    // /// Anchor point position relative to top right corner in grid units
    // pub fn anchor_points(&self) -> Vec<Vec2> {
    //     match self {
    //         Symbol::Arrow => vec![(0.0, 2.0), (4.0, 2.0)],
    //         Symbol::BurstDisk => vec![(0.0, 3.0), (4.0, 3.0)],
    //         Symbol::CheckValve => vec![(0.0, 2.5), (10.0, 2.5)],
    //         Symbol::FlexibleConnection => vec![(0.0, 3.0), (10.0, 3.0)],
    //         Symbol::ManualValve => vec![(0.0, 2.5), (10.0, 2.5)],
    //         Symbol::MotorValve(_) => vec![(0.0, 5.0), (10.0, 5.0)],
    //         Symbol::PressureGauge => vec![(3.5, 7.0)],
    //         Symbol::PressureRegulator => vec![(0.0, 7.0), (10.0, 7.0)],
    //         Symbol::PressureTransducer => vec![(3.5, 7.0)],
    //         Symbol::QuickConnector => vec![(0.0, 2.5), (6.0, 2.5)],
    //         Symbol::ReliefValve => vec![(3.0, 10.0)],
    //         Symbol::ThreeWayValve => vec![(0.0, 3.0), (10.0, 3.0), (5.0, 8.0)],
    //         Symbol::Vessel => vec![(0.0, 7.6), (8.2, 7.6), (4.1, 0.0), (4.1, 15.1)],
    //     }
    //     .iter()
    //     .map(|&p| p.into())
    //     .collect()
    // }

    fn context_menu(&mut self, ui: &mut Ui) {}
}
