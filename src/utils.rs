use egui::*;

#[inline(always)]
pub fn expand_to_pixel(mut rect: Rect, ppi: f32) -> egui::Rect {
    rect.min = map_to_pixel_pos(rect.min, ppi, f32::floor);
    rect.max = map_to_pixel_pos(rect.max, ppi, f32::ceil);
    rect
}

#[inline(always)]
pub fn map_to_pixel_pos(mut pos: Pos2, ppi: f32, map: fn(f32) -> f32) -> egui::Pos2 {
    pos.x = map_to_pixel(pos.x, ppi, map);
    pos.y = map_to_pixel(pos.y, ppi, map);
    pos
}

#[inline(always)]
pub fn map_to_pixel(point: f32, ppi: f32, map: fn(f32) -> f32) -> f32 {
    map(point * ppi) / ppi
}
