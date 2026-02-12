use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct SquareRotated;

impl Icon for SquareRotated {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::SQUARE_ROTATED_SOLID
    }
}
