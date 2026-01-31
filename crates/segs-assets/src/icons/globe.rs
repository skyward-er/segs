use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Globe;

impl Icon for Globe {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::GLOBE
    }
}
