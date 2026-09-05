use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use chrono::Utc;
use egui::ahash::{HashSet, HashSetExt};
use rand::random;
use thiserror::Error;

use super::{
    Layout, LayoutNameError,
    model::{renamed_slug, slug_with_suffix, validated_display_name},
    persistence::{LayoutStore, LayoutStoreError},
};

#[derive(Debug, Error)]
pub enum LayoutManagerError {
    #[error(transparent)]
    Name(#[from] LayoutNameError),
    #[error(transparent)]
    Store(#[from] LayoutStoreError),
    #[error("Layout '{0}' was not found.")]
    NotFound(String),
    #[error("Layout slug '{0}' already exists.")]
    DuplicateSlug(String),
    #[error("Layout '{0}' has an invalid slug.")]
    InvalidSlug(String),
}

/// Keeps the editable active layout alongside its last persisted state.
#[derive(Debug, Clone)]
struct ActiveLayout {
    working: Layout,
    saved: Layout,
}

/// Owns the saved-layout catalog, active working copy, and persistence operations.
#[derive(Debug)]
pub struct LayoutManager {
    store: LayoutStore,
    layouts: BTreeMap<String, Layout>,
    active: Option<ActiveLayout>,
    default_slug: Option<String>,
    default_update: Option<Option<String>>,
    warnings: Vec<String>,
}

impl LayoutManager {
    /// Loads the layout catalog and activates the requested default when it exists.
    pub fn load(directory: PathBuf, requested_default: Option<String>) -> Result<Self, LayoutManagerError> {
        let store = LayoutStore::new(directory);
        let (loaded, mut warnings) = store.load_all()?;
        let mut layouts = BTreeMap::new();
        let mut names = HashSet::<String>::new();
        for layout in loaded {
            // Ignore invalid entries without preventing the rest of the catalog from loading
            if validated_display_name(&layout.name).as_deref() != Ok(layout.name.as_str())
                || renamed_slug(&layout.name, &layout.slug).as_deref() != Some(layout.slug.as_str())
            {
                warnings.push(format!("{}: invalid layout slug.", layout.slug));
                continue;
            }
            if names.contains(&layout.name) {
                warnings.push(format!("{}: duplicate display name '{}'.", layout.slug, layout.name));
                continue;
            }
            if layouts.contains_key(&layout.slug) {
                warnings.push(format!("{}: duplicate layout slug.", layout.slug));
                continue;
            }
            names.insert(layout.name.clone());
            layouts.insert(layout.slug.clone(), layout);
        }

        let had_requested_default = requested_default.is_some();
        let default_valid = requested_default
            .as_ref()
            .is_some_and(|slug| layouts.contains_key(slug));
        let default_slug = requested_default.filter(|_| default_valid);
        let default_update = (had_requested_default && !default_valid).then_some(None);
        if had_requested_default && !default_valid {
            warnings.push("The configured default layout no longer exists.".to_owned());
        }
        let active = default_slug
            .as_ref()
            .and_then(|slug| layouts.get(slug))
            .cloned()
            .map(|layout| ActiveLayout {
                working: layout.clone(),
                saved: layout,
            });

        Ok(Self {
            store,
            layouts,
            active,
            default_slug,
            default_update,
            warnings,
        })
    }

    /// Iterates over saved layouts in slug order.
    pub fn layouts(&self) -> impl Iterator<Item = &Layout> {
        self.layouts.values()
    }

    /// Returns the directory containing the persisted layout catalog.
    pub fn directory(&self) -> &Path {
        self.store.directory()
    }

    /// Returns a saved layout by slug.
    pub fn layout(&self, slug: &str) -> Option<&Layout> {
        self.layouts.get(slug)
    }

    /// Returns the active in-memory working layout.
    pub fn active(&self) -> Option<&Layout> {
        self.active.as_ref().map(|active| &active.working)
    }

    /// Returns the active in-memory working layout for editing.
    pub fn active_mut(&mut self) -> Option<&mut Layout> {
        self.active.as_mut().map(|active| &mut active.working)
    }

    /// Returns the slug of the active layout.
    pub fn active_slug(&self) -> Option<&str> {
        self.active().map(|layout| layout.slug.as_str())
    }

    /// Returns the slug configured for startup.
    pub fn default_slug(&self) -> Option<&str> {
        self.default_slug.as_deref()
    }

    /// Returns non-fatal problems encountered while loading the catalog.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Returns whether the active working copy differs from its saved baseline.
    pub fn is_dirty(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|active| active.working != active.saved)
    }

    /// Replaces the active working copy with the selected saved layout.
    pub fn activate(&mut self, slug: &str) -> Result<(), LayoutManagerError> {
        let layout = self
            .layouts
            .get(slug)
            .cloned()
            .ok_or_else(|| LayoutManagerError::NotFound(slug.to_owned()))?;
        self.active = Some(ActiveLayout {
            working: layout.clone(),
            saved: layout,
        });
        Ok(())
    }

    /// Creates, persists, and activates a new empty layout.
    pub fn create_empty(&mut self, name: &str) -> Result<String, LayoutManagerError> {
        let name = self.validate_name(name, None)?;
        let slug = self.new_slug(&name);
        let layout = Layout::empty(name, slug.clone());
        self.store.save(&layout)?;
        self.layouts.insert(slug.clone(), layout.clone());
        self.active = Some(ActiveLayout {
            working: layout.clone(),
            saved: layout,
        });
        Ok(slug)
    }

    /// Copies a saved layout under a new name, then persists and activates it.
    pub fn duplicate(&mut self, source_slug: &str, name: &str) -> Result<String, LayoutManagerError> {
        let name = self.validate_name(name, None)?;
        let mut layout = self
            .layouts
            .get(source_slug)
            .cloned()
            .ok_or_else(|| LayoutManagerError::NotFound(source_slug.to_owned()))?;
        let now = Utc::now();
        layout.slug = self.new_slug(&name);
        layout.name = name;
        layout.created_at = now;
        layout.modified_at = now;
        self.store.save(&layout)?;
        let slug = layout.slug.clone();
        self.layouts.insert(slug.clone(), layout.clone());
        self.active = Some(ActiveLayout {
            working: layout.clone(),
            saved: layout,
        });
        Ok(slug)
    }

    /// Renames a saved layout while preserving the random part of its slug.
    pub fn rename(&mut self, slug: &str, name: &str) -> Result<String, LayoutManagerError> {
        let name = self.validate_name(name, Some(slug))?;
        let mut saved = self
            .layouts
            .get(slug)
            .cloned()
            .ok_or_else(|| LayoutManagerError::NotFound(slug.to_owned()))?;
        let new_slug = renamed_slug(&name, slug).ok_or_else(|| LayoutManagerError::InvalidSlug(slug.to_owned()))?;
        if new_slug != slug && (self.layouts.contains_key(&new_slug) || self.store.contains(&new_slug)) {
            return Err(LayoutManagerError::DuplicateSlug(new_slug));
        }
        saved.name = name.clone();
        saved.slug = new_slug.clone();
        saved.modified_at = Utc::now();
        self.store.rename(slug, &saved)?;
        self.layouts.remove(slug);
        self.layouts.insert(new_slug.clone(), saved.clone());

        // Keep the working changes while replacing their saved baseline with the renamed file
        if let Some(active) = &mut self.active
            && active.working.slug == slug
        {
            active.working.slug = new_slug.clone();
            active.working.name = name;
            active.working.modified_at = saved.modified_at;
            active.saved = saved;
        }
        if self.default_slug.as_deref() == Some(slug) {
            self.default_slug = Some(new_slug.clone());
            self.default_update = Some(Some(new_slug.clone()));
        }
        Ok(new_slug)
    }

    /// Persists the active working copy and makes it the new saved baseline.
    pub fn save_active(&mut self) -> Result<(), LayoutManagerError> {
        let active = self
            .active
            .as_mut()
            .ok_or_else(|| LayoutManagerError::NotFound("active".into()))?;
        active.working.modified_at = Utc::now();
        self.store.save(&active.working)?;
        active.saved = active.working.clone();
        self.layouts.insert(active.working.slug.clone(), active.working.clone());
        Ok(())
    }

    /// Restores the active layout from its saved baseline.
    pub fn discard_active(&mut self) {
        if let Some(active) = &mut self.active {
            active.working = active.saved.clone();
        }
    }

    /// Permanently deletes a saved layout and clears dependent state.
    pub fn delete(&mut self, slug: &str) -> Result<(), LayoutManagerError> {
        if !self.layouts.contains_key(slug) {
            return Err(LayoutManagerError::NotFound(slug.to_owned()));
        }
        self.store.delete(slug)?;
        self.layouts.remove(slug);
        if self.active_slug() == Some(slug) {
            self.active = None;
        }
        if self.default_slug.as_deref() == Some(slug) {
            self.default_slug = None;
            self.default_update = Some(None);
        }
        Ok(())
    }

    /// Sets or clears the layout loaded at application startup.
    pub fn set_default(&mut self, slug: Option<&str>) -> Result<(), LayoutManagerError> {
        if let Some(slug) = slug
            && !self.layouts.contains_key(slug)
        {
            return Err(LayoutManagerError::NotFound(slug.to_owned()));
        }
        let new_default = slug.map(ToOwned::to_owned);
        if self.default_slug != new_default {
            self.default_slug = new_default.clone();
            self.default_update = Some(new_default);
        }
        Ok(())
    }

    /// Returns and clears a pending default-layout memory update.
    pub fn take_default_update(&mut self) -> Option<Option<String>> {
        self.default_update.take()
    }

    /// Validates a display name and checks it for exact duplicates.
    pub fn validate_name(&self, name: &str, excluding: Option<&str>) -> Result<String, LayoutManagerError> {
        let name = validated_display_name(name)?;
        if self
            .layouts
            .values()
            .any(|layout| layout.slug != excluding.unwrap_or_default() && layout.name == name)
        {
            return Err(LayoutNameError::Duplicate(name).into());
        }
        Ok(name)
    }

    fn new_slug(&self, name: &str) -> String {
        self.new_slug_with_source(name, random)
    }

    fn new_slug_with_source(&self, name: &str, mut next: impl FnMut() -> u32) -> String {
        loop {
            let slug = slug_with_suffix(name, next());
            if !self.layouts.contains_key(&slug) && !self.store.contains(&slug) {
                return slug;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use rand::random;

    use super::*;
    use crate::layout::model::slug_suffix;

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("segs-layout-test-{:016x}", random::<u64>()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn persists_renames_defaults_and_deletes() {
        // Create a layout and verify that creation immediately persists its file
        let directory = TestDirectory::new();
        let mut manager = LayoutManager::load(directory.0.clone(), None).unwrap();
        let slug = manager.create_empty("Flight Main").unwrap();
        assert!(directory.0.join(format!("{slug}.json")).exists());

        // Rename the default layout and verify both its file and pending default update follow the new slug
        manager.set_default(Some(&slug)).unwrap();
        assert_eq!(manager.take_default_update(), Some(Some(slug.clone())));
        let renamed = manager.rename(&slug, "Flight Primary").unwrap();
        assert!(renamed.ends_with(slug_suffix(&slug).unwrap()));
        assert!(!directory.0.join(format!("{slug}.json")).exists());
        assert!(directory.0.join(format!("{renamed}.json")).exists());
        assert_eq!(manager.take_default_update(), Some(Some(renamed.clone())));

        // Delete the active default and verify dependent in-memory state is cleared
        manager.delete(&renamed).unwrap();
        assert!(manager.active().is_none());
        assert_eq!(manager.take_default_update(), Some(None));
    }

    #[test]
    fn rejects_exact_names_but_allows_case_variants() {
        // Establish an existing display name in the catalog
        let directory = TestDirectory::new();
        let mut manager = LayoutManager::load(directory.0.clone(), None).unwrap();
        let first = manager.create_empty("Flight").unwrap();

        // Exact duplicate names should fail validation
        assert!(matches!(
            manager.create_empty("Flight"),
            Err(LayoutManagerError::Name(LayoutNameError::Duplicate(_)))
        ));

        // Case variants remain distinct under the catalog's case-sensitive policy
        let second = manager.create_empty("flight").unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn loads_default_as_active_once() {
        // Persist a layout that can be requested as the default on the next load
        let directory = TestDirectory::new();
        let mut manager = LayoutManager::load(directory.0.clone(), None).unwrap();
        let slug = manager.create_empty("Default").unwrap();
        drop(manager);

        // Reloading with that slug should select it as both the active and default layout
        let manager = LayoutManager::load(directory.0.clone(), Some(slug.clone())).unwrap();
        assert_eq!(manager.active_slug(), Some(slug.as_str()));
        assert_eq!(manager.default_slug(), Some(slug.as_str()));
    }

    #[test]
    fn slug_generation_retries_collisions() {
        // Reserve the first deterministic suffix in the in-memory catalog
        let directory = TestDirectory::new();
        let mut manager = LayoutManager::load(directory.0.clone(), None).unwrap();
        let occupied = Layout::empty("Flight".into(), "flight-00000007".into());
        manager.layouts.insert(occupied.slug.clone(), occupied);

        // The generator should reject the occupied suffix and accept the next value
        let mut suffixes = [7, 8].into_iter();
        assert_eq!(
            manager.new_slug_with_source("Flight", || suffixes.next().unwrap()),
            "flight-00000008"
        );
    }

    #[test]
    fn dirty_layout_can_be_discarded_or_saved() {
        // Modify the active working copy, then verify discarding restores its saved baseline
        let directory = TestDirectory::new();
        let mut manager = LayoutManager::load(directory.0.clone(), None).unwrap();
        manager.create_empty("Flight").unwrap();
        manager.active_mut().unwrap().grid_settings.cols = 12;
        assert!(manager.is_dirty());
        manager.discard_active();
        assert_eq!(manager.active().unwrap().grid_settings.cols, 8);
        assert!(!manager.is_dirty());

        // Modify it again, save, and reload to verify the new baseline was persisted
        manager.active_mut().unwrap().grid_settings.rows = 15;
        manager.save_active().unwrap();
        assert!(!manager.is_dirty());
        let slug = manager.active_slug().unwrap().to_owned();
        drop(manager);
        let manager = LayoutManager::load(directory.0.clone(), Some(slug)).unwrap();
        assert_eq!(manager.active().unwrap().grid_settings.rows, 15);
    }

    #[test]
    fn malformed_layout_does_not_hide_valid_layouts() {
        // Place one valid layout and one malformed JSON file in the catalog directory
        let directory = TestDirectory::new();
        let mut manager = LayoutManager::load(directory.0.clone(), None).unwrap();
        manager.create_empty("Valid").unwrap();
        fs::write(directory.0.join("broken-deadbeef.json"), b"not json").unwrap();
        drop(manager);

        // Loading should retain the valid entry and report the malformed file as a warning
        let manager = LayoutManager::load(directory.0.clone(), None).unwrap();
        assert_eq!(manager.layouts().count(), 1);
        assert_eq!(manager.warnings().len(), 1);
    }

    #[test]
    fn duplicate_names_do_not_hide_unrelated_layouts() {
        // Persist two valid slugs with the same display name plus one unrelated layout
        let directory = TestDirectory::new();
        let store = LayoutStore::new(directory.0.clone());
        store
            .save(&Layout::empty("Duplicate".into(), "duplicate-00000001".into()))
            .unwrap();
        store
            .save(&Layout::empty("Duplicate".into(), "duplicate-00000002".into()))
            .unwrap();
        store
            .save(&Layout::empty("Independent".into(), "independent-00000003".into()))
            .unwrap();

        // Loading should keep one duplicate, preserve the unrelated layout, and warn about the rejected entry
        let manager = LayoutManager::load(directory.0.clone(), None).unwrap();
        let names = manager
            .layouts()
            .map(|layout| layout.name.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(names, ["Duplicate", "Independent"].into_iter().collect());
        assert!(
            manager
                .warnings()
                .iter()
                .any(|warning| warning.contains("duplicate display name"))
        );
    }
}
