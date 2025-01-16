use crate::{msg_broker, ui::utils::glam_to_egui};

use super::grid::GridInfo;
use super::symbols::Symbol;
use crate::error::ErrInstrument;
use egui::{Rect, Theme, Ui};
use glam::{Mat2, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(from = "SerialElement")]
pub struct Element {
    /// Anchor postion in grid coordinates, top-left corner
    position: glam::Vec2,

    /// Symbol to be displayed
    symbol: Symbol,

    /// Rotation in radiants
    rotation: f32,

    /// Anchor point in grid coordinates relative to the element's center
    ///
    /// These vectors include the current rotation of the element.
    /// They are cached to avoid recomputing the rotation.
    #[serde(skip)]
    anchor_points: Vec<Vec2>,
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.symbol == other.symbol
            && self.rotation == other.rotation
    }
}

impl Element {
    pub fn new(center: Vec2, symbol: Symbol) -> Self {
        Self {
            position: center - symbol.size() / 2.0,
            rotation: 0.0,
            anchor_points: symbol.anchor_points(),
            symbol,
        }
    }

    /// Check if the given position is inside the element
    pub fn contains(&self, p_g: Vec2) -> bool {
        // First we need to do a rotostranslation from the grid's frame to the element's frame
        let rotm = Mat2::from_angle(-self.rotation);
        let p_e = rotm * (p_g - self.position);

        // The bounding box is just the size
        let min_e = Vec2::ZERO;
        let max_e = self.symbol.size();

        // Check if the point is in the bounding box
        min_e.x <= p_e.x && p_e.x <= max_e.x && min_e.y <= p_e.y && p_e.y <= max_e.y
    }

    /// Moves the element such that its center is at the given position
    pub fn set_center_at(&mut self, p_g: Vec2) {
        // Rotation matrix from element's frame to grid's frame
        let rotm_e_to_g = Mat2::from_angle(self.rotation);

        // Center in grid's frame
        let center_g = rotm_e_to_g * self.size() / 2.0;

        self.position = p_g - center_g;
    }

    pub fn change_symbol(&mut self, symbol: Symbol) {
        self.symbol = symbol;

        // Anchor points can be different between symbols, realod the cache
        self.reload_anchor_points();
    }

    /// Rotate the element by its center
    pub fn rotate(&mut self, rotation: f32) {
        // Current center position relative to the top-left point in the grid reference frame
        let center_g = Mat2::from_angle(self.rotation) * self.symbol.size() / 2.0;

        // Rotate the position by the element's center
        self.position += (Mat2::IDENTITY - Mat2::from_angle(rotation)) * center_g;

        // Update absolute rotation
        self.rotation += rotation;

        // Recompute anchor points cache
        self.reload_anchor_points();
    }

    fn reload_anchor_points(&mut self) {
        // Rotation matrix from element's frame to grid's frame
        let rotm_e_to_g = Mat2::from_angle(self.rotation);

        // Then rotate the anchor points
        self.anchor_points = self
            .symbol
            .anchor_points()
            .iter()
            .map(|&p| rotm_e_to_g * p)
            .collect();
    }

    /// Returns the position of one anchor point in grid coordinates
    pub fn anchor_point(&self, idx: usize) -> Vec2 {
        self.anchor_points[idx] + self.position
    }

    pub fn anchor_points_len(&self) -> usize {
        self.anchor_points.len()
    }

    /// Size in grid units
    pub fn size(&self) -> Vec2 {
        self.symbol.size()
    }

    /// Position of the element's center in grid frame
    pub fn center(&self) -> Vec2 {
        self.position + Mat2::from_angle(self.rotation) * self.size() * 0.5
    }

    pub fn draw(&mut self, grid: &GridInfo, ui: &Ui, theme: Theme) {
        let center = glam_to_egui(grid.grid_to_screen(self.position)).to_pos2();
        let image_rect = Rect::from_min_size(center, glam_to_egui(self.size() * grid.size()));

        egui::Image::new(self.symbol.get_image(theme))
            .rotate(self.rotation, egui::Vec2::splat(0.0))
            .paint_at(ui, image_rect);

        if let Symbol::MotorValve(motor_valve) = &mut self.symbol {
            msg_broker!().refresh_view(motor_valve).log_expect("bruh");
        }
    }
}

#[derive(Deserialize)]
pub struct SerialElement {
    position: glam::Vec2,
    symbol: Symbol,
    rotation: f32,
}

impl From<SerialElement> for Element {
    fn from(value: SerialElement) -> Self {
        let mut value = Self {
            position: value.position,
            symbol: value.symbol,
            rotation: value.rotation,
            anchor_points: Vec::new(),
        };
        value.reload_anchor_points();
        value
    }
}
