use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Documents;

impl Icon for Documents {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::DOCUMENTS
    }
}
