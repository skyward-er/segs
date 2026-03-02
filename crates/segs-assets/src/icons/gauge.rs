use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Gauge;

impl Icon for Gauge {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::GAUGE
    }
}
