//! Provides a source of layouts

use crate::{APP_NAME, ui::app::AppState};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::remove_file, path::PathBuf};

/// Layouts are identified by their name
pub type LayoutStoreKey = String;

pub trait LayoutStore {
    // These functions only accesses the local copy
    fn layouts(&self) -> Vec<&LayoutStoreKey>;
    fn contains(&self, id: &LayoutStoreKey) -> bool;
    fn get(&self, id: &LayoutStoreKey) -> Result<&AppState>;

    /// Add the given layout if not present in the store.
    /// [LayoutStore::push] as to be called to synchronize upstream.
    ///
    /// **WARNING**: This function is expensive to run because it accesses the source
    fn add(&mut self, id: &LayoutStoreKey, layout: &AppState) -> Result<()> {
        if self.contains(id) {
            return Err(anyhow!("The layout already exists in the store"));
        }
        self.push(id, layout);
        Ok(())
    }

    /// Updates a given layout
    ///
    /// **WARNING**: This function is expensive to run because it accesses the source
    fn update(&mut self, id: &LayoutStoreKey, layout: &AppState) -> Result<()> {
        if !self.contains(id) {
            return Err(anyhow!("The layout does not exists in the store"));
        }
        self.push(id, layout);
        Ok(())
    }

    /// Deletes a given layout
    ///
    /// **WARNING**: This function is expensive to run because it accesses the source
    fn delete(&mut self, id: &LayoutStoreKey) -> Result<()>;

    /// Push the given layout to the store, regardless of wether the layout is present or not in the store
    ///
    /// **WARNING**: This function is expensive to run because it accesses the source
    fn push(&mut self, id: &LayoutStoreKey, layout: &AppState) -> Result<()>;

    /// Pull available layouts from the source
    ///
    /// **WARNING**: This function is expensive to run because it accesses the source
    fn pull_all(&mut self) -> Result<()>;

    /// Pull one single layout from the source
    ///
    /// **WARNING**: This function is expensive to run because it accesses the source
    fn pull_one(&mut self, id: &LayoutStoreKey) -> Result<()>;
}

static LOCAL_STORE_PATH: &str = "layouts";

#[derive(Serialize, Deserialize)]
pub struct LayoutLocalStore {
    path: PathBuf,
    #[serde(skip)]
    layouts: BTreeMap<LayoutStoreKey, AppState>,
}

impl LayoutLocalStore {
    fn id_to_path(&self, id: &LayoutStoreKey) -> PathBuf {
        self.path.join(PathBuf::from(id).with_extension("json"))
    }
}

impl Default for LayoutLocalStore {
    fn default() -> Self {
        Self {
            // TODO: Gracefully handle this
            path: eframe::storage_dir(APP_NAME)
                .unwrap()
                .join(LOCAL_STORE_PATH),
            layouts: BTreeMap::default(),
        }
    }
}

impl LayoutStore for LayoutLocalStore {
    fn layouts(&self) -> Vec<&LayoutStoreKey> {
        self.layouts.keys().collect()
    }

    fn get(&self, id: &LayoutStoreKey) -> Result<&AppState> {
        self.layouts
            .get(id)
            .ok_or(anyhow!("Layout not found in store"))
    }

    fn contains(&self, id: &LayoutStoreKey) -> bool {
        self.layouts.contains_key(id)
    }

    fn delete(&mut self, id: &LayoutStoreKey) -> Result<()> {
        self.layouts.remove(id).ok_or(anyhow!(
            "Unable to remove the layout, it's not in the store"
        ))?;
        remove_file(self.id_to_path(id))?;
        Ok(())
    }

    fn push(&mut self, id: &LayoutStoreKey, layout: &AppState) -> Result<()> {
        let path = self.id_to_path(id);
        println!("Pushed file {:?}", path);
        layout.to_file(&path)?;
        self.layouts.insert(id.clone(), layout.clone());
        Ok(())
    }

    fn pull_all(&mut self) -> Result<()> {
        let dir_content = self.path.read_dir()?.flatten();
        let json_files = dir_content.filter(|file| {
            file.file_name()
                .to_str()
                .is_some_and(|name| name.ends_with("json"))
        });
        let ids: Vec<LayoutStoreKey> = json_files
            .flat_map(|file| -> Result<LayoutStoreKey> {
                Ok(file
                    .path()
                    .file_stem()
                    .ok_or(anyhow!("Unable to get file stem from path"))?
                    .to_str()
                    .ok_or(anyhow!("Unable to transform from OsStr to str"))?
                    .to_string())
            })
            .collect();
        for id in ids {
            // TODO: How do we handle failures here? Do we fail if at leat one file coudn't be read?
            // For now we'll fail only if the dir coudn't be read
            self.pull_one(&id);
        }
        Ok(())
    }

    fn pull_one(&mut self, id: &LayoutStoreKey) -> Result<()> {
        let path = self.id_to_path(id);
        let layout = AppState::from_file(&path)?;
        self.layouts.insert(id.clone(), layout.clone());
        Ok(())
    }
}
