use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Lock {
    locked: bool,
}

impl Lock {
    pub fn locked() -> Self {
        Self { locked: true }
    }

    pub fn unlocked() -> Self {
        Self { locked: false }
    }
}

impl Icon for Lock {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.locked {
            true => &svgs::LOCK_LOCKED,
            false => &svgs::LOCK_UNLOCKED,
        }
    }
}
