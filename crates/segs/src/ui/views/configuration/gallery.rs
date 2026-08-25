use std::sync::atomic::{AtomicBool, AtomicU64};

use egui::{Id, Sense, Ui, Vec2, vec2};

use crate::{
    dataflow::{store::DataStore, StreamKey},
    ui::{
        components::widget_renderer::show_widget,
        widgets::{WidgetTrait, WidgetVariant},
    },
};

use super::{HitRegion, WidgetDragPayload, WidgetDragSource, next_drag_session};

/// Draws gallery cards and starts widget drags.
pub fn show(ui: &mut Ui, data_store: &mut DataStore) {
    data_store.ensure_mock_stream();

    for (index, variant) in WidgetVariant::gallery().into_iter().enumerate() {
        let mut preview = variant.clone();
        // Inject the mock stream for the gallery preview
        for mut setting in preview.data_settings() {
            setting.set_stream_if_empty(StreamKey::mock());
        }

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

            ui.disable();
            show_widget(ui, card_id.with("preview"), preview_rect, &preview, data_store);
        });

        let drag_response = ui
            .interact(card.response.rect, card_id.with("drag_source"), Sense::drag())
            .on_hover_cursor(egui::CursorIcon::Grab);
        if drag_response.drag_started() {
            egui::DragAndDrop::set_payload(
                ui.ctx(),
                WidgetDragPayload {
                    source: WidgetDragSource::Gallery(variant),
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
