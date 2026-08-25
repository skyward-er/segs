use egui::{CornerRadius, Id, Rect, Sense, StrokeKind, Ui, UiBuilder, Vec2, pos2};
use segs_ui::style::CtxStyleExt;

use crate::{
    dataflow::store::DataStore,
    ui::{
        grid::Grid,
        widgets::{WidgetData, WidgetTrait, WidgetVariant},
    },
};

/// Draws widgets on the grid.
pub fn show_widgets<'a>(
    ui: &mut Ui,
    widgets: impl IntoIterator<Item = &'a WidgetData>,
    grid: &Grid,
    data_store: &mut DataStore,
) {
    for widget in widgets {
        show_widget(
            ui,
            widget.id,
            grid.to_screen_rect(widget.grect),
            &widget.variant,
            data_store,
        );
    }
}

/// Draws one widget in the standard visual container.
pub fn show_widget(ui: &mut Ui, id: Id, rect: Rect, widget: &WidgetVariant, data_store: &mut DataStore) {
    let app_style = ui.app_style();
    let corner_radius = CornerRadius::same(1);

    ui.scope_builder(UiBuilder::new().id(id.with("_container")).max_rect(rect), |ui| {
        // Paint the background outside disabled content opacity
        ui.ctx()
            .layer_painter(ui.layer_id())
            .with_clip_rect(ui.clip_rect())
            .rect_filled(rect, corner_radius, app_style.main_panels_fill);

        let response = ui.allocate_rect(rect, Sense::empty());

        ui.scope_builder(UiBuilder::new().id(id).max_rect(rect), |ui| {
            ui.painter().rect_stroke(
                response.rect,
                corner_radius,
                app_style.main_view_stroke,
                StrokeKind::Outside,
            );
            ui.set_clip_rect(rect);
            widget.show(ui, data_store);
        });
    });
}

/// Draws the snapping guide used in configuration mode.
pub fn show_snapping_guide(ui: &Ui, grid: &Grid) {
    const DOT_RADIUS: f32 = 0.75;

    let painter = ui.painter();
    let color = ui.visuals().weak_text_color().gamma_multiply(0.75);
    let Vec2 {
        x: spacing_x,
        y: spacing_y,
    } = grid.cell_size;

    let start_x = grid.rect.min.x + spacing_x;
    let end_x = grid.rect.max.x - spacing_x + 1.;
    let start_y = grid.rect.min.y + spacing_y;
    let end_y = grid.rect.max.y - spacing_y + 1.;

    let mut y = start_y;
    while y < end_y {
        let mut x = start_x;
        while x < end_x {
            painter.circle_filled(pos2(x, y), DOT_RADIUS, color);
            x += spacing_x;
        }
        y += spacing_y;
    }
}
