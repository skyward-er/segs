use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Error;

impl Icon for Error {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::CIRCLED_CROSS
    }
}
