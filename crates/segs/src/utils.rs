use std::path::PathBuf;

use directories::ProjectDirs;

pub fn get_memory_dirpath() -> PathBuf {
    let mut path = project_dirs().data_dir().to_path_buf();
    // We use different directories for development and production to avoid
    // conflicts and ensure that we don't accidentally delete important data during
    // development.
    #[cfg(debug_assertions)]
    path.push("metadata_dev");
    #[cfg(not(debug_assertions))]
    path.push("metadata");
    path
}

fn project_dirs() -> ProjectDirs {
    directories::ProjectDirs::from("eu", "skyward", "segs").expect("Could not determine project directories")
}
