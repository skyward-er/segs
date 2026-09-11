use egui::{KeyboardShortcut, Label, Response, RichText, Tooltip as EguiTooltip};

const SHORTCUT_SPACING: f32 = 16.;

/// Shows standard non-interactive tooltip content with an optional keyboard shortcut.
pub struct Tooltip<'a> {
    response: &'a Response,
    text: String,
    shortcut: Option<KeyboardShortcut>,
}

impl<'a> Tooltip<'a> {
    /// Creates a tooltip for a widget response.
    ///
    /// The returned tooltip displays `text` when the response satisfies egui's hover timing.
    pub fn new(response: &'a Response, text: impl Into<String>) -> Self {
        Self {
            response,
            text: text.into(),
            shortcut: None,
        }
    }

    /// Adds a platform-formatted keyboard shortcut to the right side of the tooltip.
    ///
    /// The returned tooltip renders the shortcut using the current operating system's modifier names.
    pub fn shortcut(mut self, shortcut: KeyboardShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Displays the tooltip when its response is eligible to show one.
    pub fn show(self) {
        let Self {
            response,
            text,
            shortcut,
        } = self;

        // Select egui's matching hover behavior for the widget's enabled state
        let tooltip = if response.enabled() {
            EguiTooltip::for_enabled(response)
        } else {
            EguiTooltip::for_disabled(response)
        };

        // Render the label and optional shortcut as one non-interactive row
        tooltip.show(|ui| {
            ui.horizontal(|ui| {
                ui.add(Label::new(text).selectable(false));
                if let Some(shortcut) = shortcut {
                    ui.add_space((SHORTCUT_SPACING - ui.spacing().item_spacing.x).max(0.));
                    let shortcut_text = ui.ctx().format_shortcut(&shortcut);
                    ui.add(Label::new(RichText::new(shortcut_text).weak()).selectable(false));
                }
            });
        });
    }
}
