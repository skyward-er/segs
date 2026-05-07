use std::collections::HashSet;

use egui::{Response, ScrollArea, Sense, UiBuilder, Window};
use serde::{Deserialize, Serialize};

use crate::{
    mavlink::{
        TimedMessage,
        reflection::{IndexedField, all_fields},
    },
    ui::panes::{PaneBehavior, PaneResponse},
};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MessagesViewerPane {
    selected_fields: HashSet<usize>,

    #[serde(skip)]
    field_values: Vec<(String, String)>, // (name, formatted value)
    #[serde(skip)]
    settings_visible: bool,
    #[serde(skip)]
    search_text: String,
}

impl PaneBehavior for MessagesViewerPane {
    fn ui(&mut self, ui: &mut egui::Ui) -> PaneResponse {
        let mut pane_response = PaneResponse::default();

        if self.settings_visible {
            Window::new("Messages Viewer Settings")
                .id(ui.id().with("messages_viewer_settings"))
                .collapsible(false)
                .resizable(true)
                .open(&mut self.settings_visible)
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Search:");
                        ui.text_edit_singleline(&mut self.search_text);
                    });

                    let fields = all_fields();
                    let mut select_all = false;
                    let mut deselect_all = false;

                    if fields.len() > 1 {
                        ui.horizontal(|ui| {
                            select_all = ui.button("Select All").clicked();
                            deselect_all = ui.button("Deselect All").clicked();
                        });
                    }

                    if select_all {
                        for f in &fields {
                            self.selected_fields.insert(f.index());
                        }
                    }
                    if deselect_all {
                        self.selected_fields.clear();
                    }

                    ScrollArea::both()
                        .auto_shrink([false, true])
                        .max_width(350.0)
                        .max_height(400.0)
                        .show(ui, |ui| {
                            for field in &fields {
                                let lower = self.search_text.to_lowercase();
                                if !lower.is_empty()
                                    && !field.name().to_lowercase().contains(&lower)
                                {
                                    continue;
                                }
                                let mut selected = self.selected_fields.contains(&field.index());
                                let res: Response = ui.checkbox(&mut selected, field.name());
                                if res.clicked() {
                                    if selected {
                                        self.selected_fields.insert(field.index());
                                    } else {
                                        self.selected_fields.remove(&field.index());
                                    }
                                }
                            }
                        });
                });
        }

        let max_rect = ui.max_rect().shrink(8.);
        let res = ui.scope_builder(
            UiBuilder::new()
                .max_rect(max_rect)
                .sense(Sense::click_and_drag()),
            |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.scope_builder(UiBuilder::new().sense(Sense::click_and_drag()), |ui| {
                            egui::Grid::new("message_viewer").show(ui, |ui| {
                                for (name, value) in &self.field_values {
                                    ui.label(name);
                                    ui.label(value);
                                    ui.end_row();
                                }
                                if self.field_values.is_empty() && !self.selected_fields.is_empty()
                                {
                                    ui.label("Waiting for data…");
                                } else if self.selected_fields.is_empty() {
                                    ui.label("Right-click to open settings and select fields.");
                                }
                            });
                            ui.allocate_space(ui.available_size());
                        })
                        .response
                    })
                    .inner
            },
        );

        let res = res.inner.union(res.response);
        res.context_menu(|ui| {
            if ui.button("Open settings…").clicked() {
                self.settings_visible = true;
                ui.close_menu();
            }
        });

        if res.drag_started() {
            pane_response.set_drag_started();
        }

        pane_response
    }

    fn update(&mut self, message: Option<&TimedMessage>) {
        let Some(msg) = message else { return };
        if self.selected_fields.is_empty() {
            return;
        }

        let fields = all_fields();
        self.field_values = fields
            .iter()
            .filter(|f| self.selected_fields.contains(&f.index()))
            .map(|f: &IndexedField| (f.name().to_string(), f.extract_as_string(&msg.packet)))
            .collect();
    }
}
