use mavlink_bindgen::BindGenError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProfileParseError {
    /// Represents a failure to read the MAVLink profile file.
    #[error("Could not read profile file {}: {err}", path.display())]
    FsError {
        err: std::io::Error,
        path: std::path::PathBuf,
    },
    /// Represents a failure to parse the MAVLink profile file.
    #[error("Could not parse profile file {}: {err}", path.display())]
    ParseError {
        err: BindGenError,
        path: std::path::PathBuf,
    },
}
