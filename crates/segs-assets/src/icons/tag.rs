use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Tag;

impl Icon for Tag {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::TAG
    }
}
