use egui::{
    Color32, FontId, Frame, KeyboardShortcut, Label, Margin, ModifierNames, RichText, Stroke,
    Widget,
};

pub struct ShortcutCard {
    shortcut: KeyboardShortcut,
    text_size: f32,
    margin: Margin,
    text_color: Option<Color32>,
    fill_color: Option<Color32>,
}

impl Widget for ShortcutCard {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        #[cfg(target_os = "macos")]
        let is_mac = true;
        #[cfg(not(target_os = "macos"))]
        let is_mac = false;

        let shortcut_fmt = self.shortcut.format(&ModifierNames::SYMBOLS, is_mac);
        let default_style = ui.style().noninteractive();
        let text_color = self.text_color.unwrap_or(default_style.text_color());
        let fill_color = self.fill_color.unwrap_or(default_style.bg_fill);
        let corner_radius = default_style.corner_radius;

        let number = RichText::new(shortcut_fmt)
            .color(text_color)
            .font(FontId::monospace(self.text_size));

        Frame::canvas(ui.style())
            .fill(fill_color)
            .stroke(Stroke::NONE)
            .inner_margin(self.margin)
            .corner_radius(corner_radius)
            .show(ui, |ui| {
                Label::new(number).selectable(false).ui(ui);
            })
            .response
    }
}

impl ShortcutCard {
    pub fn new(shortcut: KeyboardShortcut) -> Self {
        Self {
            shortcut,
            text_size: 20.,
            margin: Margin::same(5),
            text_color: None,
            fill_color: None,
        }
    }

    pub fn text_size(mut self, text_size: f32) -> Self {
        self.text_size = text_size;
        self
    }

    pub fn text_color(mut self, text_color: Color32) -> Self {
        self.text_color = Some(text_color);
        self
    }

    pub fn fill_color(mut self, fill_color: Color32) -> Self {
        self.fill_color = Some(fill_color);
        self
    }

    pub fn margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }
}
