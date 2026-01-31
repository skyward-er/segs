use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct AddLayout;

impl Icon for AddLayout {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::ADD_LAYOUT
    }
}
