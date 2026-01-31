use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct AddFile;

impl Icon for AddFile {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::ADD_FILE
    }
}
