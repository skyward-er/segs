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

use crate::mavlink::MavMessage;

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
    /// Resets the subscriptions settings.
    /// IMPORTANT: This method should be called every time the msg_id changes.
    fn reset_subscriptions(&mut self);

    /// Updates the symbol based on the received message.
    fn update(&mut self, message: &MavMessage, subscribed_msg_id: u32);

    /// Renders the symbol on the UI.
    fn paint(&mut self, ui: &mut Ui, theme: Theme, pos: Vec2, size: f32, rotation: f32);

    /// Renders further elements related to the subscriptions settings
    fn subscriptions_ui(&mut self, ui: &mut Ui, mavlink_id: u32);

    /// Anchor point in grid coordinates relative to the element's center
    ///
    /// These vectors include the current rotation of the element.
    /// They are cached to avoid recomputing the rotation.
    fn anchor_points(&self) -> Option<Vec<Vec2>>;

    /// Symbol size in grid coordinates
    fn size(&self) -> Vec2;

    #[allow(unused_variables)]
    fn context_menu(&mut self, ui: &mut Ui) {}
}
