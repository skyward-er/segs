mod animation;
pub mod containers;
mod style;
pub mod utils;
pub mod widgets;

use std::sync::Arc;

pub use animation::AnimationExt;
use egui::{Context, Response, Theme, Ui};
pub use style::setup_style;

use crate::style::{AppStyle, AppVisuals};

/// Generic extension trait for egui::Ui..
pub trait UiExt {
    fn pointer_clicked_outside(&self, response: &Response) -> bool;
}

impl UiExt for Ui {
    fn pointer_clicked_outside(&self, response: &Response) -> bool {
        self.ctx().pointer_clicked_outside(response)
    }
}

impl UiExt for Context {
    fn pointer_clicked_outside(&self, response: &Response) -> bool {
        utils::pointer_clicked_outside(self, response)
    }
}

/// Extension trait for egui::Ui to get app style and visuals.
pub trait StyleExt {
    /// Get the current app style based on the UI theme.
    fn app_style(&self) -> Arc<AppStyle>;

    /// Get the current app visuals based on the UI theme.
    fn app_visuals(&self) -> &AppVisuals;
}

impl StyleExt for Ui {
    fn app_style(&self) -> Arc<AppStyle> {
        self.ctx().app_style()
    }

    fn app_visuals(&self) -> &AppVisuals {
        self.ctx().app_visuals()
    }
}

impl StyleExt for Context {
    fn app_style(&self) -> Arc<AppStyle> {
        let app_style = match self.theme() {
            Theme::Dark => style::DARK.get().unwrap().clone(),
            Theme::Light => style::LIGHT.get().unwrap().clone(),
        };
        app_style.set_egui_style(self.style().clone());
        app_style
    }

    fn app_visuals(&self) -> &AppVisuals {
        let app_style = match self.theme() {
            Theme::Dark => &style::DARK.get().unwrap(),
            Theme::Light => &style::LIGHT.get().unwrap(),
        };
        app_style.set_egui_style(self.style().clone());
        &app_style.visuals
    }
}
