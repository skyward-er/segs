use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Check;

impl Icon for Check {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::CHECK
    }
}
