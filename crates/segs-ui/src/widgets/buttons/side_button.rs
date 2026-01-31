use egui::{Pos2, Rect};

fn snap_rect_to_pixels(rect: Rect, pixels_per_point: f32) -> Rect {
    let min_px = Pos2::new(rect.min.x * pixels_per_point, rect.min.y * pixels_per_point);
    let max_px = Pos2::new(rect.max.x * pixels_per_point, rect.max.y * pixels_per_point);

    let min_px = Pos2::new(min_px.x.round(), min_px.y.round());
    let max_px = Pos2::new(max_px.x.round(), max_px.y.round());

    Rect::from_min_max(
        Pos2::new(min_px.x / pixels_per_point, min_px.y / pixels_per_point),
        Pos2::new(max_px.x / pixels_per_point, max_px.y / pixels_per_point),
    )
}
