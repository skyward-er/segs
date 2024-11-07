mod messages_viewer;
mod plot_2d;

use enum_dispatch::enum_dispatch;
use plot_2d::Plot2DPane;
use serde::{Deserialize, Serialize};

use super::composable_view::{PaneAction, PaneResponse};

#[enum_dispatch(Pane)]
pub trait PaneBehavior {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse;
    fn tab_title(&self) -> egui::WidgetText;
    fn contains_pointer(&self) -> bool;
}

// An enum to represent the different widgets available to the user.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[enum_dispatch]
pub enum Pane {
    Default(DefaultPane),
    MessagesViewer(messages_viewer::MessagesViewerPane),
    Plot2D(plot_2d::Plot2DPane),
}

impl Default for Pane {
    fn default() -> Self {
        Pane::Default(DefaultPane::default())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DefaultPane {
    occupied: f32,
    fixed: bool,
    contains_pointer: bool,
}

impl Default for DefaultPane {
    fn default() -> Self {
        DefaultPane {
            occupied: 0.0,
            fixed: false,
            contains_pointer: false,
        }
    }
}

impl PaneBehavior for DefaultPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        let mut response = PaneResponse::default();
        let pane_rect = ui.max_rect();

        let parent = ui.vertical_centered(|ui| {
            let hpad = (pane_rect.height() - self.occupied) / 2.0;
            if self.fixed {
                ui.add_space(hpad);
            }
            let mut height_occupied = 0.0;
            let btn = ui.button("Vertical Split");
            if btn.clicked() {
                response.set_action(PaneAction::SplitV);
                log::debug!("Vertical Split button clicked");
            }
            height_occupied += btn.rect.height();
            let btn = ui.button("Horizontal Split");
            if btn.clicked() {
                response.set_action(PaneAction::SplitH);
                log::debug!("Horizontal Split button clicked");
            }
            height_occupied += btn.rect.height();
            let btn = ui.button("Plot");
            if btn.clicked() {
                response.set_action(PaneAction::Replace(Box::new(Pane::Plot2D(
                    Plot2DPane::default(),
                ))));
            }
            height_occupied += btn.rect.height();
            if !self.fixed {
                self.occupied = height_occupied;
                ui.ctx().request_discard("test");
                self.fixed = true;
            }
            if self.fixed {
                ui.add_space(hpad);
            }
            ui.set_min_height(pane_rect.height());
        });

        self.contains_pointer = parent.response.contains_pointer();

        if parent
            .response
            .interact(egui::Sense::click_and_drag())
            .on_hover_cursor(egui::CursorIcon::Grab)
            .dragged()
        {
            response.set_drag_started();
        };

        response
    }

    fn tab_title(&self) -> egui::WidgetText {
        "Default".into()
    }

    fn contains_pointer(&self) -> bool {
        self.contains_pointer
    }
}
