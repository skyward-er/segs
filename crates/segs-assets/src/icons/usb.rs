use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Usb;

impl Icon for Usb {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::USB
    }
}
