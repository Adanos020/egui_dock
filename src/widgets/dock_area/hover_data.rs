use std::{
    ops::BitOrAssign,
    time::{Duration, Instant},
};

use crate::{AllowedSplits, NodeIndex, Split, Style, SurfaceIndex, TabDestination, TabIndex};
use egui::{vec2, Context, Id, LayerId, Order, Pos2, Rect, Stroke, Ui, Vec2};

#[derive(Debug, Clone)]
pub(super) struct HoverData {
    /// rect of the hovered element
    pub rect: Rect,
    ///
    pub tab: Option<Rect>,
    pub dst: DropPosition,
    pub pointer: Pos2,
    ///is some when the pointer is over rect, instant holds when the lock was last active
    pub locked: Option<Instant>,
}
#[derive(Debug, Clone)]
pub(super) enum DropPosition {
    Surface(SurfaceIndex),
    Node(SurfaceIndex, NodeIndex),
    Tab(SurfaceIndex, NodeIndex, TabIndex),
}
impl DropPosition {
    pub(super) fn break_down(&self) -> (SurfaceIndex, Option<NodeIndex>) {
        match self {
            DropPosition::Surface(surface) => (*surface, None),
            DropPosition::Node(surface, node) => (*surface, Some(*node)),
            //NOTE: TabIndex here is only used by `resolve`, since its used to factor the `TabDestination`
            DropPosition::Tab(surface, node, _) => (*surface, Some(*node)),
        }
    }
    pub(super) fn surface_index(&self) -> SurfaceIndex {
        match self {
            DropPosition::Surface(surface)
            | DropPosition::Node(surface, _)
            | DropPosition::Tab(surface, _, _) => *surface,
        }
    }
    pub(super) fn is_surface(&self) -> bool {
        match self {
            DropPosition::Surface(_) => true,
            _ => false,
        }
    }
}
impl HoverData {
    //determines if the hoverdata implies we're hovering over a tab or the tab title bar
    pub(super) fn is_on_title_bar(&self) -> bool {
        self.tab.is_some()
    }

    //resolve a TabDestination for whatever is hovered
    pub(super) fn resolve(
        &mut self,
        ui: &mut Ui,
        style: &Style,
        allowed_splits: AllowedSplits,
        is_window: bool,
    ) -> TabDestination {
        let allowed_splits = allowed_splits
            & if self.dst.is_surface() {
                AllowedSplits::None
            } else {
                AllowedSplits::All
            };
        if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
            self.pointer = pointer;
        }
        
        if self.is_on_title_bar() {
            self.resolve_traditional(ui, style, allowed_splits)
        } else {
            self.resolve_icon_based(ui, style, allowed_splits, is_window)
        }

    }

    fn resolve_icon_based(
        &mut self,
        ui: &mut Ui,
        style: &Style,
        allowed_splits: AllowedSplits,
        is_window: bool,
    ) -> TabDestination {
        assert!(!self.is_on_title_bar());
        draw_highlight_rect(self.rect, ui, style);
        let mut hovering_buttons = false;
        let total_button_spacing = style.overlay.button_padding * 2.0;
        let (rect, pointer) = (self.rect, self.pointer);
        let rect = rect.shrink(style.overlay.button_padding);
        let shortest_side = ((rect.width() - total_button_spacing) / 3.0)
            .min((rect.height() - total_button_spacing) / 3.0)
            .min(style.overlay.max_button_size);
        let mut offset_vector = vec2(0.0, shortest_side + style.overlay.button_padding);

        let mut destination = None;

        let center = rect.center();

        if !is_window {
            if button_ui(
                Rect::from_center_size(center, Vec2::splat(shortest_side)),
                ui,
                &mut hovering_buttons,
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
                        &mut hovering_buttons,
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
        let hovering_rect = self.rect.contains(pointer);

        self.update_lock(hovering_rect, hovering_buttons, style, ui.ctx());
        destination.unwrap_or(TabDestination::Window(self.pointer))
    }

    fn resolve_traditional(
        &self,
        ui: &mut Ui,
        style: &Style,
        allowed_splits: AllowedSplits,
    ) -> TabDestination {
        if let Some(rect) = self.tab {
            draw_drop_rect(rect, ui, style);

            return match self.dst {
                DropPosition::Surface(_) => TabDestination::Append,
                DropPosition::Node(_, _) => TabDestination::Append,
                DropPosition::Tab(_, _, tab_index) => TabDestination::Insert(tab_index),
            };
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
    fn update_lock(
        &mut self,
        on_node_rect: bool,
        continue_locking: bool,
        style: &Style,
        ctx: &Context,
    ) {
        if on_node_rect && self.locked.is_none() {
            self.locked = Some(Instant::now());
        } else {
            let keep_lock = if let Some(lock_time) = &mut self.locked {
                if continue_locking {
                    *lock_time = Instant::now();
                }
                let window_hold = {
                    if self.dst.surface_index() != SurfaceIndex::root() {
                        ctx.request_repaint();
                        self.is_locked(style, ctx)
                    } else {
                        false
                    }
                };
                on_node_rect || window_hold
            } else {
                true
            };
            if !keep_lock {
                self.locked = None;
            }
        }
    }
    pub(super) fn is_locked(&self, style: &Style, ctx: &Context) -> bool {
        if let Some(lock_time) = &self.locked {
            let elapsed = lock_time.elapsed().as_secs_f32();
            ctx.request_repaint_after(Duration::from_secs_f32(
                (style.overlay.max_preference_time - elapsed).max(0.0),
            ));
            elapsed < style.overlay.max_preference_time
        } else {
            false
        }
    }
}
fn draw_highlight_rect(rect: Rect, ui: &mut Ui, style: &Style) {
    ui.painter().rect(
        rect.expand(style.overlay.hovered_leaf_highlight.expansion),
        style.overlay.hovered_leaf_highlight.rounding,
        style.overlay.hovered_leaf_highlight.color,
        style.overlay.hovered_leaf_highlight.stroke,
    )
}

//draws one of the Tab drop destination icons inside "rect", which one you get is specified by "is_top_bottom"
fn button_ui(
    rect: Rect,
    ui: &mut Ui,
    lock: &mut bool,
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
    let over = rect
        .expand(style.overlay.interact_expansion)
        .contains(mouse_pos);
    if over && !*lock {
        painter.rect_filled(rect, 0.0, style.selection_color);
    }
    lock.bitor_assign(over);
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
