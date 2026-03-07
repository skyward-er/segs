use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct CaretDown {
    variant: Variant,
}

#[derive(Clone, Copy, Default)]
enum Variant {
    #[default]
    Outline,
    Solid,
}

impl Icon for CaretDown {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.variant {
            Variant::Outline => &svgs::CARET_DOWN_OUTLINE,
            Variant::Solid => &svgs::CARET_DOWN_SOLID,
        }
    }
}

impl CaretDown {
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
