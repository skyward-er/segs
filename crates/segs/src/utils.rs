// pub mod git;

use std::path::PathBuf;

use directories::ProjectDirs;

/// Returns the platform directory containing all application data.
///
/// The returned path uses the development or production SEGS identity selected
/// for the current build.
#[inline]
pub(super) fn get_data_dirpath() -> PathBuf {
    project_dirs().data_dir().to_path_buf()
}

/// Returns the directory path where the app's memory data should be stored.
#[inline]
pub fn get_memory_dirpath() -> PathBuf {
    get_data_dirpath().join("metadata")
}

/// Returns the directory containing user-created layouts.
#[inline]
pub fn get_layouts_dirpath() -> PathBuf {
    get_data_dirpath().join("layouts")
}

// We use different directories for development and production to avoid
// conflicts and ensure that we don't accidentally delete important data during
// development.

#[cfg(debug_assertions)]
fn project_dirs() -> ProjectDirs {
    directories::ProjectDirs::from("eu", "skywarder", "segs2-dev").expect("Could not determine project directories")
}

#[cfg(not(debug_assertions))]
fn project_dirs() -> ProjectDirs {
    directories::ProjectDirs::from("eu", "skywarder", "segs2").expect("Could not determine project directories")
}
