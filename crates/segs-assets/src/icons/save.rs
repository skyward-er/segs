use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Save;

impl Icon for Save {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::SAVE
    }
}
