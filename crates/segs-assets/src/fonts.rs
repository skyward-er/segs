mod figtree;

use std::fmt::Display;

use egui::{FontData, FontId};
pub use figtree::Figtree;

pub trait Font {
    fn name(&self) -> String;
    fn bytes(&self) -> &[u8];

    fn family(&self) -> egui::FontFamily {
        egui::FontFamily::Name(self.name().into())
    }

    fn sized(&self, size: f32) -> FontId {
        FontId::new(size, self.family())
    }

    fn data(&self) -> FontData {
        FontData::from_owned(self.bytes().to_vec())
    }
}

/// Different weights available for fonts.
#[derive(Clone, Copy, Default)]
pub enum Weight {
    Black,
    ExtraBold,
    Bold,
    SemiBold,
    Medium,
    #[default]
    Regular,
    Light,
}

/// Different italic styles available for fonts.
#[derive(Clone, Copy, Default)]
pub enum Italic {
    Italic,
    #[default]
    NoItalic,
}

impl Display for Weight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let weight_str = match self {
            Weight::Black => "Black",
            Weight::ExtraBold => "ExtraBold",
            Weight::Bold => "Bold",
            Weight::SemiBold => "SemiBold",
            Weight::Medium => "Medium",
            Weight::Regular => "Regular",
            Weight::Light => "Light",
        };
        write!(f, "{}", weight_str)
    }
}

impl Display for Italic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let italic_str = match self {
            Italic::Italic => "Italic",
            Italic::NoItalic => "",
        };
        write!(f, "{}", italic_str)
    }
}
