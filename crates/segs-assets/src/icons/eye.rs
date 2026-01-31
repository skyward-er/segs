use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Eye {
    appearance: EyeAppearance,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Default)]
enum EyeAppearance {
    #[default]
    Open,
    Closed,
    Crossed,
}

impl Eye {
    pub fn open() -> Self {
        Self {
            appearance: EyeAppearance::Open,
        }
    }

    pub fn closed() -> Self {
        Self {
            appearance: EyeAppearance::Closed,
        }
    }

    pub fn crossed() -> Self {
        Self {
            appearance: EyeAppearance::Crossed,
        }
    }
}

impl Icon for Eye {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.appearance {
            EyeAppearance::Open => &svgs::EYE_OPEN,
            EyeAppearance::Closed => &svgs::EYE_CLOSED,
            EyeAppearance::Crossed => &svgs::EYE_CROSSED,
        }
    }
}
