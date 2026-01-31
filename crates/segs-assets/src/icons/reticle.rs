use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Reticle {
    variant: Variant,
}

#[derive(Clone, Copy, Default)]
enum Variant {
    #[default]
    Empty,
    Solid,
    Outline,
}

impl Reticle {
    // Additional constructors for different variants
    pub fn empty() -> Self {
        Self {
            variant: Variant::Empty,
        }
    }

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

impl Icon for Reticle {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.variant {
            Variant::Empty => &svgs::RETICLE_EMPTY,
            Variant::Solid => &svgs::RETICLE_SOLID,
            Variant::Outline => &svgs::RETICLE_OUTLINE,
        }
    }
}
