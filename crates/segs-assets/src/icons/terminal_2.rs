use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Terminal2 {
    solid: bool,
}

impl Terminal2 {
    pub fn solid() -> Self {
        Self { solid: true }
    }

    pub fn outline() -> Self {
        Self { solid: false }
    }
}

impl Icon for Terminal2 {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.solid {
            true => &svgs::TERMINAL_2_SOLID,
            false => &svgs::TERMINAL_2_OUTLINE,
        }
    }
}
