use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Lightning;

impl Icon for Lightning {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::LIGHTNING
    }
}
