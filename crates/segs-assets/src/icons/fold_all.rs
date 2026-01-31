use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Debug)]
pub struct FoldAll;

impl Icon for FoldAll {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        &svgs::FOLD_ALL
    }
}
