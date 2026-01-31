use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Arrow {
    up: bool,
}

impl Arrow {
    pub fn up() -> Self {
        Self { up: true }
    }

    pub fn down() -> Self {
        Self { up: false }
    }
}

impl Icon for Arrow {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.up {
            true => &svgs::ARROW_UP,
            false => &svgs::ARROW_DOWN,
        }
    }

    fn aspect_ratio(&self) -> f32 {
        4.0 / 5.0
    }
}
