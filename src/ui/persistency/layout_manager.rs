use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use tracing::{info, trace, warn};

use crate::error::ErrInstrument;

use super::super::composable_view::ComposableViewState;

static LAYOUTS_DIR: &str = "layouts";
static SELECTED_LAYOUT_KEY: &str = "selected_layout";

#[derive(Default)]
pub struct LayoutManager {
    layouts: BTreeMap<PathBuf, ComposableViewState>,
    layouts_path: PathBuf,
    current_layout: Option<PathBuf>,
}

impl LayoutManager {
    /// Chooses the layouts path and gets the previously selected layout from storage
    pub fn new(app_name: &str, storage: &dyn eframe::Storage) -> Self {
        let mut layout_manager = Self {
            layouts_path: eframe::storage_dir(app_name)
                .log_expect("Unable to get storage dir")
                .join(LAYOUTS_DIR),
            current_layout: storage
                .get_string(SELECTED_LAYOUT_KEY)
                .map(|path| PathBuf::from_str(&path).log_expect("Path is not valid")),
            ..Self::default()
        };
        layout_manager.reload_layouts();
        layout_manager
    }

    pub fn current_layout(&self) -> Option<&PathBuf> {
        self.current_layout.as_ref()
    }

    pub fn layouts_path(&self) -> &PathBuf {
        &self.layouts_path
    }

    pub fn layouts(&self) -> &BTreeMap<PathBuf, ComposableViewState> {
        &self.layouts
    }

    /// Saves in permanent storage the file name of the currently displayed layout
    pub fn save_current_layout(&self, storage: &mut dyn eframe::Storage) {
        if let Some(current_layout) = self.current_layout.as_ref().and_then(|s| s.to_str()) {
            storage.set_string(SELECTED_LAYOUT_KEY, current_layout.to_string());
            trace!("Current layout {:?} saved in storage", current_layout);
        }
    }

    /// Scans the layout directory and reloads the layouts
    #[profiling::function]
    pub fn reload_layouts(&mut self) {
        if let Ok(files) = self.layouts_path.read_dir() {
            trace!("Reloading layouts from {:?}", self.layouts_path);
            self.layouts = files
                .flatten()
                .map(|path| path.path())
                .flat_map(|path| match ComposableViewState::from_file(&path) {
                    Ok(layout) => {
                        let path: PathBuf = path
                            .file_stem()
                            .log_expect("Unable to get file stem")
                            .into();
                        Some((path, layout))
                    }
                    Err(e) => {
                        warn!("Error loading layout at {:?}: {:?}", path, e);
                        None
                    }
                })
                .collect();
        }
    }

    pub fn get_layout(&self, name: impl Into<PathBuf>) -> Option<&ComposableViewState> {
        self.layouts.get(&name.into())
    }

    #[profiling::function]
    pub fn save_layout(&mut self, name: &str, state: &ComposableViewState) -> anyhow::Result<()> {
        let path = self.layouts_path.join(name).with_extension("json");
        state.to_file(&path)?;
        self.reload_layouts();
        Ok(())
    }

    #[profiling::function]
    pub fn load_layout(
        &mut self,
        path: impl AsRef<Path>,
        state: &mut ComposableViewState,
    ) -> anyhow::Result<()> {
        let layout = self
            .layouts
            .get(path.as_ref())
            .ok_or(anyhow::anyhow!("Layout not found"))?;
        *state = layout.clone();
        self.current_layout = Some(path.as_ref().into());
        Ok(())
    }

    pub fn delete(&mut self, key: &PathBuf) -> anyhow::Result<()> {
        if self.layouts.contains_key(key) {
            info!("Deleting layout {:?}", key);
            fs::remove_file(self.layouts_path.join(key).with_extension("json"))?;
        }
        Ok(())
    }

    // pub fn display_selected_layout(&mut self, state: &mut ComposableViewState) {
    //     if let Some(selection) = self.selection.as_ref() {
    //         if let Some(selected_layout) = self.layouts.get(selection) {
    //             *state = selected_layout.clone();
    //             self.displayed = Some(selection.clone());
    //         }
    //     }
    // }

    // pub fn save_current_layout(&mut self, state: &ComposableViewState, name: &String) {
    //     let layouts_path = &self.layouts_path;
    //     let path = layouts_path.join(name).with_extension("json");
    //     state.to_file(&path);
    //     self.selection.replace(name.into());
    //     self.displayed.replace(name.into());
    // }
}
