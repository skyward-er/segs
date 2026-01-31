use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Sun {
    variant: Variant,
}

#[derive(Clone, Copy, Default)]
pub struct Moon {
    variant: Variant,
}

#[derive(Clone, Copy, Default)]
enum Variant {
    #[default]
    Outline,
    Solid,
}

impl Sun {
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

impl Moon {
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

impl Icon for Sun {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.variant {
            Variant::Outline => &svgs::SUN_OUTLINE,
            Variant::Solid => &svgs::SUN_SOLID,
        }
    }
}

impl Icon for Moon {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.variant {
            Variant::Outline => &svgs::MOON_OUTLINE,
            Variant::Solid => &svgs::MOON_SOLID,
        }
    }
}
