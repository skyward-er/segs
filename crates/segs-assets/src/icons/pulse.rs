use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Pulse;

impl Icon for Pulse {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::PULSE
    }
}
