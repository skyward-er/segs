mod components;
pub mod containers;
mod style;
pub mod widgets;

use std::sync::Arc;

pub use components::UiComponentExt;
use egui::Theme;
pub use style::setup_style;

use crate::style::{AppStyle, AppVisuals};

/// Extension trait for egui::Ui to get app style and visuals.
pub trait StyleExt {
    /// Get the current app style based on the UI theme.
    fn app_style(&self) -> Arc<AppStyle>;

    /// Get the current app visuals based on the UI theme.
    fn app_visuals(&self) -> &AppVisuals;
}

impl StyleExt for egui::Ui {
    fn app_style(&self) -> Arc<AppStyle> {
        let app_style = match self.ctx().theme() {
            Theme::Dark => style::DARK.get().unwrap().clone(),
            Theme::Light => style::LIGHT.get().unwrap().clone(),
        };
        app_style.set_egui_style(self.style().clone());
        app_style
    }

    fn app_visuals(&self) -> &AppVisuals {
        let app_style = match self.ctx().theme() {
            Theme::Dark => &style::DARK.get().unwrap(),
            Theme::Light => &style::LIGHT.get().unwrap(),
        };
        app_style.set_egui_style(self.style().clone());
        &app_style.visuals
    }
}
