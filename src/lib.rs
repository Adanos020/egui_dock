#![warn(missing_docs)]

//! # `egui_dock`: docking support for `egui`
//!
//! Originally created by [@lain-dono](https://github.com/lain-dono), this library provides docking support for `egui`.
//! It lets you open and close tabs, freely move them around, insert them in selected parts of the [`DockArea`], and resize them.
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

#![forbid(unsafe_code)]

use egui::{
    style::Margin, vec2, CentralPanel, Color32, Context, CursorIcon, Frame, Id, LayerId, Order,
    Pos2, Rect, Rounding, ScrollArea, Sense, Stroke, Ui, WidgetText,
};

pub use crate::{
    style::{Style, StyleBuilder, TabAddAlign},
    tree::{Node, NodeIndex, Split, TabIndex, Tree},
};
pub use egui;

use std::fmt;
use utils::expand_to_pixel;

mod popup;
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
    #[inline(always)]
    pub fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|d| d.get_temp(id))
            .unwrap_or(Self { drag_start: None })
    }

    #[inline(always)]
    fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }
}

// ----------------------------------------------------------------------------

/// How to display a tab inside a [`Tree`].
pub trait TabViewer {
    /// The type of tab in which you can store state to be drawn in your tabs.
    type Tab;

    /// Actual tab content.
    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab);

    /// Content inside context_menu.
    fn context_menu(&mut self, _ui: &mut Ui, _tab: &mut Self::Tab) {}

    /// The title to be displayed.
    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText;

    /// Called after each tab button is shown, so you can add a tooltip, check for clicks, etc.
    fn on_tab_button(&mut self, _tab: &mut Self::Tab, _response: &egui::Response) {}

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

    /// Content of add_popup. Displays a popup under the add button. Useful for selecting
    /// what type of tab to add.
    ///
    /// This requires the dock style's `show_add_buttons` and `show_add_popup` to be `true`.
    fn add_popup(&mut self, _ui: &mut Ui, _node: NodeIndex) {}

    /// This is called every frame after `ui` is called (if the tab is active).
    ///
    /// Returns `true` if the tab should be forced to close, `false` otherwise.
    ///
    /// In the event this function returns true the tab will be removed without calling `on_close`.
    fn force_close(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }

    /// Sets the margins between tab's borders and its contents.
    fn inner_margin_override(&self, style: &Style) -> Margin {
        style.default_inner_margin
    }

    /// Whether the tab will be cleared with the color specified in [`Style::tab_background_color`]
    fn clear_background(&self, _tab: &Self::Tab) -> bool {
        true
    }
}

// ----------------------------------------------------------------------------

/// Displays a [`Tree`] in `egui`.
pub struct DockArea<'tree, Tab> {
    id: Id,
    tree: &'tree mut Tree<Tab>,
    style: Option<Style>,
}

impl<'tree, Tab> DockArea<'tree, Tab> {
    /// Creates a new [DockArea] from the provided [`Tree`].
    #[inline(always)]
    pub fn new(tree: &'tree mut Tree<Tab>) -> DockArea<'tree, Tab> {
        Self {
            id: Id::new("egui_dock::DockArea"),
            tree,
            style: None,
        }
    }

    /// Sets the [DockArea] id. Useful if you have more than one [DockArea].
    #[inline(always)]
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    /// Sets the dock area style.
    #[inline(always)]
    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Show the `DockArea` at the top level.
    ///
    /// This is the same as doing:
    /// ```
    /// # use egui_dock::{DockArea, Tree};
    /// # use egui::{CentralPanel, Frame};
    /// # struct TabViewer {}
    /// # impl egui_dock::TabViewer for TabViewer {
    /// #     type Tab = String;
    /// #     fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {}
    /// #     fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText { (&*tab).into() }
    /// # }
    /// # let mut tree: Tree<String> = Tree::new(vec![]);
    /// # let mut tab_viewer = TabViewer {};
    /// # egui::__run_test_ctx(|ctx| {
    /// CentralPanel::default()
    ///     .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
    ///     .show(ctx, |ui| {
    ///         DockArea::new(&mut tree).show_inside(ui, &mut tab_viewer);
    ///     });
    /// # });
    /// ```
    /// So you can't use the [`CentralPanel::show`] when using `DockArea`'s one.
    ///
    /// See also [`show_inside`](Self::show_inside).
    #[inline]
    pub fn show(self, ctx: &Context, tab_viewer: &mut impl TabViewer<Tab = Tab>) {
        CentralPanel::default()
            .frame(
                Frame::central_panel(&ctx.style())
                    .inner_margin(0.)
                    .fill(Color32::TRANSPARENT),
            )
            .show(ctx, |ui| {
                self.show_inside(ui, tab_viewer);
            });
    }

    /// Shows the docking hierarchy inside a [`Ui`].
    ///
    /// See also [`show`](Self::show).
    pub fn show_inside(self, ui: &mut Ui, tab_viewer: &mut impl TabViewer<Tab = Tab>) {
        let style = self
            .style
            .unwrap_or_else(|| Style::from_egui(ui.style().as_ref()));

        let mut state = State::load(ui.ctx(), self.id);
        let mut rect = ui.available_rect_before_wrap();

        if let Some(margin) = style.dock_area_padding {
            rect.min += margin.left_top();
            rect.max -= margin.right_bottom();
            ui.painter().rect(
                rect,
                margin.top,
                style.separator_color_idle,
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

                let (response, left, separator, right) = if is_horizontal {
                    style.hsplit(ui, fraction, rect)
                } else {
                    style.vsplit(ui, fraction, rect)
                };

                let color = if response.dragged() {
                    style.separator_color_dragged
                } else if response.hovered() {
                    style.separator_color_hovered
                } else {
                    style.separator_color_idle
                };

                ui.painter().rect_filled(separator, Rounding::none(), color);

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

                let height_topbar = style.tab_bar_height;

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

                    let mut available_width = tabbar.max.x - tabbar.min.x;
                    if style.show_add_buttons {
                        available_width -= Style::TAB_PLUS_SIZE;
                    }
                    let expanded_width = available_width / (tabs.len() as f32);

                    let mut ui = ui.child_ui(tabbar, Default::default());
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                    if !style.hline_below_active_tab_name {
                        ui.painter().hline(
                            tabbar.x_range(),
                            tabbar.max.y - px,
                            (px, style.hline_color),
                        );
                    }

                    ui.horizontal(|ui| {
                        for (tab_index, tab) in tabs.iter_mut().enumerate() {
                            let id = self.id.with((node_index, tab_index, "tab"));
                            let tab_index = TabIndex(tab_index);
                            let is_being_dragged = ui.memory(|mem| mem.is_being_dragged(id))
                                && style.tabs_are_draggable;

                            if is_being_dragged {
                                ui.output_mut(|o| o.cursor_icon = CursorIcon::Grabbing);
                            }

                            let is_active = *active == tab_index || is_being_dragged;
                            let label = tab_viewer.title(tab);

                            let response = if is_being_dragged {
                                let layer_id = LayerId::new(Order::Tooltip, id);
                                let mut response = ui
                                    .with_layer_id(layer_id, |ui| {
                                        style.tab_title(
                                            ui,
                                            label,
                                            is_active,
                                            is_active && Some(node_index) == focused,
                                            is_being_dragged,
                                            id,
                                            expanded_width,
                                        )
                                    })
                                    .response;

                                let sense = Sense::click_and_drag();
                                response = ui.interact(response.rect, id, sense);

                                if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                    let center = response.rect.center();
                                    let start = state.drag_start.unwrap_or(center);

                                    let delta = pointer_pos - start;
                                    if delta.x.abs() > 30.0 || delta.y.abs() > 6.0 {
                                        ui.ctx().translate_layer(layer_id, delta);

                                        drag_data = Some((node_index, tab_index));
                                    }
                                }

                                response
                            } else {
                                let (mut response, close_response) = style.tab_title(
                                    ui,
                                    label,
                                    is_active && Some(node_index) == focused,
                                    is_active,
                                    is_being_dragged,
                                    id,
                                    expanded_width,
                                );

                                let (close_hovered, close_clicked) = match close_response {
                                    Some(res) => (res.hovered(), res.clicked()),
                                    None => (false, false),
                                };

                                let sense = if close_hovered {
                                    Sense::click()
                                } else {
                                    Sense::click_and_drag()
                                };

                                if style.tab_hover_name {
                                    response = response.on_hover_ui(|ui| {
                                        ui.label(tab_viewer.title(tab));
                                    });
                                }

                                if style.show_context_menu {
                                    response = response.context_menu(|ui| {
                                        tab_viewer.context_menu(ui, tab);
                                        if style.show_close_buttons && ui.button("Close").clicked()
                                        {
                                            if tab_viewer.on_close(tab) {
                                                to_remove.push((node_index, tab_index));
                                            } else {
                                                *active = tab_index;
                                                new_focused = Some(node_index);
                                            }
                                        }
                                    });
                                }

                                if close_clicked {
                                    if tab_viewer.on_close(tab) {
                                        to_remove.push((node_index, tab_index));
                                    } else {
                                        *active = tab_index;
                                        new_focused = Some(node_index);
                                    }
                                }
                                let response = ui.interact(response.rect, id, sense);
                                if response.drag_started() {
                                    state.drag_start = response.hover_pos();
                                }

                                response
                            };

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

                            if state.drag_start.is_some() {
                                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                                    if response.rect.contains(pos) {
                                        tab_hover_rect = Some((response.rect, tab_index));
                                    }
                                }
                            }

                            tab_viewer.on_tab_button(tab, &response);
                        }

                        // Add button at the end of the tab bar
                        if style.show_add_buttons {
                            let id = self.id.with((node_index, "tab_add"));
                            let response = style.tab_plus(ui);

                            let response = ui.interact(response.rect, id, Sense::click());
                            let popup_id = id.with("tab_add_popup");

                            popup::popup_under_widget(ui, popup_id, &response, |ui| {
                                tab_viewer.add_popup(ui, node_index);
                            });

                            if response.clicked() {
                                if style.show_add_popup {
                                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                }
                                tab_viewer.on_add(node_index);
                            }
                        };
                    });
                });

                if style.hline_below_active_tab_name {
                    ui.painter().hline(
                        tabbar.x_range(),
                        tabbar.max.y - px,
                        (px, style.hline_color),
                    );
                }

                // tab body
                if let Some(tab) = tabs.get_mut(active.0) {
                    let top_y = rect.min.y + height_topbar;
                    let rect = rect.intersect(Rect::everything_below(top_y));
                    let rect = expand_to_pixel(rect, pixels_per_point);

                    *viewport = rect;

                    if ui.input(|i| i.pointer.any_click()) {
                        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
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
                        if style.tab_include_scrollarea {
                            ScrollArea::both()
                                .id_source(
                                    self.id
                                        .with((tab_viewer.title(tab).text(), "egui_dock::Tab")),
                                )
                                .show(ui, |ui| {
                                    Frame::none()
                                        .inner_margin(tab_viewer.inner_margin_override(&style))
                                        .show(ui, |ui| {
                                            let available_rect = ui.available_rect_before_wrap();
                                            ui.expand_to_include_rect(available_rect);
                                            tab_viewer.ui(ui, tab);
                                        });
                                });
                        } else {
                            Frame::none()
                                .inner_margin(tab_viewer.inner_margin_override(&style))
                                .show(ui, |ui| {
                                    tab_viewer.ui(ui, tab);
                                });
                        }
                    });
                }

                let is_being_dragged = ui.memory(|mem| mem.is_anything_being_dragged());
                if is_being_dragged && full_response.hovered() {
                    hover_data = ui
                        .input(|i| i.pointer.hover_pos())
                        .map(|pointer| HoverData {
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

                if ui.input(|i| i.pointer.any_released()) {
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

impl<'tree, Tab> fmt::Debug for DockArea<'tree, Tab> {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmtr.debug_struct("DockArea").finish_non_exhaustive()
    }
}
