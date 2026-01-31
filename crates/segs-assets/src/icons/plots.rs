use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Plots;

impl Icon for Plots {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::PLOTS
    }
}
