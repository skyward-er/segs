use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Trash;

impl Icon for Trash {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::TRASH
    }
}
