//! # `egui_dock`: docking support for `egui`
//!
//! Credit goes to @Iain-dono for implementing the actual library.
//!
//! This fork aims to provide documentation and further development if necessary.
//!
//! ## Usage
//!
//! First, construct the initial tree:
//!
//! ```rust
//! use egui::{Color32, RichText, style::Margin};
//! use egui_dock::{TabBuilder, Tree, WithTitle};
//!
//! let tab1 = TabBuilder::default()
//!     .title(RichText::new("Tab 1").color(Color32::BLUE))
//!     .content(|ui| {
//!         ui.label("Tab 1");
//!     })
//!     .build();
//! let tab2 = TabBuilder::default()
//!     .title("Tab 2")
//!     .inner_margin(Margin::same(4.0))
//!     .content(|ui| {
//!         ui.label("Tab 2");
//!     })
//!     .build();
//! let mut tree = Tree::new(vec![tab1, tab2]);
//! ```
//!
//! Then, you can show the tree.
//!
//! ```rust
//! let id = ui.id();
//! egui_dock::show(&mut ui, id, &style, &mut tree);
//! ```

mod style;
mod tab;
mod tree;
mod utils;

pub use self::tab::{Tab, TabBuilder, WithTitle};
pub use self::tree::{Node, NodeIndex, Split, Tree};
pub use style::{Style, StyleBuilder};

use egui::*;
use utils::*;

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
            .min_by(|(_, lhs), (_, rhs)| lhs.total_cmp(rhs))
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
pub fn show(ui: &mut Ui, id: Id, style: &Style, tree: &mut Tree) {
    let mut state = State::load(ui.ctx(), id);
    let mut rect = ui.max_rect();

    if let Some(margin) = style.padding {
        rect.min += margin.left_top();
        rect.max -= margin.right_bottom();
        ui.painter().rect(
            rect,
            margin.top,
            style.separator_color,
            Stroke::new(margin.top, style.border_color),
        );
    }

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

                tree[tree_index.left()].set_rect(left);
                tree[tree_index.right()].set_rect(right);
            }
            Node::Vertical { fraction, rect } => {
                let rect = expand_to_pixel(*rect, pixels_per_point);

                let (bottom, _separator, top) = style.vsplit(ui, fraction, rect);

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
                    ui.painter().rect_filled(
                        tabbar,
                        style.tab_rounding,
                        style.tab_bar_background_color,
                    );

                    let a = pos2(tabbar.min.x, tabbar.max.y - px);
                    let b = pos2(tabbar.max.x, tabbar.max.y - px);
                    ui.painter()
                        .line_segment([a, b], (px, style.tab_outline_color));

                    let mut ui = ui.child_ui(tabbar, Default::default());
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                    ui.horizontal(|ui| {
                        for (tab_index, tab) in tabs.iter().enumerate() {
                            let id = Id::new((tree_index, tab_index, "tab"));
                            let is_being_dragged = ui.memory().is_being_dragged(id);

                            let is_active = *active == tab_index || is_being_dragged;
                            let label = tab.title.clone();

                            if is_being_dragged {
                                let layer_id = LayerId::new(Order::Tooltip, id);
                                let response = ui
                                    .with_layer_id(layer_id, |ui| {
                                        style.tab_title(
                                            ui,
                                            label.clone(),
                                            is_active,
                                            is_being_dragged,
                                        )
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
                                let response =
                                    style.tab_title(ui, label, is_active, is_being_dragged);
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

                    ui.painter()
                        .rect_filled(rect, 0.0, style.tab_background_color);

                    let mut ui = ui.child_ui(rect, Default::default());
                    tab.ui(&mut ui);
                }

                let is_being_dragged = ui.memory().is_anything_being_dragged();
                if is_being_dragged && full_response.hovered() {
                    hover_data = ui.input().pointer.hover_pos().map(|pointer| HoverData {
                        rect,
                        dst: tree_index,
                        tabs: tabs_response.hovered().then_some(tabs_response.rect),
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

            if src != dst || tree[dst].tabs_count() > 1 {
                painter.rect_filled(helper, 0.0, style.selection_color);
            }

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
