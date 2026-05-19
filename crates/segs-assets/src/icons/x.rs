use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct X;

impl Icon for X {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::X
    }
}
