//! # `egui_dock`: docking support for `egui`
//!
//! Credit goes to @Iain-dono for implementing the actual library.
//!
//! This fork aims to provide documentation and further development if necessary.
//!
//! ## Usage
//!
//! First, create your context type and your tab widget:
//!
//! ```rust
//! use egui::{Frame, Ui, style::Margin};
//! use egui_dock::Tab;
//!
//! struct MyContext;
//!
//! struct MyTab {
//!     text: String,
//! }
//!
//! impl MyTab {
//!     fn new(text: impl ToString) -> Self {
//!         Self {
//!             text: text.to_string(),
//!         }
//!     }
//! }
//!
//! impl Tab<MyContext> for MyTab {
//!     fn title(&self) -> &str {
//!         &self.title
//!     }
//!
//!     fn ui(&mut self, ui: &mut Ui, _ctx: &mut MyContext) {
//!         let margin = Margin::same(4.0);
//!
//!         Frame::none().inner_margin(margin).show(ui, |ui| {
//!             ui.label(&self.text);
//!         });
//!     }
//! }
//! ```
//!
//! Then construct the initial tree using your tab widget:
//!
//! ```rust
//! use egui_dock::{NodeIndex, Tree};
//!
//! let tab1 = Box::new(MyTab::new("Tab 1"));
//! let tab2 = Box::new(MyTab::new("Tab 2"));
//! let tab3 = Box::new(MyTab::new("Tab 3"));
//! let tab4 = Box::new(MyTab::new("Tab 4"));
//! let tab5 = Box::new(MyTab::new("Tab 5"));
//!
//! let mut tree = Tree::new(vec![tab1, tab2]);
//!
//! // You can modify the tree in runtime
//! let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec![tab3]);
//! let [_, _] = tree.split_below(a, 0.7, vec![tab4]);
//! let [_, _] = tree.split_below(b, 0.5, vec![tab5]);
//! ```
//!
//! Finally, you can show the tree.
//!
//! ```rust
//! let id = ui.id();
//! egui_dock::show(&mut ui, id, style, tree, context);
//! ```

mod tab;
mod tree;

pub use self::tab::{Tab, TabDowncast};
pub use self::tree::{Node, NodeIndex, Split, Tree};

use egui::style::Margin;
use egui::*;

struct HoverData {
    rect: Rect,
    tabs: Option<Rect>,
    dst: NodeIndex,
    pointer: Pos2,
}

impl HoverData {
    fn resolve(&self) -> (Option<Split>, Rect) {
        if let Some(tabs) = self.tabs {
            return (None, tabs);
        }

        let (rect, pointer) = (self.rect, self.pointer);

        let center = rect.center();
        let pts = [
            center.distance(pointer),
            rect.left_center().distance(pointer),
            rect.right_center().distance(pointer),
            rect.center_top().distance(pointer),
            rect.center_bottom().distance(pointer),
        ];

        let position = pts
            .into_iter()
            .enumerate()
            .min_by(|(_, lhs), (_, rhs)| total_cmp(lhs, rhs))
            .map(|(idx, _)| idx)
            .unwrap();

        let (target, other) = match position {
            0 => (None, Rect::EVERYTHING),
            1 => (Some(Split::Left), Rect::everything_left_of(center.x)),
            2 => (Some(Split::Right), Rect::everything_right_of(center.x)),
            3 => (Some(Split::Above), Rect::everything_above(center.y)),
            4 => (Some(Split::Below), Rect::everything_below(center.y)),
            _ => unreachable!(),
        };

        (target, rect.intersect(other))
    }
}

#[derive(Clone, Debug, Default)]
struct State {
    drag_start: Option<Pos2>,
}

impl State {
    pub fn load(ctx: &Context, id: Id) -> Self {
        ctx.data().get_temp(id).unwrap_or(Self { drag_start: None })
    }

    fn store(self, ctx: &Context, id: Id) {
        ctx.data().insert_temp(id, self);
    }
}

/// Shows the docking hierarchy inside a `Ui`.
pub fn show<Ctx>(
    ui: &mut Ui,
    id: Id,
    style: &Style,
    tree: &mut Tree<Ctx>,
    context: &mut Ctx,
) {
    let mut state = State::load(ui.ctx(), id);

    let mut rect = ui.max_rect();
    if tree.is_empty() || tree[NodeIndex::root()].is_none() {
        //ui.painter().rect_filled(rect, 0.0, style.background);
        // TODO: splash screen here?
        return;
    }

    let rect = {
        rect.min += style.padding.left_top() + style.padding.left_top();
        rect.max -= style.padding.right_bottom() + style.padding.right_bottom();
        rect
    };

    tree[NodeIndex::root()].set_rect(rect);

    let mut drag_data = None;
    let mut hover_data = None;

    let pixels_per_point = ui.ctx().pixels_per_point();
    let px = pixels_per_point.recip();

    for tree_index in 0..tree.len() {
        let tree_index = NodeIndex(tree_index);
        match &mut tree[tree_index] {
            Node::None => (),

            Node::Horizontal { fraction, rect } => {
                let rect = expand_to_pixel(*rect, pixels_per_point);

                let (left, _separator, right) = style.hsplit(ui, fraction, rect);
                //ui.painter().rect_filled(separator, 0.0, style.background);

                tree[tree_index.left()].set_rect(left);
                tree[tree_index.right()].set_rect(right);
            }

            Node::Vertical { fraction, rect } => {
                let rect = expand_to_pixel(*rect, pixels_per_point);

                let (bottom, _separator, top) = style.vsplit(ui, fraction, rect);
                //ui.painter().rect_filled(separator, 0.0, style.background);

                tree[tree_index.left()].set_rect(bottom);
                tree[tree_index.right()].set_rect(top);
            }

            Node::Leaf {
                rect,
                tabs,
                active,
                viewport,
            } => {
                let rect = *rect;
                ui.set_clip_rect(rect);

                let height_topbar = 24.0;

                let bottom_y = rect.min.y + height_topbar;
                let tabbar = rect.intersect(Rect::everything_above(bottom_y));

                let full_response = ui.allocate_rect(rect, Sense::hover());
                let tabs_response = ui.allocate_rect(tabbar, Sense::hover());

                // tabs
                ui.scope(|ui| {
                    ui.painter()
                        .rect_filled(tabbar, style.tab_rounding, style.tab_bar_background);

                    let a = pos2(tabbar.min.x, tabbar.max.y - px);
                    let b = pos2(tabbar.max.x, tabbar.max.y - px);
                    ui.painter().line_segment([a, b], (px, style.tab_outline));

                    let mut ui = ui.child_ui(tabbar, Default::default());
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                    ui.horizontal(|ui| {
                        for (tab_index, tab) in tabs.iter().enumerate() {
                            let id = Id::new((tree_index, tab_index, "tab"));
                            let is_being_dragged = ui.memory().is_being_dragged(id);

                            let is_active = *active == tab_index || is_being_dragged;
                            let label = tab.title().to_string();

                            if is_being_dragged {
                                let layer_id = LayerId::new(Order::Tooltip, id);
                                let response = ui
                                    .with_layer_id(layer_id, |ui| {
                                        style.tab_title(ui, label, is_active)
                                    })
                                    .response;

                                let sense = egui::Sense::click_and_drag();
                                let response = ui
                                    .interact(response.rect, id, sense)
                                    .on_hover_cursor(CursorIcon::Grabbing);

                                if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                    let center = response.rect.center();
                                    let start = state.drag_start.unwrap_or(center);

                                    let delta = pointer_pos - start;
                                    if delta.x.abs() > 30.0 || delta.y.abs() > 6.0 {
                                        ui.ctx().translate_layer(layer_id, delta);

                                        drag_data = Some((tree_index, tab_index));
                                    }
                                }

                                if response.clicked() {
                                    *active = tab_index;
                                }
                            } else {
                                let response = style.tab_title(ui, label, is_active);
                                let sense = Sense::click_and_drag();
                                let response = ui.interact(response.rect, id, sense);
                                if response.drag_started() {
                                    state.drag_start = response.hover_pos();
                                }
                            }
                        }
                    });
                });

                // tab body
                if let Some(tab) = tabs.get_mut(*active) {
                    let top_y = rect.min.y + height_topbar;
                    let rect = rect.intersect(Rect::everything_below(top_y));
                    let rect = expand_to_pixel(rect, pixels_per_point);

                    *viewport = rect;

                    ui.painter().rect_filled(rect, 0.0, style.background);

                    let mut ui = ui.child_ui(rect, Default::default());
                    tab.ui(&mut ui, context);
                }

                let is_being_dragged = ui.memory().is_anything_being_dragged();
                if is_being_dragged && full_response.hovered() {
                    hover_data = ui.input().pointer.hover_pos().map(|pointer| HoverData {
                        rect,
                        dst: tree_index,
                        tabs: tabs_response.hovered().then(|| tabs_response.rect),
                        pointer,
                    });
                }
            }
        }
    }

    if let (Some((src, tab_index)), Some(hover)) = (drag_data, hover_data) {
        let dst = hover.dst;

        if tree[src].is_leaf() && tree[dst].is_leaf() {
            let (target, helper) = hover.resolve();

            let id = Id::new("helper");
            let layer_id = LayerId::new(Order::Foreground, id);
            let painter = ui.ctx().layer_painter(layer_id);
            painter.rect_filled(helper, 0.0, style.selection);

            if ui.input().pointer.any_released() {
                if let Node::Leaf { active, .. } = &mut tree[src] {
                    if *active >= tab_index {
                        *active = active.saturating_sub(1);
                    }
                }

                let tab = tree[src].remove_tab(tab_index).unwrap();

                if let Some(target) = target {
                    tree.split(dst, target, 0.5, Node::leaf(tab));
                } else {
                    tree[dst].append_tab(tab);
                }

                tree.remove_empty_leaf();
                for node in tree.iter_mut() {
                    if let Node::Leaf { tabs, active, .. } = node {
                        if *active >= tabs.len() {
                            *active = 0;
                        }
                    }
                }
            }
        }
    }

    state.store(ui.ctx(), id);
}

fn expand_to_pixel(mut rect: Rect, ppi: f32) -> egui::Rect {
    rect.min = map_to_pixel_pos(rect.min, ppi, f32::floor);
    rect.max = map_to_pixel_pos(rect.max, ppi, f32::ceil);
    rect
}

fn map_to_pixel_pos(mut pos: Pos2, ppi: f32, map: fn(f32) -> f32) -> egui::Pos2 {
    pos.x = map_to_pixel(pos.x, ppi, map);
    pos.y = map_to_pixel(pos.y, ppi, map);
    pos
}

#[inline(always)]
fn map_to_pixel(point: f32, ppi: f32, map: fn(f32) -> f32) -> f32 {
    map(point * ppi) / ppi
}

/// Specifies the look and feel of egui_dock.
pub struct Style {
    pub padding: Margin,

    pub background: Color32,
    pub selection: Color32,
    pub separator_size: f32,
    pub separator_extra: f32,

    pub tab_bar_background: Color32,

    pub tab_text: Color32,
    pub tab_outline: Color32,
    pub tab_rounding: Rounding,
    pub tab_background: Color32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            padding: Margin {
                left: 3.0,
                right: 3.0,
                top: 2.0,
                bottom: 2.0,
            },

            background: Color32::DARK_GREEN,
            selection: Color32::from_rgb_additive(0, 92, 128),
            separator_size: 4.0,
            separator_extra: 100.0,

            tab_bar_background: Color32::RED,

            tab_text: Color32::WHITE,
            tab_outline: Color32::RED,
            tab_background: Color32::GREEN,
            tab_rounding: Rounding {
                ne: 4.0,
                nw: 4.0,
                sw: 0.0,
                se: 0.0,
            },
        }
    }
}

impl Style {
    /// Derives relevant fields from `egui::Style` and sets the remaining fields to their default values.
    ///
    /// Fields overwritten by `egui::Style` are: `selection`, `background`, `tab_bar_background`, `tab_text`,
    /// `tab_outline`, and `tab_background`.
    pub fn from_egui(style: &egui::Style) -> Self {
        Self {
            selection: style.visuals.selection.bg_fill.linear_multiply(0.5),

            background: style.visuals.window_fill(),
            tab_bar_background: style.visuals.faint_bg_color,

            tab_text: style.visuals.widgets.active.fg_stroke.color,
            tab_outline: style.visuals.widgets.active.bg_stroke.color,
            tab_background: style.visuals.widgets.active.bg_fill,

            ..Self::default()
        }
    }

    fn hsplit(&self, ui: &mut Ui, fraction: &mut f32, rect: Rect) -> (Rect, Rect, Rect) {
        let pixels_per_point = ui.ctx().pixels_per_point();

        let mut separator = rect;

        let midpoint = rect.min.x + rect.width() * *fraction;
        separator.min.x = midpoint - self.separator_size * 0.5;
        separator.max.x = midpoint + self.separator_size * 0.5;

        let response = ui
            .allocate_rect(separator, Sense::click_and_drag())
            .on_hover_cursor(CursorIcon::ResizeHorizontal);

        {
            let delta = response.drag_delta().x;
            let range = rect.max.x - rect.min.x;
            let min = (self.separator_extra / range).min(1.0);
            let max = 1.0 - min;
            let (min, max) = (min.min(max), max.max(min));
            *fraction = (*fraction + delta / range).clamp(min, max);
        }

        let midpoint = rect.min.x + rect.width() * *fraction;
        separator.min.x = map_to_pixel(
            midpoint - self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );
        separator.max.x = map_to_pixel(
            midpoint + self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );

        (
            rect.intersect(Rect::everything_right_of(separator.max.x)),
            separator,
            rect.intersect(Rect::everything_left_of(separator.min.x)),
        )
    }

    fn vsplit(&self, ui: &mut Ui, fraction: &mut f32, rect: Rect) -> (Rect, Rect, Rect) {
        let pixels_per_point = ui.ctx().pixels_per_point();

        let mut separator = rect;

        let midpoint = rect.min.y + rect.height() * *fraction;
        separator.min.y = midpoint - self.separator_size * 0.5;
        separator.max.y = midpoint + self.separator_size * 0.5;

        let response = ui
            .allocate_rect(separator, Sense::click_and_drag())
            .on_hover_cursor(CursorIcon::ResizeVertical);

        {
            let delta = response.drag_delta().y;
            let range = rect.max.y - rect.min.y;

            let min = (self.separator_extra / range).min(1.0);
            let max = 1.0 - min;
            let (min, max) = (min.min(max), max.max(min));
            *fraction = (*fraction + delta / range).clamp(min, max);
        }

        let midpoint = rect.min.y + rect.height() * *fraction;
        separator.min.y = map_to_pixel(
            midpoint - self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );
        separator.max.y = map_to_pixel(
            midpoint + self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );

        (
            rect.intersect(Rect::everything_above(separator.min.y)),
            separator,
            rect.intersect(Rect::everything_below(separator.max.y)),
        )
    }

    fn tab_title(&self, ui: &mut Ui, label: String, active: bool) -> Response {
        let px = ui.ctx().pixels_per_point().recip();
        let rounding = self.tab_rounding;

        let font_id = FontId::proportional(14.0);
        let galley = ui.painter().layout_no_wrap(label, font_id, self.tab_text);

        let offset = egui::vec2(8.0, 0.0);
        let text_size = galley.size();

        let mut desired_size = text_size + offset * 2.0;
        desired_size.y = 24.0;

        let (rect, response) = ui.allocate_at_least(desired_size, Sense::hover());
        let response = response.on_hover_cursor(CursorIcon::PointingHand);

        if active {
            let mut tab = rect;

            tab.min.x -= px;
            tab.max.x += px;
            ui.painter().rect_filled(tab, rounding, self.tab_outline);

            tab.min.x += px;
            tab.max.x -= px;
            tab.min.y += px;
            ui.painter().rect_filled(tab, rounding, self.background);
        }

        let pos = Align2::LEFT_TOP
            .anchor_rect(rect.shrink2(vec2(8.0, 5.0)))
            .min;

        ui.painter().galley(pos, galley);

        response
    }
}

fn total_cmp(lhs: &f32, rhs: &f32) -> std::cmp::Ordering {
    let mut lhs = lhs.to_bits() as i32;
    let mut rhs = rhs.to_bits() as i32;
    lhs ^= (((lhs >> 31) as u32) >> 1) as i32;
    rhs ^= (((rhs >> 31) as u32) >> 1) as i32;
    lhs.cmp(&rhs)
}
