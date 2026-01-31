use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Network;

impl Icon for Network {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::NETWORK
    }
}
