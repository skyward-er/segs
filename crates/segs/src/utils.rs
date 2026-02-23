pub mod git;

use std::path::PathBuf;

use directories::ProjectDirs;

/// Returns the directory path where the app's memory data should be stored.
#[inline]
pub fn get_memory_dirpath() -> PathBuf {
    project_dirs().data_dir().to_path_buf().join("metadata")
}

/// Returns the directory path where the app downloaded data should be stored.
#[inline]
pub fn get_downloaded_dirpath() -> PathBuf {
    project_dirs().data_dir().to_path_buf().join("downloaded")
}

// We use different directories for development and production to avoid
// conflicts and ensure that we don't accidentally delete important data during
// development.

#[cfg(debug_assertions)]
fn project_dirs() -> ProjectDirs {
    directories::ProjectDirs::from("eu", "skyward", "segs-dev").expect("Could not determine project directories")
}

#[cfg(not(debug_assertions))]
fn project_dirs() -> ProjectDirs {
    directories::ProjectDirs::from("eu", "skyward", "segs").expect("Could not determine project directories")
}
