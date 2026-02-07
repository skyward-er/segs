use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Ethernet;

impl Icon for Ethernet {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::ETHERNET_PORT
    }
}
