use egui::emath::*;

#[inline(always)]
pub fn expand_to_pixel(mut rect: Rect, ppi: f32) -> Rect {
    rect.min = map_to_pixel_pos(rect.min, ppi, f32::floor);
    rect.max = map_to_pixel_pos(rect.max, ppi, f32::ceil);
    rect
}

#[inline(always)]
pub fn map_to_pixel_pos(mut pos: Pos2, ppi: f32, map: fn(f32) -> f32) -> Pos2 {
    pos.x = map_to_pixel(pos.x, ppi, map);
    pos.y = map_to_pixel(pos.y, ppi, map);
    pos
}

#[inline(always)]
pub fn map_to_pixel(point: f32, ppi: f32, map: fn(f32) -> f32) -> f32 {
    map(point * ppi) / ppi
}

pub fn rect_set_size_centered(rect: &mut Rect, size: Vec2) {
    let center = rect.center();
    rect.set_width(size.x);
    rect.set_height(size.y);
    rect.set_center(center);
}

/// Shrink a rectangle so that the stroke is fully contained inside
/// the original rectangle.
pub fn rect_stroke_box(rect: Rect, width: f32) -> Rect {
    rect.expand(-f32::ceil(width / 2.0))
}
