use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Refresh;

impl Icon for Refresh {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::REFRESH
    }
}
