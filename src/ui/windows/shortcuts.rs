use egui::{Align2, RichText};

#[derive(Default)]
pub struct ShortcutsWindow {
    pub visible: bool,
}

impl ShortcutsWindow {
    pub fn show(&mut self, ctx: &egui::Context) {
        let mut visible = self.visible;
        egui::Window::new("Shortcuts")
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .open(&mut visible)
            .show(ctx, |ui| {
                self.ui(ui);
            });
        self.visible = visible;
    }

    fn ui(&self, ui: &mut egui::Ui) {
        let pane_shortcuts: &[(&str, &str)] = &[
            ("Ctrl + H", "Split the hovered pane horizontally"),
            ("Ctrl + V", "Split the hovered pane vertically"),
            ("Ctrl + W", "Close the hovered pane"),
            ("Ctrl + R", "Replace the hovered pane via the widget gallery"),
            ("Shift + Esc", "Maximize the hovered pane"),
            ("Esc", "Exit the maximized pane"),
        ];

        section(ui, "Panes", pane_shortcuts);

        #[cfg(feature = "conrig")]
        {
            ui.add_space(6.0);
            section(
                ui,
                "Commands",
                &[
                    ("/", "Open / close the command switch"),
                    ("<command key>", "Trigger a command shown in the command switch"),
                ],
            );
        }
    }
}

fn section(ui: &mut egui::Ui, title: &str, entries: &[(&str, &str)]) {
    ui.label(RichText::new(title).strong());
    egui::Grid::new(ui.id().with(title))
        .num_columns(2)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            for (keys, description) in entries {
                ui.label(RichText::new(*keys).monospace());
                ui.label(*description);
                ui.end_row();
            }
        });
}
