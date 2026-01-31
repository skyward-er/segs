use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Warning;

impl Icon for Warning {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::ALERT
    }
}
