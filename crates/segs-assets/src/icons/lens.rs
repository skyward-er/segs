use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Lens;

impl Icon for Lens {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::LENS
    }
}
