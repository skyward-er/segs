use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Charts {
    variant: Variant,
}

impl Icon for Charts {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.variant {
            Variant::Outline => &svgs::CHARTS_OUTLINE,
            Variant::Solid => &svgs::CHARTS_SOLID,
        }
    }
}

#[derive(Clone, Copy, Default)]
enum Variant {
    #[default]
    Outline,
    Solid,
}

impl Charts {
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
