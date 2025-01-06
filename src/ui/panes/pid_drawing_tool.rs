mod pid_elements;

use egui::{Sense, Theme};
use pid_elements::PidElement;
use serde::{Deserialize, Serialize};

use crate::ui::composable_view::PaneResponse;

use super::PaneBehavior;

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PidPane {
    elements: Vec<PidElement>,
    dragged: Option<usize>,
}

impl PidPane {
    pub fn new() -> Self {
        let mut main_window = PidPane::default();
        main_window.elements = vec![
            PidElement {
                x: 2,
                y: 2,
                size: 10,
            },
            PidElement {
                x: 10,
                y: 4,
                size: 10,
            },
        ];
        main_window
    }
}

impl PaneBehavior for PidPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        let step_size: i32 = 10;
        let window_rect = ui.max_rect();
        let painter = ui.painter();

        let theme = ui.ctx().options(|options| match options.theme_preference {
            egui::ThemePreference::Light => Theme::Light,
            egui::ThemePreference::Dark => Theme::Dark,
            egui::ThemePreference::System => match ui.ctx().system_theme() {
                Some(Theme::Light) => Theme::Light,
                Some(Theme::Dark) => Theme::Dark,
                None => options.fallback_theme,
            },
        });

        // Draw the dot grid
        let dot_color = match theme {
            Theme::Dark => egui::Color32::DARK_GRAY,
            Theme::Light => egui::Color32::BLACK,
        };
        for x in
            (window_rect.min.x as i32..window_rect.max.x.round() as i32).step_by(step_size as usize)
        {
            for y in (window_rect.min.y as i32..window_rect.max.y.round() as i32)
                .step_by(step_size as usize)
            {
                let rect = egui::Rect::from_min_size(
                    egui::Pos2::new(x as f32, y as f32),
                    egui::Vec2::new(1.0, 1.0),
                );
                painter.rect_filled(rect, 0.0, dot_color);
            }
        }

        // Draw elements
        for element in &self.elements {
            let image_rect = egui::Rect::from_min_size(
                egui::Pos2::new(
                    (element.x * step_size) as f32,
                    (element.y * step_size) as f32,
                ),
                egui::Vec2::new(
                    (element.size * step_size) as f32,
                    (element.size * step_size) as f32,
                ),
            );

            match theme {
                Theme::Light => {
                    egui::Image::new(egui::include_image!("../../../icons/ball_valve.svg"))
                        .paint_at(ui, image_rect);
                }
                Theme::Dark => {
                    egui::Image::new(egui::include_image!("../../../icons/ball_valve_dark.svg"))
                        .paint_at(ui, image_rect);
                }
            }
        }

        let (_, response) = ui.allocate_at_least(window_rect.size(), Sense::drag());

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let x = pointer_pos.x;
            let y = pointer_pos.y;
            let x_grid = x as i32 / step_size;
            let y_grid = y as i32 / step_size;

            if response.drag_started() {
                // Find which element the drag started on
                self.dragged = self
                    .elements
                    .iter()
                    .position(|element| element.contains(x_grid, y_grid));
            }

            if response.dragged() {
                if let Some(dragged) = self.dragged {
                    let element = &mut self.elements[dragged];

                    element.x = x_grid - element.size / 2;
                    element.y = y_grid - element.size / 2;
                }
            }
            if response.drag_stopped() {
                // Reset dragged item
                self.dragged.take();
            }
        }

        PaneResponse::default()
    }

    fn contains_pointer(&self) -> bool {
        false
    }
}
