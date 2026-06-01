use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Settings {
    variant: Variant,
}

impl Icon for Settings {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.variant {
            Variant::Outline => &svgs::SETTINGS_OUTLINE,
            Variant::Solid => &svgs::SETTINGS_SOLID,
        }
    }
}

#[derive(Clone, Copy, Default)]
enum Variant {
    #[default]
    Outline,
    Solid,
}

impl Settings {
    pub fn solid() -> Self {
        Self {
            variant: Variant::Solid,
        }
    }

    pub fn outline() -> Self {
        Self {
            variant: Variant::Outline,
        }
    }
}
