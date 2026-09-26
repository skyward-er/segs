//! Shaped measurements and reusable raster sizes for read-only, automatically fitted text.

use std::sync::Arc;

use egui::{Align, Color32, FontId, Galley, Pos2, Ui, emath::TSTransform, epaint::TextShape, text::LayoutJob};

/// Reference size in logical points, small enough to measure without large glyph allocations.
pub const REFERENCE_SIZE: f32 = 16.;

/// Returns shaped, unwrapped text at the reference size using egui's font-aware layout cache.
/// The returned dimensions are in logical points; divide by `REFERENCE_SIZE` for unit-size metrics.
pub fn measure(ui: &Ui, text: &str, mut font: FontId) -> Arc<Galley> {
    font.size = REFERENCE_SIZE;
    layout(ui, text.to_owned(), font, Align::LEFT, false)
}

/// Returns a paint-only galley with explicit newlines and the requested alignment and size.
/// Automatic text is rasterized at the next power-of-two size and scaled down to avoid
/// rasterizing every intermediate size during resizing; fixed text is laid out natively.
/// Colors must be supplied with `galley_with_override_text_color` when painting.
pub fn layout(ui: &Ui, text: String, mut font: FontId, alignment: Align, automatic: bool) -> Arc<Galley> {
    // Use stable raster sizes while leaving the displayed size continuous
    let requested_size = font.size;
    if automatic {
        font.size = requested_size.max(REFERENCE_SIZE).log2().ceil().exp2();
    }
    let scale = if automatic { requested_size / font.size } else { 1. };
    let mut job = LayoutJob::simple(text, font, Color32::WHITE, f32::INFINITY);
    job.halign = alignment;
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));

    // Transform a copy of the cached geometry without retaining atlas-dependent galleys
    scaled(galley, scale)
}

/// Returns the galley's painting geometry uniformly scaled by `scale` without rerasterizing.
/// Glyph cursor metrics remain unscaled, so the result must not be used for selection or ellipsis.
/// A scale of one returns the original shared galley without copying its geometry.
pub fn scaled(galley: Arc<Galley>, scale: f32) -> Arc<Galley> {
    if scale == 1. {
        return galley;
    }
    let mut shape = TextShape::new(Pos2::ZERO, galley, Color32::WHITE);
    shape.transform(TSTransform::from_scaling(scale));
    shape.galley
}
