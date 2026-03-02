use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Tools;

impl Icon for Tools {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::TOOLS
    }
}
