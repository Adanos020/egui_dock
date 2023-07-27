use crate::{AllowedSplits, NodeIndex, Split, Style, SurfaceIndex, TabDestination, TabIndex};
use egui::{vec2, Id, LayerId, Order, Pos2, Rect, Stroke, Ui, Vec2};

#[derive(Debug)]
pub(super) struct HoverData {
    pub rect: Rect,
    pub tabs: Option<Rect>,
    pub tab: Option<(Rect, TabIndex)>,
    pub dst: (SurfaceIndex, Option<NodeIndex>),
    pub pointer: Pos2,
}

impl HoverData {
    //determines if the hoverdata implies we're hovering over a tab or the tab title bar
    pub(super) fn is_on_title_bar(&self) -> bool {
        self.tab.is_some() || self.tabs.is_some()
    }

    //resolve a TabDestination for whatever is hovered
    pub(super) fn resolve(
        &self,
        ui: &mut Ui,
        style: &Style,
        allowed_splits: AllowedSplits,
        is_window: bool,
    ) -> TabDestination {
        if self.is_on_title_bar() {
            self.resolve_traditional(ui, style, allowed_splits)
        } else {
            self.resolve_icon_based(ui, style, allowed_splits, is_window)
        }
    }

    fn resolve_icon_based(
        &self,
        ui: &mut Ui,
        style: &Style,
        allowed_splits: AllowedSplits,
        is_window: bool,
    ) -> TabDestination {
        const PADDING: f32 = 10.0;
        const BUTTON_SPACING: f32 = 10.0;
        const TOTAL_BUTTON_SPACING: f32 = BUTTON_SPACING * 2.0;
        assert!(!self.is_on_title_bar());
        let (rect, pointer) = (self.rect, self.pointer);
        let rect = rect.shrink(PADDING);
        let shortest_side = ((rect.width() - TOTAL_BUTTON_SPACING) / 3.0)
            .min((rect.height() - TOTAL_BUTTON_SPACING) / 3.0)
            .min(100.0);
        let mut offset_vector = vec2(0.0, shortest_side + BUTTON_SPACING);

        let mut destination = None;

        let center = rect.center();

        if !is_window {
            if button_ui(
                Rect::from_center_size(center, Vec2::splat(shortest_side)),
                ui,
                pointer,
                style,
                None,
            ) {
                destination = Some(TabDestination::Append);
            }
        }
        for (split, is_top_bottom) in vec![
            (Split::Below, true),
            (Split::Right, false),
            (Split::Above, true),
            (Split::Left, false),
        ] {
            match allowed_splits {
                AllowedSplits::TopBottomOnly if is_top_bottom => continue,
                AllowedSplits::LeftRightOnly if !is_top_bottom => continue,
                AllowedSplits::None => continue,
                _ => {
                    if button_ui(
                        Rect::from_center_size(center + offset_vector, Vec2::splat(shortest_side)),
                        ui,
                        pointer,
                        style,
                        Some(is_top_bottom),
                    ) {
                        destination = Some(TabDestination::Split(split));
                    }
                    offset_vector = offset_vector.rot90();
                }
            }
        }

        destination.unwrap_or(TabDestination::Window(self.pointer))
    }

    fn resolve_traditional(
        &self,
        ui: &mut Ui,
        style: &Style,
        allowed_splits: AllowedSplits,
    ) -> TabDestination {
        if let Some(tab) = self.tab {
            draw_drop_rect(tab.0, ui, style);
            return TabDestination::Insert(tab.1);
        }
        if let Some(tabs) = self.tabs {
            draw_drop_rect(tabs, ui, style);
            return TabDestination::Append;
        }
        //technically this code is unreachable
        //but i don't want to remove it in case we want a setting to enable/disable icon based drops
        let (rect, pointer) = (self.rect, self.pointer);

        let center = rect.center();

        let pts = match allowed_splits {
            AllowedSplits::All => vec![
                (
                    center.distance(pointer),
                    TabDestination::Append,
                    Rect::EVERYTHING,
                ),
                (
                    rect.left_center().distance(pointer),
                    TabDestination::Split(Split::Left),
                    Rect::everything_left_of(center.x),
                ),
                (
                    rect.right_center().distance(pointer),
                    TabDestination::Split(Split::Right),
                    Rect::everything_right_of(center.x),
                ),
                (
                    rect.center_top().distance(pointer),
                    TabDestination::Split(Split::Above),
                    Rect::everything_above(center.y),
                ),
                (
                    rect.center_bottom().distance(pointer),
                    TabDestination::Split(Split::Below),
                    Rect::everything_below(center.y),
                ),
            ],
            AllowedSplits::LeftRightOnly => vec![
                (
                    center.distance(pointer),
                    TabDestination::Append,
                    Rect::EVERYTHING,
                ),
                (
                    rect.left_center().distance(pointer),
                    TabDestination::Split(Split::Left),
                    Rect::everything_left_of(center.x),
                ),
                (
                    rect.right_center().distance(pointer),
                    TabDestination::Split(Split::Right),
                    Rect::everything_right_of(center.x),
                ),
            ],
            AllowedSplits::TopBottomOnly => vec![
                (
                    rect.center_top().distance(pointer),
                    TabDestination::Split(Split::Above),
                    Rect::everything_above(center.y),
                ),
                (
                    rect.center_bottom().distance(pointer),
                    TabDestination::Split(Split::Below),
                    Rect::everything_below(center.y),
                ),
            ],
            AllowedSplits::None => vec![(
                center.distance(pointer),
                TabDestination::Append,
                Rect::EVERYTHING,
            )],
        };

        let (_, tab_dst, overlay) = pts
            .into_iter()
            .min_by(|(lhs, ..), (rhs, ..)| lhs.total_cmp(rhs))
            .unwrap();

        let overlay = rect.intersect(overlay);
        draw_drop_rect(overlay, ui, style);

        tab_dst
    }
}

//draws one of the Tab drop destination icons inside "rect", which one you get is specified by "is_top_bottom"
fn button_ui(
    rect: Rect,
    ui: &mut Ui,
    mouse_pos: Pos2,
    style: &Style,
    is_top_bottom: Option<bool>,
) -> bool {
    let visuals = ui.style().visuals.widgets.noninteractive;
    let painter = ui.painter();
    painter.rect_stroke(rect, 0.0, visuals.bg_stroke);
    let rect = rect.shrink(10.0);
    painter.rect_stroke(rect, 0.0, visuals.fg_stroke);
    let rim = { Rect::from_two_pos(rect.min, rect.lerp_inside(vec2(1.0, 0.1))) };
    painter.rect(rim, 0.0, visuals.fg_stroke.color, Stroke::NONE);

    if let Some(top_bottom) = is_top_bottom {
        let list = dashed_line_alphas();
        for line in list.chunks(2) {
            let start = rect.lerp_inside(lerp_vec(top_bottom, line[0]));
            let end = rect.lerp_inside(lerp_vec(top_bottom, line[1]));
            painter.line_segment([start, end], visuals.fg_stroke);
        }
    }
    let over = rect.contains(mouse_pos);
    if over {
        painter.rect_filled(rect, 0.0, style.selection_color);
    }
    over
}

//a bunch of lerp alphas describing the 4 dashed lines on the tab destination icons
#[inline(always)]
const fn dashed_line_alphas() -> &'static [f32] {
    &[
        0.0625, 0.1875, 0.3125, 0.4375, 0.5625, 0.6875, 0.8125, 0.9375,
    ]
}

#[inline(always)]
const fn lerp_vec(top_bottom: bool, alpha: f32) -> Vec2 {
    if top_bottom {
        vec2(alpha, 0.5)
    } else {
        vec2(0.5, alpha)
    }
}

//this only draws the rect describing where a tab will be dropped
#[inline(always)]
fn draw_drop_rect(rect: Rect, ui: &mut Ui, style: &Style) {
    let id = Id::new("overlay");
    let layer_id = LayerId::new(Order::Foreground, id);
    let painter = ui.ctx().layer_painter(layer_id);
    painter.rect_filled(rect, 0.0, style.selection_color);
}
