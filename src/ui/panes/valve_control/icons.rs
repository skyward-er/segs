use egui::{Context, Image, ImageSource, SizeHint, TextureOptions, Theme, Ui};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tracing::error;

use crate::error::ErrInstrument;

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Icon {
    Wiggle,
    Aperture,
    Timing,
}

impl Icon {
    fn as_image_source(&self, theme: Theme) -> ImageSource {
        match (&self, theme) {
            (Icon::Wiggle, Theme::Light) => {
                egui::include_image!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/icons/valve_control/light/wiggle.svg"
                ))
            }
            (Icon::Wiggle, Theme::Dark) => {
                egui::include_image!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/icons/valve_control/dark/wiggle.svg"
                ))
            }
            (Icon::Aperture, Theme::Light) => {
                egui::include_image!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/icons/valve_control/light/aperture.svg"
                ))
            }
            (Icon::Aperture, Theme::Dark) => {
                egui::include_image!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/icons/valve_control/dark/aperture.svg"
                ))
            }
            (Icon::Timing, Theme::Light) => {
                egui::include_image!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/icons/valve_control/light/timing.svg"
                ))
            }
            (Icon::Timing, Theme::Dark) => {
                egui::include_image!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/icons/valve_control/dark/timing.svg"
                ))
            }
        }
    }

    pub fn init_cache(ctx: &Context, size_hint: (u32, u32)) {
        let size_hint = SizeHint::Size(size_hint.0, size_hint.1);
        for icon in Self::iter() {
            if let Err(e) =
                icon.as_image_source(ctx.theme())
                    .load(ctx, TextureOptions::LINEAR, size_hint)
            {
                error!("Error loading icons: {}", e);
            }
        }
    }

    pub fn as_image(&self, theme: Theme) -> Image {
        Image::new(self.as_image_source(theme))
    }
}
