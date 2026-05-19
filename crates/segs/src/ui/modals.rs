use egui::{Id, Ui};
use segs_assets::icons;
use segs_ui::widgets::UiWidgetExt;

/// A simple reusable modal overlay component.
pub struct Modal<'a> {
    enabled: &'a mut bool,
    id: Option<Id>,
    title: Option<String>,
}

impl<'a> Modal<'a> {
    pub fn new(enabled: &'a mut bool) -> Self {
        Self {
            enabled,
            title: None,
            id: None,
        }
    }

    pub fn with_id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Show the modal. `add_contents` draws the user-provided content inside the modal body.
    pub fn show(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
        if !*self.enabled {
            return;
        }

        let id = self.id.unwrap_or_else(|| ui.id().with("_modal"));
        egui::Modal::new(id).show(ui.ctx(), |ui| {
            // A grid with 2 columns: Column 1 (Title/Content), Column 2 (Close button)
            egui::Grid::new(id.with("_grid"))
                .num_columns(2)
                .min_col_width(80.)
                .spacing([0.0, 5.0]) // Tightly packed horizontally
                .show(ui, |ui| {
                    // --- ROW 1: HEADER ---
                    let title = if let Some(title) = &self.title { title } else { "" };
                    // Define a frame with only left padding/margin
                    egui::Frame::new()
                        .inner_margin(egui::Margin {
                            left: 2, // Adjust this value to perfectly mirror the X button
                            right: 0,
                            top: 1,
                            bottom: 0,
                        })
                        .show(ui, |ui| {
                            ui.add(egui::Label::new(title).selectable(false));
                        });

                    // Align the X button to the right side of the second column
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let close_res = ui.icon_btn(icons::X::default());
                        if close_res.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            *self.enabled = false;
                        }
                    });
                    ui.end_row();

                    // --- ROW 2: CONTENT ---
                    add_contents(ui);
                    ui.end_row();
                });
        });
    }
}
