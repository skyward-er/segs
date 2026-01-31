mod components;
mod style;
pub mod widgets;

use std::sync::Arc;

use crate::style::{Style, Visuals};

pub use components::UiComponentExt;
pub use style::setup_style;

pub trait StyleExt {
    fn app_style(&self) -> Arc<Style>;
    fn app_visuals(&self) -> Arc<Visuals>;
}

impl StyleExt for egui::Ui {
    fn app_style(&self) -> Arc<Style> {
        if self.visuals().dark_mode {
            style::DARK.clone()
        } else {
            style::LIGHT.clone()
        }
    }

    fn app_visuals(&self) -> Arc<Visuals> {
        self.app_style().visuals.clone()
    }
}
