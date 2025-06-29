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
use crate::ui::persistency::layout_store::{LayoutLocalStore, LayoutStore, LayoutStoreKey};
use anyhow::{Result, anyhow};
use egui::ahash::HashMap;

use crate::error::ErrInstrument;

use super::super::app::AppState;

static LAYOUTS_DIR: &str = "layouts";
static SELECTED_LAYOUT_KEY: &str = "selected_layout";

#[derive(Default)]
pub struct LayoutManager {
    stores: HashMap<String, Box<dyn LayoutStore>>,
    selected: Option<LayoutRef>, // (store id, layout id)
    current: AppState,
}

impl LayoutManager {
    pub fn new(layouts_path: PathBuf) -> Self {
        Self {
            layouts_path,
            ..Self::default()
        }
    }
    /// Chooses the layouts path and gets the previously selected layout from storage
    pub fn set_current_layout_with_storage(&mut self, storage: &dyn eframe::Storage) {
        self.current_layout = storage
            .get_string(SELECTED_LAYOUT_KEY)
            .map(|path| PathBuf::from_str(&path).log_expect("Path is not valid"));
        self.reload_layouts();
    }

    pub fn selected(&self) -> Option<&LayoutRef> {
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
    pub fn load_layout(&mut self, key: &LayoutRef) -> Result<()> {
        let store = self.stores.get(&key.0).ok_or(anyhow!("Store not found"))?;
        self.current = store.get(&key.1)?.clone();
        self.selected = Some(key.clone());
        Ok(())
    }

    pub fn stores(&self) -> &HashMap<String, Box<dyn LayoutStore>> {
        &self.stores
    }

    pub fn get_store_mut(&mut self, store_name: &String) -> Result<&mut Box<dyn LayoutStore>> {
        self.stores
            .get_mut(store_name)
            .ok_or(anyhow!("Store not found"))
    }

    #[profiling::function]
    pub fn pull_all(&mut self) {
        self.stores.iter_mut().for_each(|store| {
            // TODO: How do we handle failures here?
            store.1.pull_all();
        });
    }

    /// Saves the current layout to the selected ref regardless if the store has the ref or not
    #[profiling::function]
    pub fn save(&mut self) -> Result<()> {
        let selected = self.selected.clone().ok_or(anyhow!("No layout selected"))?;
        let current = self.current.clone();
        let store = self.get_store_mut(&selected.0)?;
        store.push(&selected.1, &current)?;
        Ok(())
    }

    /// Saves the current layout to the given ref and makes it the selected on, regardless if the store has the ref or not
    pub fn save_new(&mut self, key: &LayoutRef) -> Result<()> {
        let current = self.current.clone();
        let store = self.get_store_mut(&key.0)?;
        store.push(&key.1, &current)?;
        self.selected = Some(key.clone());
        Ok(())
    }

    /// Checks if the current layout matches the local copy of the selected one
    pub fn is_saved(&self) -> Result<bool> {
        // If the current layout is still the default one, and there is no layout selected, we consider it saved
        if self.current == AppState::default() && self.selected.is_none() {
            return Ok(true);
        }
        let selected = self.selected.clone().ok_or(anyhow!("No layout selected"))?;
        let store = self
            .stores
            .get(&selected.0)
            .ok_or(anyhow!("Store not found"))?;
        Ok(store.get(&selected.1)? == &self.current)
    }

    /// Deletes the given layout
    pub fn delete(&mut self, key: &LayoutRef) -> Result<()> {
        self.selected.take_if(|selected| selected == key);
        let store = self.get_store_mut(&key.0)?;
        store.delete(&key.1)?;
        Ok(())
    }
}
