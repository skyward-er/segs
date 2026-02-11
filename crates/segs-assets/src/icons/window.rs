use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Window {
    variant: Variant,
}

impl Icon for Window {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.variant {
            Variant::Outline => &svgs::WINDOW_OUTLINE,
            Variant::Solid => &svgs::WINDOW_SOLID,
        }
    }
}

#[derive(Clone, Copy, Default)]
enum Variant {
    #[default]
    Outline,
    Solid,
}

impl Window {
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
