//! Contains widgets that can be added to a plot at some fixed screen
//! coordinates.

mod coordinates;
mod legend;

pub use coordinates::CoordinatesFormatter;
pub use legend::{ColorConflictHandling, Legend, LegendWidget};
