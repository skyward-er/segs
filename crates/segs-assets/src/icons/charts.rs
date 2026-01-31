use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Charts;

impl Icon for Charts {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::CHARTS
    }
}
