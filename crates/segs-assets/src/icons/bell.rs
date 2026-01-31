use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Bell {
    solid: bool,
}

impl Bell {
    pub fn solid() -> Self {
        Self { solid: true }
    }

    pub fn outline() -> Self {
        Self { solid: false }
    }
}

impl Icon for Bell {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.solid {
            true => &svgs::BELL_SOLID,
            false => &svgs::BELL_OUTLINE,
        }
    }
}
