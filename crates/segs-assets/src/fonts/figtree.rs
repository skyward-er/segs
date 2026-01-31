use super::{Font, Italic, Weight};
use crate::sources::fonts;

/// Represents a font available in the assets.
#[derive(Clone, Copy, Default)]
pub struct Figtree {
    weight: Weight,
    italic: Italic,
}

impl Font for Figtree {
    fn name(&self) -> String {
        format!("Figtree-{}{}", self.weight, self.italic)
    }

    fn bytes(&self) -> &[u8] {
        match (self.weight, self.italic) {
            (Weight::Black, Italic::Italic) => fonts::FIGTREE_BLACK_ITALIC,
            (Weight::Black, Italic::NoItalic) => fonts::FIGTREE_BLACK,
            (Weight::ExtraBold, Italic::Italic) => fonts::FIGTREE_EXTRA_BOLD_ITALIC,
            (Weight::ExtraBold, Italic::NoItalic) => fonts::FIGTREE_EXTRA_BOLD,
            (Weight::Bold, Italic::Italic) => fonts::FIGTREE_BOLD_ITALIC,
            (Weight::Bold, Italic::NoItalic) => fonts::FIGTREE_BOLD,
            (Weight::SemiBold, Italic::Italic) => fonts::FIGTREE_SEMI_BOLD_ITALIC,
            (Weight::SemiBold, Italic::NoItalic) => fonts::FIGTREE_SEMI_BOLD,
            (Weight::Medium, Italic::Italic) => fonts::FIGTREE_MEDIUM_ITALIC,
            (Weight::Medium, Italic::NoItalic) => fonts::FIGTREE_MEDIUM,
            (Weight::Regular, Italic::Italic) => fonts::FIGTREE_ITALIC,
            (Weight::Regular, Italic::NoItalic) => fonts::FIGTREE_REGULAR,
            (Weight::Light, Italic::Italic) => fonts::FIGTREE_LIGHT_ITALIC,
            (Weight::Light, Italic::NoItalic) => fonts::FIGTREE_LIGHT,
        }
    }
}

impl Figtree {
    pub fn all() -> Vec<Self> {
        vec![
            Self::black(),
            Self::black().italic(),
            Self::extra_bold(),
            Self::extra_bold().italic(),
            Self::bold(),
            Self::bold().italic(),
            Self::semi_bold(),
            Self::semi_bold().italic(),
            Self::medium(),
            Self::medium().italic(),
            Self::regular(),
            Self::regular().italic(),
            Self::light(),
            Self::light().italic(),
        ]
    }

    pub const fn black() -> Self {
        Self {
            weight: Weight::Black,
            italic: Italic::NoItalic,
        }
    }

    pub const fn extra_bold() -> Self {
        Self {
            weight: Weight::ExtraBold,
            italic: Italic::NoItalic,
        }
    }

    pub const fn bold() -> Self {
        Self {
            weight: Weight::Bold,
            italic: Italic::NoItalic,
        }
    }

    pub const fn semi_bold() -> Self {
        Self {
            weight: Weight::SemiBold,
            italic: Italic::NoItalic,
        }
    }

    pub const fn medium() -> Self {
        Self {
            weight: Weight::Medium,
            italic: Italic::NoItalic,
        }
    }

    pub const fn regular() -> Self {
        Self {
            weight: Weight::Regular,
            italic: Italic::NoItalic,
        }
    }

    pub const fn light() -> Self {
        Self {
            weight: Weight::Light,
            italic: Italic::NoItalic,
        }
    }

    pub const fn italic(mut self) -> Self {
        self.italic = Italic::Italic;
        self
    }
}
