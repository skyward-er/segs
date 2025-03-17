use std::f32::consts::FRAC_PI_2;

use egui::{Theme, Ui};
use glam::{Mat2, Vec2};
use serde::{Deserialize, Serialize};

use crate::mavlink::MavMessage;

use super::{
    grid::GridInfo,
    symbols::{Symbol, SymbolBehavior},
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Element {
    /// Anchor postion in grid coordinates, top-left corner
    position: glam::Vec2,

    /// Symbol to be displayed
    symbol: Symbol,

    /// Rotation in radiants
    rotation: f32,
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

    pub fn context_menu(&mut self, ui: &mut Ui) {
        match &mut self.symbol {
            Symbol::Icon(_) => {
                if ui.button("Rotate 90° ⟲").clicked() {
                    self.rotate(-FRAC_PI_2);
                    ui.close_menu();
                }
                if ui.button("Rotate 90° ⟳").clicked() {
                    self.rotate(FRAC_PI_2);
                    ui.close_menu();
                }
            }
            Symbol::Label(label) => {
                label.context_menu(ui);
            }
        }
    }

    /// Rotate the element by its center
    pub fn rotate(&mut self, rotation: f32) {
        // Current center position relative to the top-left point in the grid reference frame
        let center_g = Mat2::from_angle(self.rotation) * self.symbol.size() / 2.0;

        // Rotate the position by the element's center
        self.position += (Mat2::IDENTITY - Mat2::from_angle(rotation)) * center_g;

        // Update absolute rotation
        self.rotation += rotation;
    }

    /// Returns the position of one anchor point in grid coordinates
    pub fn anchor_point(&self, idx: usize) -> Vec2 {
        if let Some(anchor_points) = self.symbol.anchor_points() {
            // Rotation matrix from element's frame to grid's frame
            let rotm_e_to_g = Mat2::from_angle(self.rotation);

            // Then rotate and translate the anchor points
            rotm_e_to_g * anchor_points[idx] + self.position
        } else {
            Vec2::ZERO
        }
    }

    pub fn anchor_points_len(&self) -> usize {
        self.symbol.anchor_points().map_or(0, |v| v.len())
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
        let pos = grid.grid_to_screen(self.position);
        let size = grid.size();
        self.symbol.paint(ui, theme, pos, size, self.rotation);
    }

    pub fn update(&mut self, message: &MavMessage) {
        self.symbol.update(message);
    }
}
