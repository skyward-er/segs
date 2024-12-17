use std::{fs, path::PathBuf};

use egui::{
    Button, Color32, Context, InnerResponse, RichText, Separator, Stroke, TextEdit, Ui, Vec2,
    Widget,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use egui_file::FileDialog;

use crate::ui::composable_view::ComposableViewState;

use super::LayoutManager;

#[derive(Default)]
pub struct LayoutManagerWindow {
    visible: bool,
    file_dialog: Option<FileDialog>,
    text_input: String,
    /// Currently selected layout in the list, gets reset to the displayed layout when the dialog is opened
    pub selection: Option<PathBuf>,
}

impl LayoutManagerWindow {
    pub fn toggle_open_state(&mut self, layout_manager: &LayoutManager) {
        self.visible = !self.visible;

        if self.visible {
            // When opening, we set the selection to the current layout
            self.selection = layout_manager.current_layout().cloned();
        } else {
            // When closing, we delete also the file dialog
            self.file_dialog.take();
        }
    }

    pub fn show(
        &mut self,
        ctx: &Context,
        layout_manager: &mut LayoutManager,
        state: &mut ComposableViewState,
    ) {
        let LayoutManagerWindow {
            visible: window_visible,
            file_dialog,
            text_input,
            selection,
        } = self;
        egui::Window::new("Layouts Manager")
            .collapsible(false)
            .open(window_visible)
            .show(ctx, |ui| {
                // Make sure to reload the layots, this ways the user sees always
                // the current content of the layouts folder
                layout_manager.reload_layouts();

                let changed = selection
                    .as_ref()
                    .and_then(|path| layout_manager.get_layout(path))
                    .map(|layout| layout != state)
                    .unwrap_or(true);

                // Layouts table
                StripBuilder::new(ui)
                    .size(Size::remainder().at_least(100.0))
                    .size(Size::exact(7.0))
                    .size(Size::exact(40.0))
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            show_layouts_table(ui, layout_manager, state, selection, changed)
                        });
                        strip.cell(|ui| {
                            ui.add(Separator::default().spacing(7.0));
                        });
                        strip.strip(|builder| {
                            show_action_buttons(
                                builder,
                                layout_manager,
                                state,
                                file_dialog,
                                text_input,
                                selection,
                                changed,
                            )
                        });
                    });
            });
    }
}

fn show_layouts_table(
    ui: &mut Ui,
    layout_manager: &mut LayoutManager,
    state: &mut ComposableViewState,
    selection: &mut Option<PathBuf>,
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
            let mut to_select: Option<PathBuf> = None;
            let mut to_open: Option<PathBuf> = None;
            let mut to_delete: Option<PathBuf> = None;

            for key in layout_manager.layouts().keys() {
                let name = key.to_str().unwrap();
                let is_selected = selection
                    .as_ref()
                    .map_or_else(|| false, |selected_key| selected_key == key);

                let name_button = if is_selected && changed {
                    Button::new(RichText::new(name).color(Color32::BLACK))
                        .stroke(Stroke::new(1.0, Color32::BROWN))
                        .fill(Color32::YELLOW)
                } else if is_selected && !changed {
                    Button::new(RichText::new(name).color(Color32::BLACK))
                        .stroke(Stroke::new(1.0, Color32::GREEN))
                        .fill(Color32::LIGHT_GREEN)
                } else {
                    Button::new(name).fill(Color32::TRANSPARENT)
                };
                let open_button = Button::new("↗");
                let delete_button = Button::new("🗑");

                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        let name_button_resp = name_button.ui(ui);
                        if name_button_resp.clicked() {
                            to_select = Some(key.clone());
                        }
                        if name_button_resp.double_clicked() {
                            to_open = Some(key.clone());
                        }
                    });
                    row.col(|ui| {
                        if open_button.ui(ui).clicked() {
                            to_open = Some(key.clone());
                        }
                    });
                    row.col(|ui| {
                        if delete_button.ui(ui).clicked() {
                            to_delete = Some(key.clone());
                        }
                    });
                });
            }

            if let Some(to_select) = to_select {
                selection.replace(to_select);
            }
            if let Some(to_open) = to_open {
                layout_manager.load_layout(&to_open, state);
                selection.replace(to_open.clone());
            }
            if let Some(to_delete) = to_delete {
                layout_manager.delete(&to_delete);
            }
        });
}

fn show_action_buttons(
    builder: StripBuilder,
    layout_manager: &mut LayoutManager,
    state: &mut ComposableViewState,
    file_dialog: &mut Option<FileDialog>,
    text_input: &mut String,
    selection: &mut Option<PathBuf>,
    changed: bool,
) {
    builder.sizes(Size::remainder(), 2).horizontal(|mut strip| {
        // Load empty and import buttons
        strip.cell(|ui| {
            // Load empty button
            let open_empty_resp = ui.add_sized(
                Vec2::new(ui.available_width(), 0.0),
                Button::new("Load empty"),
            );
            if open_empty_resp.clicked() {
                *state = ComposableViewState::default();
                selection.take();
            }

            // Import button
            let import_layout_resp =
                ui.add_sized(Vec2::new(ui.available_width(), 0.0), Button::new("Import"));
            if import_layout_resp.clicked() {
                let mut file_dialog_inner = FileDialog::open_file(None);
                file_dialog_inner.open();
                *file_dialog = Some(file_dialog_inner);
            }
            if let Some(file_dialog) = file_dialog {
                if file_dialog.show(ui.ctx()).selected() {
                    if let Some(file) = file_dialog.path() {
                        println!("Selected layout to import: {:?}", file);

                        let file_name: &std::ffi::OsStr = file.file_name().unwrap();
                        let layout_path = layout_manager.layouts_path();
                        let destination = layout_path.join(file_name);

                        // First check if the layouts folder exists
                        if !layout_path.exists() {
                            match fs::create_dir_all(&layout_manager.layouts_path()) {
                                Ok(_) => println!("Created layouts folder"),
                                Err(e) => {
                                    println!("Error creating layouts folder: {:?}", e)
                                }
                            }
                        }

                        match fs::copy(file, destination.clone()) {
                            Ok(_) => {
                                println!("Layout imported in {}", destination.to_str().unwrap());
                                selection.replace(file_name.into());
                                layout_manager.reload_layouts();
                                layout_manager.load_layout(&file_name, state);
                            }
                            Err(e) => println!("Error importing layout: {:?}", e),
                        }
                    }
                }
            }
        });
        // Layout save ui
        strip.cell(|ui| {
            let InnerResponse { inner: to_save, .. } = ui.add_enabled_ui(changed, |ui| {
                // Text edit
                let text_edit_resp = ui.add_sized(
                    Vec2::new(ui.available_width(), 0.0),
                    TextEdit::singleline(text_input),
                );

                // Save button
                let InnerResponse {
                    inner: save_button_resp,
                    ..
                } = ui.add_enabled_ui(!text_input.is_empty(), |ui| {
                    ui.add_sized(
                        Vec2::new(ui.available_width(), 0.0),
                        Button::new("Save layout"),
                    )
                });

                let to_save =
                    text_edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                let to_save = to_save || save_button_resp.clicked();
                to_save
            });

            if to_save {
                let name = text_input.clone();
                layout_manager.save_layout(&name, &state);
                *selection = Some(name.clone().into());
            }
        });
    });
}
