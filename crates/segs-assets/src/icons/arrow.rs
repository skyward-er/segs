use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Arrow {
    variant: ArrowVariant,
}

impl Arrow {
    pub fn narrow_right() -> Self {
        Self {
            variant: ArrowVariant::NarrowRight,
        }
    }
}

impl Icon for Arrow {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.variant {
            ArrowVariant::NarrowRight => &svgs::ARROW_NARROW_RIGHT,
        }
    }
}

#[derive(Clone, Copy, Default)]
enum ArrowVariant {
    #[default]
    NarrowRight,
}
