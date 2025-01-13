mod connections;
mod elements;
mod grid;
mod pos;
mod symbols;

use connections::Connection;
use egui::{
    epaint::PathStroke, Color32, Context, CursorIcon, PointerButton, Pos2, Rounding, Sense, Stroke,
    Theme, Ui, Vec2,
};
use elements::Element;
use grid::GridInfo;
use pos::Pos;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use strum::IntoEnumIterator;
use symbols::Symbol;

use crate::ui::composable_view::PaneResponse;

use super::PaneBehavior;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
enum Action {
    Connect(usize),
    ContextMenu(Pos2),
    DragElement(usize),
    DragConnection(usize, usize),
}

/// Piping and instrumentation diagram
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct PidPane {
    elements: Vec<Element>,
    connections: Vec<Connection>,

    grid: GridInfo,

    #[serde(skip)]
    action: Option<Action>,
}

impl Default for PidPane {
    fn default() -> Self {
        Self {
            elements: Vec::default(),
            connections: Vec::default(),
            grid: GridInfo { size: 10.0 },

            action: None,
        }
    }
}

impl PaneBehavior for PidPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        let theme = PidPane::find_theme(ui.ctx());
        self.draw_grid(theme, ui);
        self.draw_connections(ui);
        self.draw_elements(theme, ui);

        // Allocate the space to sense inputs
        let (_, response) = ui.allocate_at_least(ui.max_rect().size(), Sense::click_and_drag());
        let pointer_pos = response.hover_pos();

        // Set grab icon when hovering an element
        if let Some(pointer_pos) = &pointer_pos {
            if self.is_hovering_element(pointer_pos)
                || self.is_hovering_connection_point(pointer_pos)
            {
                ui.ctx()
                    .output_mut(|output| output.cursor_icon = CursorIcon::Grab);
            }
        }

        // Detect the action
        if let Some(pointer_pos) = &pointer_pos {
            if response.clicked_by(PointerButton::Secondary) {
                println!("Context menu opened");
                self.action = Some(Action::ContextMenu(pointer_pos.clone()));
            } else if response.drag_started() {
                println!("Checking drag start at {:?}", pointer_pos);
                if let Some(drag_connection_point) = self
                    .find_hovered_connection_point(pointer_pos)
                    .map(|(idx1, idx2)| Action::DragConnection(idx1, idx2))
                {
                    self.action = Some(drag_connection_point);
                    println!("Connection point drag started");
                }
                if let Some(drag_element_action) = self
                    .find_hovered_element_idx(pointer_pos)
                    .map(|idx| Action::DragElement(idx))
                {
                    self.action = Some(drag_element_action);
                    println!("Element drag started");
                }
            } else if response.drag_stopped() {
                self.action.take();
                println!("Drag stopped");
            }
        }

        // Context menu
        if let Some(Action::ContextMenu(pointer_pos)) = self.action.clone() {
            response.context_menu(|ui| self.draw_context_menu(&pointer_pos, ui));
        }

        // Connect action
        if let Some(pointer_pos) = pointer_pos {
            match self.action {
                Some(Action::Connect(start)) => {
                    if let Some(end) = self.find_hovered_element_idx(&pointer_pos) {
                        if response.clicked() {
                            if start != end {
                                self.connections.push(Connection::new(start, end));
                                println!("Added connection from {} to {}", start, end);
                            }
                            self.action.take();
                            println!("Connect action ended");
                        }
                    }
                }
                Some(Action::DragElement(idx)) => {
                    self.elements[idx].position = Pos::from_pos2(&self.grid, &pointer_pos)
                }
                Some(Action::DragConnection(conn_idx, midpoint_idx)) => {
                    self.connections[conn_idx].middle_points[midpoint_idx] =
                        Pos::from_pos2(&self.grid, &pointer_pos);
                }
                _ => {}
            }
        }

        PaneResponse::default()
    }

    fn contains_pointer(&self) -> bool {
        false
    }
}

impl PidPane {
    fn is_hovering_element(&self, pointer_pos: &Pos2) -> bool {
        self.elements
            .iter()
            .find(|element| element.contains(&self.grid, pointer_pos))
            .is_some()
    }

    fn is_hovering_connection_point(&self, pointer_pos: &Pos2) -> bool {
        self.connections
            .iter()
            .find(|conn| {
                conn.middle_points
                    .iter()
                    .find(|p| p.distance(&self.grid, pointer_pos) < 10.0)
                    .is_some()
            })
            .is_some()
    }

    /// Returns the currently used theme
    fn find_theme(ctx: &Context) -> Theme {
        // In Egui you can either decide a theme or use the system one.
        // If the system theme cannot be determined, a fallback theme can be set.
        ctx.options(|options| match options.theme_preference {
            egui::ThemePreference::Light => Theme::Light,
            egui::ThemePreference::Dark => Theme::Dark,
            egui::ThemePreference::System => match ctx.system_theme() {
                Some(Theme::Light) => Theme::Light,
                Some(Theme::Dark) => Theme::Dark,
                None => options.fallback_theme,
            },
        })
    }

    fn dots_color(theme: Theme) -> Color32 {
        match theme {
            Theme::Dark => Color32::DARK_GRAY,
            Theme::Light => Color32::BLACK,
        }
    }

    fn find_hovered_element_idx(&self, pos: &Pos2) -> Option<usize> {
        self.elements
            .iter()
            .position(|elem| elem.contains(&self.grid, pos))
    }

    fn find_hovered_element_mut(&mut self, pos: &Pos2) -> Option<&mut Element> {
        self.elements
            .iter_mut()
            .find(|element| element.contains(&self.grid, pos))
    }

    /// Return the connection and segment indexes where the position is on, if any
    fn find_hovered_connection_idx(&self, pos: &Pos2) -> Option<(usize, usize)> {
        self.connections
            .iter()
            .enumerate()
            .find_map(|(idx, conn)| Some(idx).zip(conn.contains(&self, pos)))
    }

    fn find_hovered_connection_point(&self, pos: &Pos2) -> Option<(usize, usize)> {
        let mut midpoint_idx = Some(0);
        let connection_idx = self.connections.iter().position(|conn| {
            midpoint_idx = conn
                .middle_points
                .iter()
                .position(|p| p.distance(&self.grid, pos) < 12.0);
            midpoint_idx.is_some()
        });

        connection_idx.zip(midpoint_idx)
    }

    fn draw_grid(&self, theme: Theme, ui: &Ui) {
        let painter = ui.painter();
        let window_rect = ui.max_rect();
        let dot_color = PidPane::dots_color(theme);

        for x in (window_rect.min.x as i32..window_rect.max.x.round() as i32)
            .step_by(self.grid.size as usize)
        {
            for y in (window_rect.min.y as i32..window_rect.max.y.round() as i32)
                .step_by(self.grid.size as usize)
            {
                let rect = egui::Rect::from_min_size(
                    egui::Pos2::new(x as f32, y as f32),
                    egui::Vec2::new(1.0, 1.0),
                );
                painter.rect_filled(rect, 0.0, dot_color);
            }
        }
    }

    fn draw_connections(&self, ui: &Ui) {
        let painter = ui.painter();

        for connection in &self.connections {
            let mut points = Vec::new();

            // Append start point
            let start = &self.elements[connection.start];
            points.push(start.position.into_pos2(&self.grid));

            // Append all midpoints
            connection
                .middle_points
                .iter()
                .map(|p| p.into_pos2(&self.grid))
                .for_each(|p| points.push(p));

            // Append end point
            let end = &self.elements[connection.end];
            points.push(end.position.into_pos2(&self.grid));

            // Draw line segments
            for i in 0..(points.len() - 1) {
                let a = points[i];
                let b = points[i + 1];
                painter.line_segment([a, b], PathStroke::new(1.0, Color32::GREEN));
            }

            // Draw dragging boxes
            for middle_point in &connection.middle_points {
                painter.rect(
                    egui::Rect::from_center_size(
                        middle_point.into_pos2(&self.grid),
                        Vec2::new(self.grid.size, self.grid.size),
                    ),
                    Rounding::ZERO,
                    Color32::DARK_GRAY,
                    Stroke::NONE,
                );
            }
        }
    }

    fn draw_elements(&self, theme: Theme, ui: &Ui) {
        for element in &self.elements {
            let image_rect = egui::Rect::from_center_size(
                egui::Pos2::new(
                    element.position.x as f32 * self.grid.size,
                    element.position.y as f32 * self.grid.size,
                ),
                egui::Vec2::new(
                    element.size as f32 * self.grid.size,
                    element.size as f32 * self.grid.size,
                ),
            );

            egui::Image::new(element.symbol.get_image(theme))
                .rotate(element.rotation, Vec2::new(0.5, 0.5))
                .paint_at(ui, image_rect);
        }
    }

    fn draw_context_menu(&mut self, pointer_pos: &Pos2, ui: &mut Ui) {
        ui.set_max_width(100.0); // To make sure we wrap long text

        if self.is_hovering_element(&pointer_pos) {
            let hovered_element = self.find_hovered_element_idx(&pointer_pos);
            if ui.button("Connect").clicked() {
                if let Some(idx) = hovered_element {
                    println!("Connect action started");
                    self.action = Some(Action::Connect(idx));
                } else {
                    panic!("No element found where the \"Connect\" action was issued");
                }
                ui.close_menu();
            }
            if ui.button("Rotate 90° ⟲").clicked() {
                if let Some(elem) = self.find_hovered_element_mut(&pointer_pos) {
                    elem.rotation += PI / 2.0;
                } else {
                    panic!("No element found where the \"Rotate 90° ⟲\" action was issued");
                }
                ui.close_menu();
            }
            if ui.button("Rotate 90° ⟳").clicked() {
                if let Some(elem) = self.find_hovered_element_mut(&pointer_pos) {
                    elem.rotation -= PI / 2.0;
                } else {
                    panic!("No element found where the \"Rotate 90° ⟳\" action was issued");
                }
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                if let Some(idx) = self.find_hovered_element_idx(&pointer_pos) {
                    self.delete_element(idx);
                } else {
                    panic!("No element found where the \"Delete\" action was issued");
                }
                ui.close_menu();
            }
        } else if let Some((conn_idx, segm_idx)) = self.find_hovered_connection_idx(&pointer_pos) {
            if ui.button("Split").clicked() {
                println!("Splitting connection line");
                self.connections[conn_idx].split(segm_idx, Pos::from_pos2(&self.grid, pointer_pos));
                ui.close_menu();
            }
        } else {
            ui.menu_button("Symbols", |ui| {
                for symbol in Symbol::iter() {
                    if ui.button(symbol.to_string()).clicked() {
                        self.elements.push(Element::new(
                            Pos::from_pos2(&self.grid, &pointer_pos),
                            symbol,
                        ));
                        ui.close_menu();
                    }
                }
            });
        }
    }

    fn delete_element(&mut self, idx: usize) {
        // First delete connection referencing this element
        self.connections
            .retain(|elem| elem.start != idx && elem.end != idx);

        // Then the element
        self.elements.remove(idx);
    }
}
