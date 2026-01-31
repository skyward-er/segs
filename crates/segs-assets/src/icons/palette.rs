use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Palette;

impl Icon for Palette {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::PALETTE
    }
}
