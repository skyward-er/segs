use std::sync::atomic::{AtomicBool, AtomicU64};

use egui::{Id, Sense, Ui, Vec2, vec2};

use crate::{
    dataflow::preview::PreviewContext,
    ui::{
        components::widget_renderer::show_widget,
        widgets::{WidgetTrait, WidgetVariant},
    },
};

use super::{HitRegion, WidgetDragPayload, WidgetDragSource, next_drag_session};

/// Retains sample data and prepared widgets for one configuration session.
pub(super) struct Gallery {
    /// Isolated animated telemetry shared by the gallery previews.
    samples: PreviewContext,
    /// Default widgets and their sample-bound previews in display order.
    entries: Vec<GalleryEntry>,
}

impl Default for Gallery {
    /// Returns a gallery with each preview configured once against its isolated samples.
    fn default() -> Self {
        // Prepare sample-bound copies while preserving clean defaults for insertion
        let samples = PreviewContext::new();
        let entries = WidgetVariant::gallery()
            .into_iter()
            .map(|variant| {
                let mut preview = variant.clone();
                preview.configure_preview(&samples);
                GalleryEntry { variant, preview }
            })
            .collect();
        Self { samples, entries }
    }
}

impl Gallery {
    /// Draws visible previews and starts drags carrying unconfigured widget defaults.
    pub(super) fn show(&mut self, ui: &mut Ui) {
        // Update the retained sample streams only when their next values are due
        let repaint_after = self.samples.update();
        ui.ctx().request_repaint_after(repaint_after);

        // Allocate every card to preserve scroll geometry and stable drag identities
        for (index, GalleryEntry { variant, preview }) in self.entries.iter().enumerate() {
            let name = variant.display_name();
            let card_id = Id::new(("widget_gallery_card", index, name));

            let card = ui.scope(|ui| {
                ui.label(name);
                ui.add_space(4.);

                let default_size = variant.default_size();
                let aspect = if default_size.is_finite() && default_size.x > 0. && default_size.y > 0. {
                    default_size.y / default_size.x
                } else {
                    1.
                };
                let width = ui.available_width().max(1.);
                let preview_size = vec2(width, (width * aspect).clamp(56., 120.));
                let (preview_rect, _) = ui.allocate_exact_size(preview_size, Sense::hover());

                // Skip preview rendering outside the scroll viewport
                if ui.is_rect_visible(preview_rect) {
                    ui.disable();
                    show_widget(
                        ui,
                        card_id.with("preview"),
                        preview_rect,
                        preview,
                        &mut crate::ui::widgets::WidgetRenderContext::preview(self.samples.data_store()),
                    );
                }
            });

            let drag_response = ui
                .interact(card.response.rect, card_id.with("drag_source"), Sense::DRAG)
                .on_hover_cursor(egui::CursorIcon::Grab);
            if drag_response.drag_started() {
                egui::DragAndDrop::set_payload(
                    ui.ctx(),
                    WidgetDragPayload {
                        source: WidgetDragSource::Gallery(variant.clone()),
                        session: next_drag_session(),
                        snap_generation: AtomicU64::new(0),
                        snap_visible: AtomicBool::new(false),
                        interaction: HitRegion::INSIDE,
                        initial_rect: None,
                        pointer_offset: Vec2::ZERO,
                    },
                );
            }

            ui.add_space(10.);
        }
    }
}

/// A clean insertion default paired with its retained gallery-only preview.
struct GalleryEntry {
    /// Widget cloned into the drag payload without sample bindings.
    variant: WidgetVariant,
    /// Sample-bound widget used only for gallery rendering.
    preview: WidgetVariant,
}
