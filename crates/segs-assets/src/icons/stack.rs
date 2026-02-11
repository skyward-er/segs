use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Stack {
    variant: Variant,
}

impl Icon for Stack {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.variant {
            Variant::Outline => &svgs::STACK_OUTLINE,
            Variant::Solid => &svgs::STACK_SOLID,
        }
    }
}

#[derive(Clone, Copy, Default)]
enum Variant {
    #[default]
    Outline,
    Solid,
}

impl Stack {
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
