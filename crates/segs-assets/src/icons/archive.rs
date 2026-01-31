use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Archive;

impl Icon for Archive {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::ARCHIVE
    }
}
