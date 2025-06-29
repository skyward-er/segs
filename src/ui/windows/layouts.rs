use egui::{
    Button, Color32, Context, InnerResponse, RichText, Separator, Stroke, TextEdit, Ui, Vec2,
    Widget,
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
                        strip.cell(|ui| show_layouts_table(ui, layout_manager, is_saved));
                        strip.cell(|ui| {
                            ui.add(Separator::default().spacing(7.0));
                        });
                        strip.strip(|builder| {
                            show_action_buttons(
                                builder,
                                layout_manager,
                                file_dialog,
                                text_input,
                                is_saved,
                            )
                        });
                    });
            });
    }
}

fn show_layouts_table(ui: &mut Ui, layout_manager: &mut LayoutManager, is_saved: bool) {
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

            for store in layout_manager.stores() {
                for layout_store_key in store.1.layouts() {
                    let layout_key = (store.0.clone(), layout_store_key.clone());
                    let is_selected = layout_manager
                        .selected()
                        .map_or(false, |selected| selected == &layout_key);

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
                    let open_button = Button::new("↗");
                    let delete_button = Button::new("🗑");

                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            let name_button_resp = name_button.ui(ui);
                            if name_button_resp.clicked() {
                                to_load = Some(layout_key.clone());
                            }
                        });
                        row.col(|ui| {
                            if open_button.ui(ui).clicked() {
                                to_load = Some(layout_key.clone());
                            }
                        });
                        row.col(|ui| {
                            if delete_button.ui(ui).clicked() {
                                to_delete = Some(layout_key.clone());
                            }
                        });
                    });
                }
            }

            if let Some(to_select) = to_load {
                // FIXME: when error dialog will be implemented this will be changed
                if let Err(e) = layout_manager.load_layout(&to_select) {
                    error!("Error opening layout {:?}: {:?}", to_select, e);
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
    builder.sizes(Size::remainder(), 2).horizontal(|mut strip| {
        // Load empty and import buttons
        strip.cell(|ui| {
            // Load empty button
            let open_empty_resp = ui.add_sized(
                Vec2::new(ui.available_width(), 0.0),
                Button::new("Load empty"),
            );
            if open_empty_resp.clicked() {
                layout_manager.load_default();
            }

            // Import button
            // let import_layout_resp =
            //     ui.add_sized(Vec2::new(ui.available_width(), 0.0), Button::new("Import"));
            // if import_layout_resp.clicked() {
            //     let mut file_dialog_inner = FileDialog::open_file(None);
            //     file_dialog_inner.open();
            //     *file_dialog = Some(file_dialog_inner);
            // }
            // if let Some(file_dialog) = file_dialog {
            //     if file_dialog.show(ui.ctx()).selected() {
            //         if let Some(file) = file_dialog.path() {
            //             debug!("Selected layout to import: {:?}", file);

            //             let file_name: &std::ffi::OsStr =
            //                 file.file_name().log_expect("Unable to get file name");
            //             let layout_path = layout_manager.layouts_path();
            //             let destination = layout_path.join(file_name);

            //             // First check if the layouts folder exists
            //             if !layout_path.exists() {
            //                 fs::create_dir_all(layout_manager.layouts_path())
            //                     .log_expect("Unable to create layouts folder");
            //                 debug!("Created layouts folder");
            //             }

            //             if let Err(e) = fs::copy(file, destination.clone()) {
            //                 // FIXME when error dialog will be implemented this will be changed
            //                 error!("Error importing layout: {:?}", e);
            //             }

            //             debug!("Layout imported in {}", destination.to_str().log_unwrap());
            //             selection.replace(file_name.into());
            //             layout_manager.reload_layouts();
            //             if let Err(e) = layout_manager.load_layout(file_name) {
            //                 // FIXME when error dialog will be implemented this will be changed
            //                 error!("Error loading imported layout: {:?}", e);
            //             }
            //         }
            //     }
            // }
        });
        // Layout save ui
        strip.cell(|ui| {
            let InnerResponse { inner: to_save, .. } = ui.add_enabled_ui(!is_saved, |ui| {
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

                to_save || save_button_resp.clicked()
            });

            if to_save {
                println!("Saving layout!!!!");
                let name = text_input.clone();
                if let Err(e) = layout_manager.save_new(&("local".to_string(), name.clone())) {
                    // FIXME when error dialog will be implemented this will be changed
                    error!("Error saving layout: {:?}", e);
                    panic!("Error saving layout: {:?}", e);
                }
            }
        });
    });
}
