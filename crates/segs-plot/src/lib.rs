//! Plotting primitives used by SEGS.
//!
//! This crate exposes upstream [`egui_plot`] and adds a mapped line that can
//! render caller-owned data without first materializing [`PlotPoint`] values.

mod mapped_line;

pub use egui_plot::*;
pub use mapped_line::mapped_line;
