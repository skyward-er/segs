use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct RectangleVertical {
    variant: Variant,
}

impl Icon for RectangleVertical {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.variant {
            Variant::Outline => &svgs::RECTANGLE_VERTICAL_OUTLINE,
            Variant::Solid => &svgs::RECTANGLE_VERTICAL_SOLID,
        }
    }
}

#[derive(Clone, Copy, Default)]
enum Variant {
    #[default]
    Outline,
    Solid,
}

impl RectangleVertical {
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
