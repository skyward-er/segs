use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Layout;

impl Icon for Layout {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::LAYOUT
    }
}
