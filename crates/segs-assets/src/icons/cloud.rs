use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct Cloud {
    variant: Variant,
}

impl Icon for Cloud {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match self.variant {
            Variant::Outline => &svgs::CLOUD_OUTLINE,
            Variant::Solid => &svgs::CLOUD_SOLID,
            Variant::Plus => &svgs::CLOUD_PLUS,
            Variant::Download => &svgs::CLOUD_DOWNLOAD,
        }
    }
}

#[derive(Clone, Copy, Default)]
enum Variant {
    #[default]
    Outline,
    Solid,
    Plus,
    Download,
}

impl Cloud {
    pub fn solid() -> Self {
        Self {
            variant: Variant::Solid,
        }
    }

    pub fn outline() -> Self {
        Self {
            variant: Variant::Outline,
        }
    }

    pub fn plus() -> Self {
        Self { variant: Variant::Plus }
    }

    pub fn download() -> Self {
        Self {
            variant: Variant::Download,
        }
    }
}
