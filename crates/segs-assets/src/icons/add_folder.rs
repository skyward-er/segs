use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct AddFolder;

impl Icon for AddFolder {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::ADD_FOLDER
    }
}
