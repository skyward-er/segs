use std::{collections::BTreeMap, fs, path::PathBuf, str::FromStr};

use egui::{
    Button, Color32, Context, InnerResponse, RichText, Separator, Stroke, TextEdit, Ui, Vec2,
    Widget,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use egui_file::FileDialog;

use super::{composable_view::ComposableViewState, ComposableView};

static LAYOUTS_DIR: &str = "layouts";
static SELECTED_LAYOUT_KEY: &str = "selected_layout";

#[derive(Default)]
pub struct LayoutManager {
    open: bool,

    /// Currently dislayed layout in the ui
    displayed: Option<PathBuf>,

    /// Currently selected layout in the list, gets reset to the displayed layout when the dialog is opened
    selection: Option<PathBuf>,

    text_input: String,
    file_dialog: Option<FileDialog>,
    layouts: BTreeMap<PathBuf, ComposableViewState>,
    layouts_path: PathBuf,
}

impl LayoutManager {
    /// Chooses the layouts path and gets the previously selected layout from storage
    pub fn new(app_name: &str, storage: &dyn eframe::Storage) -> Self {
        let mut layout_manager = Self {
            layouts_path: eframe::storage_dir(app_name).unwrap().join(LAYOUTS_DIR),
            selection: storage
                .get_string(SELECTED_LAYOUT_KEY)
                .map(|path| PathBuf::from_str(path.as_str()).unwrap()),
            ..Self::default()
        };
        layout_manager.reload_layouts();
        layout_manager
    }

    /// Saves in permanent storage the file name of the currently displayed layout
    pub fn save_displayed(&self, storage: &mut dyn eframe::Storage) {
        if let Some(displayed) = self.displayed.as_ref().map(|s| s.to_str()).flatten() {
            storage.set_string(SELECTED_LAYOUT_KEY, displayed.to_string());
            println!("Layout \"{}\" will be displayed next time", displayed)
        }
    }

    /// Scans the layout directory and reloads the layouts
    pub fn reload_layouts(&mut self) {
        if let Ok(files) = self.layouts_path.read_dir() {
            self.layouts = files
                .flat_map(|x| x)
                .map(|path| path.path())
                .map(|path| {
                    if let Some(layout) = ComposableViewState::from_file(&path) {
                        let path: PathBuf = path.file_stem().unwrap().into();
                        Some((path, layout))
                    } else {
                        None
                    }
                })
                .flatten()
                .collect();
        }
    }

    pub fn get_selected(&self) -> Option<&ComposableViewState> {
        self.selection
            .as_ref()
            .map(|selection| self.layouts.get(selection))
            .flatten()
    }

    pub fn delete(&mut self, key: &PathBuf) {
        if self.layouts.contains_key(key) {
            let _ = fs::remove_file(self.layouts_path.join(key).with_extension("json"));
        }
    }

    pub fn try_display_selected_layout(cv: &mut ComposableView) {
        if let Some(selection) = cv.layout_manager.selection.as_ref() {
            if let Some(selected_layout) = cv.layout_manager.layouts.get(selection) {
                cv.state = selected_layout.clone();
                cv.layout_manager.displayed = Some(selection.clone());
            }
        }
    }

    pub fn save_current_layout(cv: &mut ComposableView, name: &String) {
        let layouts_path = &cv.layout_manager.layouts_path;
        let path = layouts_path.join(name).with_extension("json");
        cv.state.to_file(&path);
        cv.layout_manager.selection.replace(name.into());
        cv.layout_manager.displayed.replace(name.into());
    }

    pub fn toggle_open_state(&mut self) {
        self.open = !self.open;

        if self.open {
            // When opening, we set the selection to the current layout
            self.selection = self.displayed.clone();
        } else {
            // When closing, we delete also the file dialog
            self.file_dialog.take();
        }
    }

    pub fn show(cv: &mut ComposableView, ctx: &Context) {
        let mut window_visible = cv.layout_manager.open;
        egui::Window::new("Layouts Manager")
            .collapsible(false)
            .open(&mut window_visible)
            .show(ctx, |ui| {
                // Make sure to reload the layots, this ways the user sees always
                // the current content of the layouts folder
                cv.layout_manager.reload_layouts();

                let changed = match cv.layout_manager.get_selected() {
                    Some(selected_layout) => *selected_layout != cv.state,
                    None => true,
                };

                // Layouts table
                StripBuilder::new(ui)
                    .size(Size::remainder().at_least(100.0))
                    .size(Size::exact(7.0))
                    .size(Size::exact(40.0))
                    .vertical(|mut strip| {
                        strip.cell(|ui| LayoutManager::show_layouts_table(ui, cv, changed));
                        strip.cell(|ui| {
                            ui.add(Separator::default().spacing(7.0));
                        });
                        strip.strip(|builder| {
                            LayoutManager::show_action_buttons(builder, cv, changed)
                        });
                    });
            });
        cv.layout_manager.open = window_visible;
    }

    fn show_layouts_table(ui: &mut Ui, cv: &mut ComposableView, changed: bool) {
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

                for key in cv.layout_manager.layouts.keys() {
                    let name = key.to_str().unwrap();
                    let is_selected = cv
                        .layout_manager
                        .selection
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
                    cv.layout_manager.selection.replace(to_select);
                }
                if let Some(to_open) = to_open {
                    cv.layout_manager.selection.replace(to_open);
                    LayoutManager::try_display_selected_layout(cv);
                }
                if let Some(to_delete) = to_delete {
                    cv.layout_manager.delete(&to_delete);
                }
            });
    }

    fn show_action_buttons(builder: StripBuilder, cv: &mut ComposableView, changed: bool) {
        builder.sizes(Size::remainder(), 2).horizontal(|mut strip| {
            // Load empty and import buttons
            strip.cell(|ui| {
                // Load empty button
                let open_empty_resp = ui.add_sized(
                    Vec2::new(ui.available_width(), 0.0),
                    Button::new("Load empty"),
                );
                if open_empty_resp.clicked() {
                    cv.state = ComposableViewState::default();
                    cv.layout_manager.selection.take();
                }

                // Import button
                let import_layout_resp =
                    ui.add_sized(Vec2::new(ui.available_width(), 0.0), Button::new("Import"));
                if import_layout_resp.clicked() {
                    let mut file_dialog = FileDialog::open_file(None);
                    file_dialog.open();
                    cv.layout_manager.file_dialog = Some(file_dialog);
                }
                if let Some(file_dialog) = &mut cv.layout_manager.file_dialog {
                    if file_dialog.show(ui.ctx()).selected() {
                        if let Some(file) = file_dialog.path() {
                            println!("Selected layout to import: {:?}", file);

                            let file_name = file.file_name().unwrap();
                            let destination = cv.layout_manager.layouts_path.join(file_name);

                            // First check if the layouts folder exists
                            if !cv.layout_manager.layouts_path.exists() {
                                match fs::create_dir_all(&cv.layout_manager.layouts_path) {
                                    Ok(_) => println!("Created layouts folder"),
                                    Err(e) => {
                                        println!("Error creating layouts folder: {:?}", e)
                                    }
                                }
                            }

                            match fs::copy(file, destination.clone()) {
                                Ok(_) => {
                                    println!(
                                        "Layout imported in {}",
                                        destination.to_str().unwrap()
                                    );
                                    cv.layout_manager.selection.replace(file_name.into());
                                    cv.layout_manager.reload_layouts();
                                    LayoutManager::try_display_selected_layout(cv);
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
                        TextEdit::singleline(&mut cv.layout_manager.text_input),
                    );

                    // Save button
                    let InnerResponse {
                        inner: save_button_resp,
                        ..
                    } = ui.add_enabled_ui(!cv.layout_manager.text_input.is_empty(), |ui| {
                        ui.add_sized(
                            Vec2::new(ui.available_width(), 0.0),
                            Button::new("Save layout"),
                        )
                    });

                    let to_save = text_edit_resp.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let to_save = to_save || save_button_resp.clicked();
                    to_save
                });

                if to_save {
                    let name = cv.layout_manager.text_input.clone();
                    LayoutManager::save_current_layout(cv, &name);
                }
            });
        });
    }
}
