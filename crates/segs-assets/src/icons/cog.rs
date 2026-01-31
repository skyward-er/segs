use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Cog;

impl Icon for Cog {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::COG
    }
}
