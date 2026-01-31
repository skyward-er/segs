use super::Icon;
use crate::sources::svgs;

#[derive(Clone, Copy, Default)]
pub struct PanelToggle {
    variant: PanelVariant,
    solid: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum PanelVariant {
    #[default]
    Left,
    Right,
    Bottom,
}

impl PanelToggle {
    pub fn left_panel() -> Self {
        Self {
            variant: PanelVariant::Left,
            ..Default::default()
        }
    }

    pub fn right_panel() -> Self {
        Self {
            variant: PanelVariant::Right,
            ..Default::default()
        }
    }

    pub fn bottom_panel() -> Self {
        Self {
            variant: PanelVariant::Bottom,
            ..Default::default()
        }
    }

    pub fn solid(mut self) -> Self {
        self.solid = true;
        self
    }
}

impl Icon for PanelToggle {
    fn as_image_source(&self) -> &egui::ImageSource<'static> {
        match (self.variant, self.solid) {
            (PanelVariant::Left, true) => &svgs::LEFT_PANEL_SOLID,
            (PanelVariant::Left, false) => &svgs::LEFT_PANEL_OUTLINE,
            (PanelVariant::Right, true) => &svgs::RIGHT_PANEL_SOLID,
            (PanelVariant::Right, false) => &svgs::RIGHT_PANEL_OUTLINE,
            (PanelVariant::Bottom, true) => &svgs::BOTTOM_PANEL_SOLID,
            (PanelVariant::Bottom, false) => &svgs::BOTTOM_PANEL_OUTLINE,
        }
    }
}
