//! # `egui_dock`: docking support for `egui`
//!
//! Credit goes to @lain-dono for implementing the actual library.
//!
//! This fork aims to provide documentation and further development if necessary.
//!
//! ## Usage
//!
//! The library is centered around the [`Tree`].
//! It stores the layout of [`Node`]s which contains tabs.
//!
//! [`Tree`] is generic (`Tree<Tab>`) so you can use any data to represent a tab.
//! You show the tabs using [`DockArea`] and specify how they are shown
//! by implementing [`TabViewer`].
//!
//! ```rust
//! use egui_dock::{NodeIndex, Tree};
//!
//! struct MyTabs {
//!     tree: Tree<String>
//! }
//!
//! impl MyTabs {
//!     pub fn new() -> Self {
//!         let tab1 = "tab1".to_string();
//!         let tab2 = "tab2".to_string();
//!
//!         let mut tree = Tree::new(vec![tab1]);
//!         tree.split_left(NodeIndex::root(), 0.20, vec![tab2]);
//!
//!         Self { tree }
//!     }
//!
//!     fn ui(&mut self, ui: &mut egui::Ui) {
//!         let style = egui_dock::Style::from_egui(ui.style().as_ref());
//!         egui_dock::DockArea::new(&mut self.tree)
//!             .style(style)
//!             .show_inside(ui, &mut TabViewer {});
//!     }
//! }
//!
//! struct TabViewer {}
//!
//! impl egui_dock::TabViewer for TabViewer {
//!     type Tab = String;
//!
//!     fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
//!         ui.label(format!("Content of {tab}"));
//!     }
//!
//!     fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
//!         (&*tab).into()
//!     }
//! }
//!
//! # let mut my_tabs = MyTabs::new();
//! # egui::__run_test_ctx(|ctx| {
//! #     egui::CentralPanel::default().show(ctx, |ui| my_tabs.ui(ui));
//! # });
//! ```

use egui::style::Margin;
use egui::*;

use tree::TabIndex;
use utils::*;

pub use crate::{
    dynamic_tab::{DynamicTabViewer, DynamicTree, Tab, TabBuilder},
    style::{Style, StyleBuilder},
    tree::{Node, NodeIndex, Split, Tree},
};
pub use egui;

mod dynamic_tab;
mod style;
mod tree;
mod utils;

// ----------------------------------------------------------------------------

struct HoverData {
    rect: Rect,
    tabs: Option<Rect>,
    tab: Option<(Rect, TabIndex)>,
    dst: NodeIndex,
    pointer: Pos2,
}

impl HoverData {
    fn resolve(&self) -> (Option<Split>, Rect, Option<TabIndex>) {
        if let Some(tab) = self.tab {
            return (None, tab.0, Some(tab.1));
        }
        if let Some(tabs) = self.tabs {
            return (None, tabs, None);
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

        (target, rect.intersect(other), None)
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

// ----------------------------------------------------------------------------

/// How we view a tab when its in a [`Tree`].
pub trait TabViewer {
    type Tab;

    /// Actual tab content.
    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab);

    /// The title to be displayed.
    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText;

    /// This is called when the tabs close button is pressed.
    ///
    /// Returns `true` if the tab should close immediately, `false` otherwise.
    ///
    /// NOTE if returning false `ui` will still be called once more if this tab is active.
    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        true
    }

    /// This is called when the tabs add button is pressed.
    ///
    /// This requires the dock style's `show_add_buttons` to be `true`.
    ///
    /// The `_node` specifies which `Node` or split of the tree that this
    /// particular add button was pressed on.
    fn on_add(&mut self, _node: NodeIndex) {}

    /// This is called every frame after `ui` is called (if the tab is active).
    ///
    /// Returns `true` if the tab should be forced to close, `false` otherwise.
    ///
    /// In the event this function returns true the tab will be removed without calling `on_close`.
    fn force_close(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }

    /// Sets the margins between tab's borders and its contents.
    fn inner_margin(&self) -> Margin {
        Margin::same(4.0)
    }

    /// Whether the tab will be cleared with the color specified in [`Style::tab_background_color`]
    fn clear_background(&self, _tab: &Self::Tab) -> bool {
        true
    }
}

// ----------------------------------------------------------------------------

/// Stores the layout and position of all its tabs
///
/// Keeps track of the currently focused leaf and currently active tabs
pub struct DockArea<'tree, Tab> {
    id: Id,
    tree: &'tree mut Tree<Tab>,
    style: Option<Style>,
}

impl<'tree, Tab> DockArea<'tree, Tab> {
    pub fn new(tree: &'tree mut Tree<Tab>) -> DockArea<'tree, Tab> {
        Self {
            id: Id::new("egui_dock::DockArea"),
            tree,
            style: None,
        }
    }

    /// Sets the [DockArea] id. Useful if you have more than one [DockArea].
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    /// Sets the dock area style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Shows the docking area.
    pub fn show(self, ctx: &Context, tab_viewer: &mut impl TabViewer<Tab = Tab>) {
        let layer_id = LayerId::background();
        let max_rect = ctx.available_rect();
        let clip_rect = ctx.available_rect();

        let mut ui = Ui::new(ctx.clone(), layer_id, self.id, max_rect, clip_rect);
        self.show_inside(&mut ui, tab_viewer);
    }

    /// Shows the docking hierarchy inside a `Ui`.
    pub fn show_inside(self, ui: &mut Ui, tab_viewer: &mut impl TabViewer<Tab = Tab>) {
        let style = self
            .style
            .unwrap_or_else(|| Style::from_egui(ui.style().as_ref()));

        let mut state = State::load(ui.ctx(), self.id);
        let mut rect = ui.max_rect();

        if let Some(margin) = style.dock_area_padding {
            rect.min += margin.left_top();
            rect.max -= margin.right_bottom();
            ui.painter().rect(
                rect,
                margin.top,
                style.separator_color,
                Stroke::new(margin.top, style.border_color),
            );
        }

        if self.tree.is_empty() {
            ui.allocate_rect(rect, Sense::hover());
            return;
        }

        self.tree[NodeIndex::root()].set_rect(rect);

        let mut drag_data = None;
        let mut hover_data = None;

        let pixels_per_point = ui.ctx().pixels_per_point();
        let px = pixels_per_point.recip();

        let focused = self.tree.focused_leaf();

        let mut to_remove = Vec::new();
        let mut new_focused = None;

        // Deal with Horizontal and Vertical nodes first
        for node_index in 0..self.tree.len() {
            let node_index = NodeIndex(node_index);
            let is_horizontal = self.tree[node_index].is_horizontal();
            if let Node::Horizontal { fraction, rect } | Node::Vertical { fraction, rect } =
                &mut self.tree[node_index]
            {
                let rect = expand_to_pixel(*rect, pixels_per_point);

                let (left, separator, right) = if is_horizontal {
                    style.hsplit(ui, fraction, rect)
                } else {
                    style.vsplit(ui, fraction, rect)
                };

                ui.painter()
                    .rect_filled(separator, Rounding::none(), style.separator_color);

                self.tree[node_index.left()].set_rect(left);
                self.tree[node_index.right()].set_rect(right);
            }
        }

        // Then process Leaf nodes
        for node_index in 0..self.tree.len() {
            let node_index = NodeIndex(node_index);
            if let Node::Leaf {
                rect,
                tabs,
                active,
                viewport,
            } = &mut self.tree[node_index]
            {
                let rect = *rect;
                ui.set_clip_rect(rect);

                let height_topbar = 24.0;

                let bottom_y = rect.min.y + height_topbar;
                let tabbar = rect.intersect(Rect::everything_above(bottom_y));

                let full_response = ui.allocate_rect(rect, Sense::hover());
                let tabs_response = ui.allocate_rect(tabbar, Sense::hover());
                let mut tab_hover_rect = None;

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
                        for (tab_index, tab) in tabs.iter_mut().enumerate() {
                            let id = self.id.with((node_index, tab_index, "tab"));
                            let tab_index = TabIndex(tab_index);
                            let is_being_dragged =
                                ui.memory().is_being_dragged(id) && style.tabs_are_draggable;

                            let is_active = *active == tab_index || is_being_dragged;
                            let label = tab_viewer.title(tab);

                            let response = if is_being_dragged {
                                let layer_id = LayerId::new(Order::Tooltip, id);
                                let response = ui
                                    .with_layer_id(layer_id, |ui| {
                                        style.tab_title(
                                            ui,
                                            label,
                                            is_active,
                                            is_active && Some(node_index) == focused,
                                            is_being_dragged,
                                            id,
                                        )
                                    })
                                    .response;

                                let sense = Sense::click_and_drag();
                                let response = ui
                                    .interact(response.rect, id, sense)
                                    .on_hover_cursor(CursorIcon::Grabbing);

                                if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                    let center = response.rect.center();
                                    let start = state.drag_start.unwrap_or(center);

                                    let delta = pointer_pos - start;
                                    if delta.x.abs() > 30.0 || delta.y.abs() > 6.0 {
                                        ui.ctx().translate_layer(layer_id, delta);

                                        drag_data = Some((node_index, tab_index));
                                    }
                                }

                                if response.clicked() {
                                    *active = tab_index;
                                    new_focused = Some(node_index);
                                }

                                if response.middle_clicked() && style.show_close_buttons {
                                    if tab_viewer.on_close(tab) {
                                        to_remove.push((node_index, tab_index));
                                    } else {
                                        *active = tab_index;
                                        new_focused = Some(node_index);
                                    }
                                }

                                response
                            } else {
                                let response = style.tab_title(
                                    ui,
                                    label,
                                    is_active && Some(node_index) == focused,
                                    is_active,
                                    is_being_dragged,
                                    id,
                                );

                                let sense = if response.1 {
                                    Sense::click()
                                } else {
                                    Sense::click_and_drag()
                                };

                                if response.2 {
                                    if tab_viewer.on_close(tab) {
                                        to_remove.push((node_index, tab_index));
                                    } else {
                                        *active = tab_index;
                                        new_focused = Some(node_index);
                                    }
                                }
                                let response = ui.interact(response.0.rect, id, sense);
                                if response.drag_started() {
                                    state.drag_start = response.hover_pos();
                                }

                                response
                            };
                            if state.drag_start.is_some() {
                                if let Some(pos) = ui.input().pointer.hover_pos() {
                                    if response.rect.contains(pos) {
                                        tab_hover_rect = Some((response.rect, tab_index));
                                    }
                                }
                            }
                        }

                        // Add button at the end of the tab bar
                        if style.show_add_buttons {
                            let id = self.id.with((node_index, "tab_add"));
                            let response = style.tab_plus(ui);

                            let response = ui.interact(response.rect, id, Sense::click());
                            if response.clicked() {
                                tab_viewer.on_add(node_index);
                            }
                        };
                    });
                });

                // tab body
                if let Some(tab) = tabs.get_mut(active.0) {
                    let top_y = rect.min.y + height_topbar;
                    let rect = rect.intersect(Rect::everything_below(top_y));
                    let rect = expand_to_pixel(rect, pixels_per_point);

                    *viewport = rect;

                    if ui.input().pointer.any_click() {
                        if let Some(pos) = ui.input().pointer.hover_pos() {
                            if rect.contains(pos) {
                                new_focused = Some(node_index);
                            }
                        }
                    }

                    if tab_viewer.clear_background(tab) {
                        ui.painter()
                            .rect_filled(rect, 0.0, style.tab_background_color);
                    }

                    let mut ui = ui.child_ui(rect, Default::default());
                    ui.push_id(node_index, |ui| {
                        ScrollArea::both()
                            .id_source(
                                self.id
                                    .with((tab_viewer.title(tab).text(), "egui_dock::Tab")),
                            )
                            .show(ui, |ui| {
                                Frame::none().inner_margin(tab_viewer.inner_margin()).show(
                                    ui,
                                    |ui| {
                                        let available_rect = ui.available_rect_before_wrap();
                                        ui.expand_to_include_rect(available_rect);
                                        tab_viewer.ui(ui, tab);
                                    },
                                );
                            });
                    });
                }

                let is_being_dragged = ui.memory().is_anything_being_dragged();
                if is_being_dragged && full_response.hovered() {
                    hover_data = ui.input().pointer.hover_pos().map(|pointer| HoverData {
                        rect,
                        dst: node_index,
                        tabs: tabs_response.hovered().then_some(tabs_response.rect),
                        tab: tab_hover_rect,
                        pointer,
                    });
                }

                for (tab_index, tab) in tabs.iter_mut().enumerate() {
                    if tab_viewer.force_close(tab) {
                        to_remove.push((node_index, TabIndex(tab_index)));
                    }
                }
            }
        }

        let mut emptied = 0;
        let mut last = (NodeIndex(usize::MAX), TabIndex(usize::MAX));
        for remove in to_remove.iter().rev() {
            if let Node::Leaf { tabs, active, .. } = &mut self.tree[remove.0] {
                tabs.remove(remove.1 .0);
                if remove.1 <= *active {
                    active.0 = active.0.saturating_sub(1);
                }
                if tabs.is_empty() {
                    emptied += 1;
                }
                if last.0 == remove.0 {
                    assert!(last.1 > remove.1)
                }
                last = *remove;
            } else {
                panic!();
            }
        }
        for _ in 0..emptied {
            self.tree.remove_empty_leaf()
        }

        if let Some(focused) = new_focused {
            self.tree.set_focused_node(focused);
        }

        if let (Some((src, tab_index)), Some(hover)) = (drag_data, hover_data) {
            let dst = hover.dst;

            if self.tree[src].is_leaf() && self.tree[dst].is_leaf() {
                let (target, helper, tap_pos) = hover.resolve();

                let id = Id::new("helper");
                let layer_id = LayerId::new(Order::Foreground, id);
                let painter = ui.ctx().layer_painter(layer_id);

                if src != dst || self.tree[dst].tabs_count() > 1 {
                    painter.rect_filled(helper, 0.0, style.selection_color);
                }

                if ui.input().pointer.any_released() {
                    if let Node::Leaf { active, .. } = &mut self.tree[src] {
                        if *active >= tab_index {
                            active.0 = active.0.saturating_sub(1);
                        }
                    }

                    let tab = self.tree[src].remove_tab(tab_index).unwrap();

                    if let Some(target) = target {
                        self.tree.split(dst, target, 0.5, Node::leaf(tab));
                    } else {
                        if let Some(index) = tap_pos {
                            self.tree[dst].insert_tab(index, tab);
                        } else {
                            self.tree[dst].append_tab(tab);
                        }
                        self.tree.set_focused_node(dst);
                    }

                    self.tree.remove_empty_leaf();
                    for node in self.tree.iter_mut() {
                        if let Node::Leaf { tabs, active, .. } = node {
                            if active.0 >= tabs.len() {
                                active.0 = 0;
                            }
                        }
                    }
                }
            }
        }

        state.store(ui.ctx(), self.id);
    }
}
