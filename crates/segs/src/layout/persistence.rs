use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;
use thiserror::Error;

use super::{CURRENT_LAYOUT_SCHEMA, Layout};

#[derive(Debug, Error)]
pub enum LayoutStoreError {
    #[error("Layout storage error: {0}.")]
    Io(#[from] io::Error),
    #[error("Invalid layout JSON: {0}.")]
    Json(#[from] serde_json::Error),
    #[error("Layout '{0}' uses unsupported schema version.")]
    UnsupportedSchema(String),
    #[error("Layout filename does not match serialized slug '{0}'.")]
    SlugMismatch(String),
}

#[derive(Debug)]
pub struct LayoutStore {
    directory: PathBuf,
}

impl LayoutStore {
    /// Creates a store rooted at the application's layouts directory.
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    /// Loads all valid JSON layouts and reports malformed entries as warnings.
    pub fn load_all(&self) -> Result<(Vec<Layout>, Vec<String>), LayoutStoreError> {
        fs::create_dir_all(&self.directory)?;
        let mut paths = fs::read_dir(&self.directory)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "json"))
            .collect::<Vec<_>>();
        paths.sort();

        let mut layouts = Vec::new();
        let mut warnings = Vec::new();
        for path in paths {
            match self.load_path(&path) {
                Ok(layout) => layouts.push(layout),
                Err(error) => warnings.push(format!("{}: {error}", path.display())),
            }
        }
        Ok((layouts, warnings))
    }

    fn load_path(&self, path: &Path) -> Result<Layout, LayoutStoreError> {
        let bytes = fs::read(path)?;
        let layout: Layout = serde_json::from_slice(&bytes)?;
        if layout.schema_version != CURRENT_LAYOUT_SCHEMA {
            return Err(LayoutStoreError::UnsupportedSchema(layout.slug));
        }
        if path.file_stem().and_then(|stem| stem.to_str()) != Some(layout.slug.as_str()) {
            return Err(LayoutStoreError::SlugMismatch(layout.slug));
        }
        Ok(layout)
    }

    /// Returns whether a layout file exists for the slug.
    pub fn contains(&self, slug: &str) -> bool {
        self.path(slug).exists()
    }

    /// Serializes a complete layout and atomically replaces its file.
    pub fn save(&self, layout: &Layout) -> Result<(), LayoutStoreError> {
        fs::create_dir_all(&self.directory)?;
        let bytes = serde_json::to_vec_pretty(layout)?;
        self.atomic_write(&self.path(&layout.slug), &bytes)
    }

    /// Writes a renamed layout and removes its former file.
    pub fn rename(&self, old_slug: &str, layout: &Layout) -> Result<(), LayoutStoreError> {
        if old_slug == layout.slug {
            return self.save(layout);
        }
        let new_path = self.path(&layout.slug);
        if new_path.exists() {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "target layout already exists").into());
        }
        self.save(layout)?;
        if let Err(error) = fs::remove_file(self.path(old_slug)) {
            // Roll back the new file when the old filename cannot be removed
            let _ = fs::remove_file(new_path);
            return Err(error.into());
        }
        Ok(())
    }

    /// Permanently deletes the layout file for the slug.
    pub fn delete(&self, slug: &str) -> Result<(), LayoutStoreError> {
        fs::remove_file(self.path(slug)).map_err(Into::into)
    }

    fn path(&self, slug: &str) -> PathBuf {
        self.directory.join(format!("{slug}.json"))
    }

    fn atomic_write(&self, destination: &Path, bytes: &[u8]) -> Result<(), LayoutStoreError> {
        // Flush a same-directory temporary file before atomically replacing the destination
        let mut temporary = NamedTempFile::new_in(&self.directory)?;
        temporary.write_all(bytes)?;
        temporary.as_file().sync_all()?;
        temporary.persist(destination).map_err(|error| error.error)?;
        Ok(())
    }
}
