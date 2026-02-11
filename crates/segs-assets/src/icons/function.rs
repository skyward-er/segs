use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Function {
    variant: Variant,
}

impl Icon for Function {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.variant {
            Variant::Outline => &svgs::FUNCTION_OUTLINE,
            Variant::Solid => &svgs::FUNCTION_SOLID,
        }
    }
}

#[derive(Clone, Copy, Default)]
enum Variant {
    #[default]
    Outline,
    Solid,
}

impl Function {
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
