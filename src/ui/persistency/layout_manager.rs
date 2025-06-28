//! Manages storing and loading of layouts on permanent storage
//!
//! # Quick explanation
//!
//! For layout we mean the part of the app's runtime state that we want store across executions.
//! This includes the layout of panes, each pane's state and other components included in the
//! [super::super::app::AppState] struct.
//!
//! # Details
//!
//! The goal of SEGS is to be a run-time customizable mission control software. The application
//! allows to customize the interface with many different **panes** (i.e. widgets) and to compose
//! such panes with splitters, tabs and windows. For the user to take advantage of this
//! customizability, there must be a way to save layouts persistently, and to load them at runtime
//! with minimal effort.

use super::super::app::AppState;
use crate::{APP_NAME, error::ErrInstrument};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use tracing::{info, trace, warn};

static LAYOUTS_DIR: &str = "layouts";
static SELECTED_LAYOUT_KEY: &str = "selected_layout";

#[derive(Default)]
pub struct LayoutManager {
    layouts_path: PathBuf, // Path to the layout directory on the system
    layouts: BTreeMap<PathBuf, AppState>,
    selected: Option<PathBuf>,
    current: AppState,
}

impl LayoutManager {
    pub fn from_storage(storage: &dyn eframe::Storage) -> Self {
        let selected = storage
            .get_string(SELECTED_LAYOUT_KEY)
            .map(|path| PathBuf::from_str(&path).log_expect("Path is not valid"));

        let mut layout_manager = Self {
            layouts_path: eframe::storage_dir(APP_NAME)
                .log_expect("Unable to get storage dir")
                .join(LAYOUTS_DIR),
            selected: selected.clone(),
            ..Self::default()
        };
        layout_manager.reload_layouts();
        layout_manager.current = selected
            .as_ref()
            .and_then(|selected| layout_manager.layouts.get(selected).cloned())
            .unwrap_or_default();

        println!("LayoutManager loaded. Selected layout is {:#?}", selected);
        layout_manager
    }

    pub fn selected(&self) -> Option<&PathBuf> {
        self.selected.as_ref()
    }

    pub fn current_mut(&mut self) -> &mut AppState {
        &mut self.current
    }

    /// TODO: This should check if the current is not saved
    pub fn load_default(&mut self) {
        self.current = AppState::default()
    }

    /// TODO: This should check if the current is not saved
    #[profiling::function]
    pub fn load_layout(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let layout = self
            .layouts
            .get(path.as_ref())
            .ok_or(anyhow::anyhow!("Layout not found"))?;
        self.current = layout.clone();
        self.selected = Some(path.as_ref().into());
        Ok(())
    }

    pub fn layouts(&self) -> Vec<&PathBuf> {
        self.layouts.keys().collect()
    }

    pub fn layouts_path(&self) -> &PathBuf {
        &self.layouts_path
    }

    /// Scans the layout directory and reloads the layouts
    #[profiling::function]
    pub fn reload_layouts(&mut self) {
        if let Ok(files) = self.layouts_path.read_dir() {
            trace!("Reloading layouts from {:?}", self.layouts_path);
            self.layouts = files
                .flatten()
                .map(|path| path.path())
                .flat_map(|path| match AppState::from_file(&path) {
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

    #[profiling::function]
    pub fn save_current(&self) -> anyhow::Result<()> {
        let current_layout = self
            .selected
            .as_ref()
            .ok_or(anyhow::anyhow!("No selected layout to save onto"))?;
        self.current.to_file(current_layout)
    }

    pub fn is_saved(&self) -> bool {
        self.selected
            .as_ref()
            .and_then(|current_layout| self.layouts.get(current_layout))
            .is_some_and(|saved_layout| saved_layout == &self.current)
    }

    /// TODO: Should it check if we are deliting the selected layout?
    pub fn delete(&mut self, key: &PathBuf) -> anyhow::Result<()> {
        if self.layouts.contains_key(key) {
            info!("Deleting layout {:?}", key);
            fs::remove_file(self.layouts_path.join(key).with_extension("json"))?;
        }
        Ok(())
    }
}
