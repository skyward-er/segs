//! Saving a P&ID to a file of its own and loading it back, to reuse a diagram in other layouts.
//!
//! Stream bindings are saved as they are. They point at the same data again only when the file
//! is loaded with the same data source and MAVLink profile it was saved with.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::PidWidget;
use crate::layout::CURRENT_LAYOUT_SCHEMA;

/// Marks a file as a SEGS v2 P&ID export.
const FORMAT: &str = "segs-pid";

/// Writes `diagram` to `path` as JSON.
///
/// Returns a readable error when the file cannot be written.
pub(super) fn export(path: &Path, diagram: &PidWidget) -> Result<(), String> {
    let file = ExportFile {
        format: FORMAT,
        schema_version: CURRENT_LAYOUT_SCHEMA,
        diagram,
    };
    let json = serde_json::to_string_pretty(&file).map_err(|error| format!("Cannot encode the diagram: {error}"))?;
    std::fs::write(path, json).map_err(|error| format!("Cannot write {}: {error}", path.display()))
}

/// Reads a diagram exported with [`export`] and appends it to `diagram`.
///
/// Files from older layout schemas are read as they are, since the P&ID wire format has not
/// changed since the widget was added in schema 9. A change to that format must convert older
/// files here before decoding them.
///
/// Returns the number of symbols added, or a readable error when the file cannot be read,
/// is not a P&ID export, or was written by a newer version; on error `diagram` is unchanged.
pub(super) fn import(path: &Path, diagram: &mut PidWidget) -> Result<usize, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("Cannot read {}: {error}", path.display()))?;

    // Check what the file is before decoding the diagram, for a clearer error
    let header: Header = serde_json::from_str(&text).map_err(|_| "Not a SEGS P&ID file".to_owned())?;
    if header.format != FORMAT {
        return Err("Not a SEGS P&ID file".to_owned());
    }
    if header.schema_version > CURRENT_LAYOUT_SCHEMA {
        return Err(format!(
            "Saved by a newer version with layout schema {}, this version reads up to {}",
            header.schema_version, CURRENT_LAYOUT_SCHEMA
        ));
    }

    let file: ImportFile = serde_json::from_str(&text).map_err(|error| format!("Damaged P&ID file: {error}"))?;
    let elements = file.diagram.elements.len();
    diagram.append(file.diagram);
    Ok(elements)
}

#[derive(Serialize)]
struct ExportFile<'a> {
    format: &'a str,
    /// Layout schema the diagram was serialized with, since it shares the layout wire format.
    schema_version: u32,
    diagram: &'a PidWidget,
}

#[derive(Deserialize)]
struct Header {
    format: String,
    schema_version: u32,
}

#[derive(Deserialize)]
struct ImportFile {
    diagram: PidWidget,
}
