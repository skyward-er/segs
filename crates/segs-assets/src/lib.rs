#![allow(dead_code)]

pub mod fonts;
pub mod icons;
mod sources;

use std::sync::Arc;

use egui::{FontDefinitions, FontFamily};
pub use fonts::Font;

use crate::fonts::Figtree;

/// Loads all used fonts into the given egui context.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // 1. Add custom fonts and fontdata
    for font in all_fonts() {
        let key = font.name();
        fonts.font_data.insert(key.clone(), Arc::new(font.data()));
        fonts.families.entry(font.family()).or_default().insert(0, key);
    }

    // 2. Set default font families
    fonts
        .families
        .insert(FontFamily::Proportional, vec![Figtree::medium().name()]);
    fonts
        .families
        .insert(FontFamily::Monospace, vec![Figtree::regular().name()]);

    ctx.set_fonts(fonts);
}

/// Load all icons into the given egui context.
pub fn install_icons(ctx: &egui::Context) {
    egui_extras::install_image_loaders(ctx);
}

/// Load the application icon.
pub fn load_app_icon() -> egui::IconData {
    eframe::icon_data::from_png_bytes(sources::icons::SEGS_1024X1024).expect("Failed to load icon data")
}

/// Returns a list of all fonts used in the application.
fn all_fonts() -> Vec<Box<dyn Font>> {
    Figtree::all()
        .into_iter()
        .map(|f| Box::new(f) as Box<dyn Font>)
        .collect()
}
