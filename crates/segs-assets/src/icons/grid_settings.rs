use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct GridSettings;

impl Icon for GridSettings {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::GRID_SETTINGS
    }
}
