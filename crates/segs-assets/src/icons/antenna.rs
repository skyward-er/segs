use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Antenna;

impl Icon for Antenna {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::ANTENNA
    }
}
