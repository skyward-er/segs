mod connections;
mod elements;
mod pos;
mod symbols;

use connections::Connection;
use egui::{epaint::PathStroke, Color32, Context, PointerButton, Pos2, Sense, Theme, Ui, Vec2};
use elements::Element;
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
    ContextMenu(Pos),
    Drag(usize),
}

/// Piping and instrumentation diagram
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct PidPane {
    elements: Vec<Element>,
    connections: Vec<Connection>,

    grid_size: f32,

    #[serde(skip)]
    action: Option<Action>,
}

impl Default for PidPane {
    fn default() -> Self {
        Self {
            elements: Vec::default(),
            connections: Vec::default(),
            grid_size: 10.0,

            action: None,
        }
    }
}

impl PaneBehavior for PidPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        // Set cursor icon
        // ui.ctx().output_mut(|output| output.cursor_icon = CursorIcon::Grab);

        let theme = PidPane::find_theme(ui.ctx());
        self.draw_grid(theme, ui);
        self.draw_connections(ui);
        self.draw_elements(theme, ui);

        // Allocate the space to sense inputs
        let (_, response) = ui.allocate_at_least(ui.max_rect().size(), Sense::click_and_drag());
        let pointer_pos = response.hover_pos().map(|pos| self.screen_to_grid_pos(pos));

        // Detect the action
        if let Some(pointer_pos) = &pointer_pos {
            if response.clicked_by(PointerButton::Secondary) {
                println!("Context menu opened");
                self.action = Some(Action::ContextMenu(pointer_pos.clone()));
            } else if response.drag_started() {
                println!("Drag started");
                self.action = self
                    .find_hovered_element_idx(pointer_pos)
                    .map(|idx| Action::Drag(idx));
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
                                self.connections.push(Connection { start, end });
                                println!("Added connection from {} to {}", start, end);
                            }
                            self.action.take();
                            println!("Connect action ended");
                        }
                    }
                }
                Some(Action::Drag(idx)) => {
                    let element = &mut self.elements[idx];
                    element.position.x = pointer_pos.x - element.size / 2;
                    element.position.y = pointer_pos.y - element.size / 2;
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
    fn is_hovering_element(&self, pointer_pos: &Pos) -> bool {
        self.elements
            .iter()
            .find(|element| element.contains(pointer_pos))
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

    fn screen_to_grid_pos(&self, screen_pos: Pos2) -> Pos {
        Pos {
            x: (screen_pos.x / self.grid_size) as i32,
            y: (screen_pos.y / self.grid_size) as i32,
        }
    }

    fn find_hovered_element_idx(&self, pos: &Pos) -> Option<usize> {
        self.elements.iter().position(|elem| elem.contains(&pos))
    }

    fn find_hovered_element_mut(&mut self, pos: &Pos) -> Option<&mut Element> {
        self.elements
            .iter_mut()
            .find(|element| element.contains(&pos))
    }

    fn draw_grid(&self, theme: Theme, ui: &Ui) {
        let painter = ui.painter();
        let window_rect = ui.max_rect();
        let dot_color = PidPane::dots_color(theme);

        for x in (window_rect.min.x as i32..window_rect.max.x.round() as i32)
            .step_by(self.grid_size as usize)
        {
            for y in (window_rect.min.y as i32..window_rect.max.y.round() as i32)
                .step_by(self.grid_size as usize)
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
            let elem1 = &self.elements[connection.start];
            let elem2 = &self.elements[connection.end];

            let x1 = (elem1.position.x + elem1.size / 2) as f32 * self.grid_size;
            let y1 = (elem1.position.y + elem1.size / 2) as f32 * self.grid_size;
            let x2 = (elem2.position.x + elem2.size / 2) as f32 * self.grid_size;
            let y2 = (elem2.position.y + elem2.size / 2) as f32 * self.grid_size;

            painter.line_segment(
                [Pos2::new(x1, y1), Pos2::new(x2, y2)],
                PathStroke::new(1.0, Color32::GREEN),
            );
        }
    }

    fn draw_elements(&self, theme: Theme, ui: &Ui) {
        for element in &self.elements {
            let image_rect = egui::Rect::from_min_size(
                egui::Pos2::new(
                    element.position.x as f32 * self.grid_size,
                    element.position.y as f32 * self.grid_size,
                ),
                egui::Vec2::new(
                    element.size as f32 * self.grid_size,
                    element.size as f32 * self.grid_size,
                ),
            );

            egui::Image::new(element.symbol.get_image(theme))
                .rotate(element.rotation, Vec2::new(0.5, 0.5))
                .paint_at(ui, image_rect);
        }
    }

    fn draw_context_menu(&mut self, pointer_pos: &Pos, ui: &mut Ui) {
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
        } else {
            ui.menu_button("Symbols", |ui| {
                for symbol in Symbol::iter() {
                    if ui.button(symbol.to_string()).clicked() {
                        self.elements.push(Element::new(pointer_pos, symbol));
                        ui.close_menu();
                    }
                }
            });
        }
    }
}
