use egui::{
    Button, Color32, Context, RichText, Separator, Stroke, TextEdit, Ui, UiBuilder, Vec2, Widget,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use egui_file::FileDialog;
use tracing::error;

use crate::ui::persistency::layout_manager::{LayoutManager, LayoutRef};

#[derive(Default)]
pub struct LayoutManagerWindow {
    visible: bool,
    file_dialog: Option<FileDialog>,
    text_input: String,
}

impl LayoutManagerWindow {
    pub fn toggle_open_state(&mut self) {
        self.visible = !self.visible;
        if !self.visible {
            self.file_dialog.take();
        }
    }

    /// Currently selected layout in the list, gets reset to the displayed layout when the dialog is opened
    #[profiling::function]
    pub fn show(&mut self, ctx: &Context, layout_manager: &mut LayoutManager) {
        let LayoutManagerWindow {
            visible: window_visible,
            file_dialog,
            text_input,
        } = self;
        if !*window_visible {
            text_input.clear();
        }
        egui::Window::new("Layouts Manager")
            .collapsible(false)
            .open(window_visible)
            .show(ctx, |ui| {
                let is_saved = layout_manager.is_saved().unwrap_or(false);

                // TODO: Add a reload/sync button to pull the layouts
                StripBuilder::new(ui)
                    .size(Size::initial(200.0))
                    .size(Size::exact(7.0))
                    .size(Size::exact(40.0))
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            show_layouts_table(
                                ui,
                                layout_manager,
                                state,
                                selection,
                                text_input,
                                changed,
                            )
                        });
                        strip.cell(|ui| {
                            ui.add(Separator::default().spacing(7.0));
                        });
                        strip.cell(|ui| {
                            ui.horizontal(|ui| {
                                ui.add(Button::new("Load empty"));
                                ui.add(
                                    TextEdit::singleline(text_input).desired_width(f32::INFINITY),
                                );
                                ui.add(Button::new("Save new"));
                                let save_button = Button::new("Save new");
                            });
                        });
                        // strip.strip(|builder| {
                        //     show_action_buttons(
                        //         builder,
                        //         layout_manager,
                        //         file_dialog,
                        //         text_input,
                        //         is_saved,
                        //     )
                        // });
                    });
            });
    }
}

fn show_layouts_table(
    ui: &mut Ui,
    layout_manager: &mut LayoutManager,
    selection: &mut Option<PathBuf>,
    text_input: &mut String,
    changed: bool,
) {
    let available_height = ui.available_height();

    TableBuilder::new(ui)
        .column(Column::remainder())
        .column(Column::auto())
        .column(Column::auto())
        .min_scrolled_height(0.0)
        .max_scroll_height(available_height)
        .body(|mut body| {
            let mut to_load: Option<LayoutRef> = None;
            let mut to_delete: Option<LayoutRef> = None;

            for key in layout_manager.layouts() {
                let name = key.to_str().log_expect("Unable to convert path to string");
                let is_selected = selection.as_ref().is_some_and(|s| s == key);

                    let name_button = if is_selected && !is_saved {
                        Button::new(RichText::new(layout_store_key).color(Color32::BLACK))
                            .stroke(Stroke::new(1.0, Color32::BROWN))
                            .fill(Color32::YELLOW)
                    } else if is_selected && is_saved {
                        Button::new(RichText::new(layout_store_key).color(Color32::BLACK))
                            .stroke(Stroke::new(1.0, Color32::GREEN))
                            .fill(Color32::LIGHT_GREEN)
                    } else {
                        Button::new(layout_store_key).fill(Color32::TRANSPARENT)
                    };

                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            let name_button_resp = name_button.ui(ui);
                            if name_button_resp.clicked() {
                                to_load = Some(layout_key.clone());
                            }
                        });
                        row.col(|ui| {
                            ui.scope_builder(
                                UiBuilder {
                                    invisible: !is_selected,
                                    disabled: is_saved,
                                    ..Default::default()
                                },
                                |ui| {
                                    if Button::new("Save").ui(ui).clicked() {
                                        to_load = Some(layout_key.clone());
                                    }
                                },
                            );
                        });
                        row.col(|ui| {
                            if Button::new("🗑").ui(ui).clicked() {
                                to_delete = Some(layout_key.clone());
                            }
                        });
                    });
                }
            }

            if let Some(to_select) = to_select {
                *text_input = to_select
                    .to_str()
                    .log_expect("Unable to convert path to string")
                    .to_string();
                selection.replace(to_select);
            }
            if let Some(to_open) = to_open {
                // FIXME when error dialog will be implemented this will be changed
                if layout_manager.load_layout(&to_open).is_ok() {
                    selection.replace(to_open.clone());
                } else {
                    error!("Error opening layout: {:?}", to_open);
                }
            }
            if let Some(to_delete) = to_delete {
                // FIXME: when error dialog will be implemented this will be changed
                if let Err(e) = layout_manager.delete(&to_delete) {
                    error!("Error deleting layout: {:?}", e);
                }
            }
        });
}

fn show_action_buttons(
    builder: StripBuilder,
    layout_manager: &mut LayoutManager,
    file_dialog: &mut Option<FileDialog>,
    text_input: &mut String,
    is_saved: bool,
) {
    builder
        .size(Size::initial(20.0))
        .size(Size::remainder())
        .size(Size::initial(20.0))
        .horizontal(|mut strip| {
            // Load empty and import buttons
            strip.cell(|ui| {
                // Load empty button
                if Button::new("Load empty")
                    .min_size(Vec2::new(80.0, 0.0))
                    .ui(ui)
                    .clicked()
                {
                    layout_manager.load_default();
                }
            });
            strip.cell(|ui| {
                TextEdit::singleline(text_input).ui(ui);
            });
            strip.cell(|ui| {
                if ui
                    .add_enabled(!text_input.is_empty(), Button::new("Save new"))
                    .clicked()
                {
                    layout_manager.load_default();
                }
            });
        });
}
