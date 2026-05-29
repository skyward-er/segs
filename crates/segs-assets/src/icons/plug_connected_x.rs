use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct PlugConnectedX;

impl Icon for PlugConnectedX {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::PLUG_CONNECTED_X
    }
}
