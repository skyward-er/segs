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

mod storage_keys {
    pub(super) static LOCAL_STORE: &str = "local";
    pub(super) static SELECTED: &str = "selected_layout";
}

/// Reference to a layout in a specific store
pub type LayoutRef = (String, LayoutStoreKey);

#[derive(Default)]
pub struct LayoutManager {
    stores: HashMap<String, Box<dyn LayoutStore>>,
    selected: Option<LayoutRef>, // (store id, layout id)
    current: AppState,
}

impl LayoutManager {
    /// SEGS will store in [eframe::Storage] the list of configured stores and the id the last selected layout
    pub fn from_storage(storage: &dyn eframe::Storage) -> Self {
        let mut layout_manager = LayoutManager::default();

        println!("Creating layout manager from storage");

        // Try to load the local store. When this fails, add a local store with a default path
        let mut local_store =
            eframe::get_value::<LayoutLocalStore>(storage, storage_keys::LOCAL_STORE)
                .unwrap_or_default();
        if let Err(e) = local_store.pull_all() {
            println!("Error pulling layouts from local storage: {:?}", e);
        }
        layout_manager
            .stores
            .insert("local".to_string(), Box::from(local_store));

        // Get previously selected layout, if available
        layout_manager.selected = eframe::get_value::<LayoutRef>(storage, storage_keys::SELECTED);

        layout_manager
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
