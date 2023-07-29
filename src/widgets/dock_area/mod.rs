mod hover_data;
mod state;

use std::ops::RangeInclusive;

use crate::{
    utils::{expand_to_pixel, map_to_pixel, rect_set_size_centered, rect_stroke_box},
    widgets::popup::popup_under_widget,
    Node, NodeIndex, Style, TabAddAlign, TabIndex, TabStyle, TabViewer, Tree,
};

use duplicate::duplicate;
use egui::{
    containers::*, emath::*, epaint::*, layers::*, Context, CursorIcon, Id, Layout, Response,
    Sense, TextStyle, Ui, WidgetText,
};
use hover_data::HoverData;
use paste::paste;
use state::State;

/// What directions can this dock split in?
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum AllowedSplits {
    #[default]
    /// Allow splits in any direction (horizontal and vertical).
    All,
    /// Only allow split in a horizontal direction.
    LeftRightOnly,
    /// Only allow splits in a vertical direction.
    TopBottomOnly,
    /// Don't allow splits at all.
    None,
}

/// Displays a [`Tree`] in `egui`.
pub struct DockArea<'tree, Tab> {
    id: Id,
    tree: &'tree mut Tree<Tab>,
    style: Option<Style>,
    show_add_popup: bool,
    show_add_buttons: bool,
    show_close_buttons: bool,
    tab_context_menus: bool,
    draggable_tabs: bool,
    show_tab_name_on_hover: bool,
    scroll_area_in_tabs: bool,
    allowed_splits: AllowedSplits,

    drag_data: Option<(NodeIndex, TabIndex)>,
    hover_data: Option<HoverData>,
    to_remove: Vec<(NodeIndex, TabIndex)>,
    new_focused: Option<NodeIndex>,
    tab_hover_rect: Option<(Rect, TabIndex)>,
}

// Builder
impl<'tree, Tab> DockArea<'tree, Tab> {
    /// Creates a new [`DockArea`] from the provided [`Tree`].
    #[inline(always)]
    pub fn new(tree: &'tree mut Tree<Tab>) -> DockArea<'tree, Tab> {
        Self {
            id: Id::new("egui_dock::DockArea"),
            tree,
            style: None,
            show_add_popup: false,
            show_add_buttons: false,
            show_close_buttons: true,
            tab_context_menus: true,
            draggable_tabs: true,
            show_tab_name_on_hover: false,
            scroll_area_in_tabs: true,
            allowed_splits: AllowedSplits::default(),
            drag_data: None,
            hover_data: None,
            to_remove: Vec::new(),
            new_focused: None,
            tab_hover_rect: None,
        }
    }

    /// Sets the [`DockArea`] id. Useful if you have more than one [`DockArea`].
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

    /// Shows or hides the add button popup.
    /// By default it's false.
    pub fn show_add_popup(mut self, show_add_popup: bool) -> Self {
        self.show_add_popup = show_add_popup;
        self
    }

    /// Shows or hides the tab add buttons.
    /// By default it's false.
    pub fn show_add_buttons(mut self, show_add_buttons: bool) -> Self {
        self.show_add_buttons = show_add_buttons;
        self
    }

    /// Shows or hides the tab close buttons.
    /// By default it's true.
    pub fn show_close_buttons(mut self, show_close_buttons: bool) -> Self {
        self.show_close_buttons = show_close_buttons;
        self
    }

    /// Whether tabs show a context menu.
    /// By default it's true.
    pub fn tab_context_menus(mut self, tab_context_menus: bool) -> Self {
        self.tab_context_menus = tab_context_menus;
        self
    }

    /// Whether tabs can be dragged between nodes and reordered on the tab bar.
    /// By default it's true.
    pub fn draggable_tabs(mut self, draggable_tabs: bool) -> Self {
        self.draggable_tabs = draggable_tabs;
        self
    }

    /// Whether tabs show their name when hovered over them.
    /// By default it's false.
    pub fn show_tab_name_on_hover(mut self, show_tab_name_on_hover: bool) -> Self {
        self.show_tab_name_on_hover = show_tab_name_on_hover;
        self
    }

    /// Whether tabs have a [`ScrollArea`] out of the box.
    /// By default it's true.
    pub fn scroll_area_in_tabs(mut self, scroll_area_in_tabs: bool) -> Self {
        self.scroll_area_in_tabs = scroll_area_in_tabs;
        self
    }

    /// What directions can a node be split in: left-right, top-bottom, all, or none.
    /// By default it's all.
    pub fn allowed_splits(mut self, allowed_splits: AllowedSplits) -> Self {
        self.allowed_splits = allowed_splits;
        self
    }
}

// UI
impl<'tree, Tab> DockArea<'tree, Tab> {
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
    pub fn show_inside(mut self, ui: &mut Ui, tab_viewer: &mut impl TabViewer<Tab = Tab>) {
        self.style
            .get_or_insert_with(|| Style::from_egui(ui.style().as_ref()));

        let mut state = State::load(ui.ctx(), self.id);

        // First compute all rect sizes in the node graph.
        self.allocate_area_for_root(ui);
        for node_index in self.tree.breadth_first_index_iter() {
            if self.tree[node_index].is_parent() {
                self.compute_rect_sizes(ui, node_index);
            }
        }

        // Then draw the bodies of each leafs.
        for node_index in self.tree.breadth_first_index_iter() {
            if self.tree[node_index].is_leaf() {
                self.show_leaf(ui, &mut state, node_index, tab_viewer);
            }
        }

        // Finally draw separators so that their "interaction zone" is above
        // bodies (see `SeparatorStyle::extra_interact_width`).
        for node_index in self.tree.breadth_first_index_iter() {
            if self.tree[node_index].is_parent() {
                self.show_separator(ui, node_index);
            }
        }

        for index in self.to_remove.iter().copied().rev() {
            self.tree.remove_tab(index);
        }

        if let Some(focused) = self.new_focused {
            self.tree.set_focused_node(focused);
        }

        if let (Some((src, tab_index)), Some(hover)) = (self.drag_data, self.hover_data) {
            let dst = hover.dst;
            let style = self.style.as_ref().unwrap();

            if self.tree[src].is_leaf()
                && self.tree[dst].is_leaf()
                && (src != dst || self.tree[dst].tabs_count() > 1)
            {
                let (overlay, tab_dst) = hover.resolve(&self.allowed_splits);
                let id = Id::new("overlay");
                let layer_id = LayerId::new(Order::Foreground, id);
                let painter = ui.ctx().layer_painter(layer_id);
                painter.rect_filled(overlay, 0.0, style.selection_color);
                if ui.input(|i| i.pointer.any_released()) {
                    self.tree.move_tab((src, tab_index), (dst, tab_dst));
                }
            }
        }

        state.store(ui.ctx(), self.id);
    }

    fn allocate_area_for_root(&mut self, ui: &mut Ui) {
        let style = self.style.as_ref().unwrap();
        let mut rect = ui.available_rect_before_wrap();

        if let Some(margin) = style.dock_area_padding {
            rect.min += margin.left_top();
            rect.max -= margin.right_bottom();
        }

        ui.painter().rect_stroke(rect, style.rounding, style.border);
        rect = rect.expand(-style.border.width / 2.0);
        ui.allocate_rect(rect, Sense::hover());

        if !self.tree.is_empty() {
            self.tree[NodeIndex::root()].set_rect(rect);
        }
    }

    fn compute_rect_sizes(&mut self, ui: &mut Ui, node_index: NodeIndex) {
        assert!(self.tree[node_index].is_parent());

        let style = self.style.as_ref().unwrap();
        let pixels_per_point = ui.ctx().pixels_per_point();

        duplicate! {
            [
                orientation   dim_point  dim_size  left_of    right_of  ;
                [Horizontal]  [x]        [width]   [left_of]  [right_of];
                [Vertical]    [y]        [height]  [above]    [below]   ;
            ]
            if let Node::orientation { fraction, rect } = &mut self.tree[node_index] {
                debug_assert!(!rect.any_nan() && rect.is_finite());
                let rect = expand_to_pixel(*rect, pixels_per_point);

                let midpoint = rect.min.dim_point + rect.dim_size() * *fraction;
                let left_separator_border = map_to_pixel(
                    midpoint - style.separator.width * 0.5,
                    pixels_per_point,
                    f32::round
                );
                let right_separator_border = map_to_pixel(
                    midpoint + style.separator.width * 0.5,
                    pixels_per_point,
                    f32::round
                );

                paste! {
                    let left = rect.intersect(Rect::[<everything_ left_of>](left_separator_border));
                    let right = rect.intersect(Rect::[<everything_ right_of>](right_separator_border));
                }

                self.tree[node_index.left()].set_rect(left);
                self.tree[node_index.right()].set_rect(right);
            }
        }
    }

    fn show_separator(&mut self, ui: &mut Ui, node_index: NodeIndex) {
        assert!(self.tree[node_index].is_parent());

        let style = self.style.as_ref().unwrap();
        let pixels_per_point = ui.ctx().pixels_per_point();

        duplicate! {
            [
                orientation   dim_point  dim_size;
                [Horizontal]  [x]        [width] ;
                [Vertical]    [y]        [height];
            ]
            if let Node::orientation { fraction, ref rect } = &mut self.tree[node_index] {
                let mut separator = *rect;

                let midpoint = rect.min.dim_point + rect.dim_size() * *fraction;
                separator.min.dim_point = midpoint - style.separator.width * 0.5;
                separator.max.dim_point = midpoint + style.separator.width * 0.5;

                let mut expand = Vec2::ZERO;
                expand.dim_point += style.separator.extra_interact_width / 2.0;
                let interact_rect = separator.expand2(expand);

                let response = ui.allocate_rect(interact_rect, Sense::click_and_drag())
                    .on_hover_and_drag_cursor(paste!{ CursorIcon::[<Resize orientation>]});

                let midpoint = rect.min.dim_point + rect.dim_size() * *fraction;
                separator.min.dim_point = map_to_pixel(
                    midpoint - style.separator.width * 0.5,
                    pixels_per_point,
                    f32::round,
                );
                separator.max.dim_point = map_to_pixel(
                    midpoint + style.separator.width * 0.5,
                    pixels_per_point,
                    f32::round,
                );

                let color = if response.dragged() {
                    style.separator.color_dragged
                } else if response.hovered() {
                    style.separator.color_hovered
                } else {
                    style.separator.color_idle
                };

                ui.painter().rect_filled(separator, Rounding::none(), color);

                // Update 'fraction' interaction after drawing seperator,
                // overwise it may overlap on other separator / bodies when
                // shrunk fast.
                if let Some(pos) = response.interact_pointer_pos() {
                    let dim_point = pos.dim_point;
                    let delta = response.drag_delta().dim_point;

                    if (delta > 0. && dim_point > midpoint && dim_point < rect.max.dim_point)
                        || (delta < 0. && dim_point < midpoint && dim_point > rect.min.dim_point)
                    {
                        let range = rect.max.dim_point - rect.min.dim_point;
                        let min = (style.separator.extra / range).min(1.0);
                        let max = 1.0 - min;
                        let (min, max) = (min.min(max), max.max(min));
                        *fraction = (*fraction + delta / range).clamp(min, max);
                    }
                }

                if response.double_clicked() {
                    *fraction = 0.5;
                }
            }
        }
    }

    fn show_leaf(
        &mut self,
        ui: &mut Ui,
        state: &mut State,
        node_index: NodeIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
    ) {
        assert!(self.tree[node_index].is_leaf());

        let rect = {
            let Node::Leaf { rect, .. } = &mut self.tree[node_index] else {
                unreachable!()
            };
            *rect
        };
        let ui = &mut ui.child_ui_with_id_source(
            rect,
            Layout::top_down_justified(Align::Min),
            (node_index, "node"),
        );
        let spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        ui.set_clip_rect(rect);

        let tabbar_response = self.tab_bar(ui, state, node_index, tab_viewer);
        self.tab_body(ui, state, node_index, tab_viewer, spacing, tabbar_response);

        let Node::Leaf { tabs, .. } = &mut self.tree[node_index] else {
            unreachable!()
        };
        for (tab_index, tab) in tabs.iter_mut().enumerate() {
            if tab_viewer.force_close(tab) {
                self.to_remove.push((node_index, TabIndex(tab_index)));
            }
        }
    }

    fn tab_bar(
        &mut self,
        ui: &mut Ui,
        state: &mut State,
        node_index: NodeIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
    ) -> Response {
        assert!(self.tree[node_index].is_leaf());

        let style = self.style.as_ref().unwrap();
        let (tabbar_outer_rect, tabbar_response) = ui.allocate_exact_size(
            vec2(ui.available_width(), style.tab_bar.height),
            Sense::hover(),
        );
        ui.painter().rect_filled(
            tabbar_outer_rect,
            style.tab_bar.rounding,
            style.tab_bar.bg_fill,
        );

        let mut available_width = tabbar_outer_rect.width();

        // Reserve space for the add button at the end of the tab bar
        if self.show_add_buttons {
            available_width -= Style::TAB_ADD_BUTTON_SIZE;
        }

        let actual_width = {
            let Node::Leaf { tabs, scroll, .. } = &mut self.tree[node_index] else {
                unreachable!()
            };

            let tabbar_inner_rect = Rect::from_min_size(
                (tabbar_outer_rect.min - pos2(-*scroll, 0.0)).to_pos2(),
                vec2(tabbar_outer_rect.width(), tabbar_outer_rect.height()),
            );

            let tabs_ui = &mut ui.child_ui_with_id_source(
                tabbar_inner_rect,
                Layout::left_to_right(Align::Center),
                "tabs",
            );

            let mut clip_rect = tabbar_outer_rect;
            clip_rect.set_width(available_width);
            tabs_ui.set_clip_rect(clip_rect);

            // Desired size for tabs in "expanded" mode
            let prefered_width = style
                .tab_bar
                .fill_tab_bar
                .then_some(available_width / (tabs.len() as f32));

            self.tabs(
                tabs_ui,
                state,
                node_index,
                tab_viewer,
                tabbar_outer_rect,
                prefered_width,
            );

            // Draw hline from tab end to edge of tabbar
            let px = ui.ctx().pixels_per_point().recip();
            let style = self.style.as_ref().unwrap();

            ui.painter().hline(
                tabs_ui.min_rect().right().min(clip_rect.right())..=tabbar_outer_rect.right(),
                tabbar_outer_rect.bottom() - px,
                (px, style.tab_bar.hline_color),
            );

            // Add button at the end of the tab bar
            if self.show_add_buttons {
                let offset = match style.buttons.add_tab_align {
                    TabAddAlign::Left => {
                        (clip_rect.width() - tabs_ui.min_rect().width()).at_least(0.0)
                    }
                    TabAddAlign::Right => 0.0,
                };
                self.tab_plus(ui, node_index, tab_viewer, tabbar_outer_rect, offset);
            }

            tabs_ui.min_rect().width()
        };

        self.tab_bar_scroll(
            ui,
            node_index,
            actual_width,
            available_width,
            &tabbar_response,
        );

        tabbar_response
    }

    fn tabs(
        &mut self,
        tabs_ui: &mut Ui,
        state: &mut State,
        node_index: NodeIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        tabbar_outer_rect: Rect,
        prefered_width: Option<f32>,
    ) {
        assert!(self.tree[node_index].is_leaf());

        let focused = self.tree.focused_leaf();
        let tabs_len = {
            let Node::Leaf { tabs, .. } = &mut self.tree[node_index] else {
                unreachable!()
            };
            tabs.len()
        };

        for tab_index in 0..tabs_len {
            let id = self.id.with((node_index, "node")).with((tab_index, "tab"));
            let tab_index = TabIndex(tab_index);
            let is_being_dragged =
                tabs_ui.memory(|mem| mem.is_being_dragged(id)) && self.draggable_tabs;

            if is_being_dragged {
                tabs_ui.output_mut(|o| o.cursor_icon = CursorIcon::Grabbing);
            }

            let (is_active, label, tab_style, closeable) = {
                let Node::Leaf { tabs, active, .. } = &mut self.tree[node_index] else { unreachable!() };
                let style = self.style.as_ref().unwrap();
                let tab_style = tab_viewer.tab_style_override(&tabs[tab_index.0], &style.tab);
                (
                    *active == tab_index || is_being_dragged,
                    tab_viewer.title(&mut tabs[tab_index.0]),
                    tab_style.unwrap_or(style.tab.clone()),
                    tab_viewer.closeable(&mut tabs[tab_index.0]),
                )
            };

            let show_close_button = self.show_close_buttons && closeable;

            let response = if is_being_dragged {
                let layer_id = LayerId::new(Order::Tooltip, id);
                let response = tabs_ui
                    .with_layer_id(layer_id, |ui| {
                        self.tab_title(
                            ui,
                            &tab_style,
                            id,
                            label,
                            is_active && Some(node_index) == focused,
                            is_active,
                            is_being_dragged,
                            prefered_width,
                            show_close_button,
                        )
                    })
                    .response;

                let sense = Sense::click_and_drag();
                let response = tabs_ui.interact(response.rect, id, sense);

                if let Some(pointer_pos) = tabs_ui.ctx().pointer_interact_pos() {
                    let center = response.rect.center();
                    let start = state.drag_start.unwrap_or(center);

                    let delta = pointer_pos - start;
                    if delta.x.abs() > 30.0 || delta.y.abs() > 6.0 {
                        tabs_ui.ctx().translate_layer(layer_id, delta);

                        self.drag_data = Some((node_index, tab_index));
                    }
                }

                response
            } else {
                let (mut response, close_response) = self.tab_title(
                    tabs_ui,
                    &tab_style,
                    id,
                    label,
                    is_active && Some(node_index) == focused,
                    is_active,
                    is_being_dragged,
                    prefered_width,
                    show_close_button,
                );

                let (close_hovered, close_clicked) = close_response
                    .map(|res| (res.hovered(), res.clicked()))
                    .unwrap_or_default();

                let sense = if close_hovered {
                    Sense::click()
                } else {
                    Sense::click_and_drag()
                };

                let Node::Leaf { tabs, active, .. } = &mut self.tree[node_index] else {
                    unreachable!()
                };
                let tab = &mut tabs[tab_index.0];
                if self.show_tab_name_on_hover {
                    response = response.on_hover_ui(|ui| {
                        ui.label(tab_viewer.title(tab));
                    });
                }

                if self.tab_context_menus {
                    response = response.context_menu(|ui| {
                        tab_viewer.context_menu(ui, tab);
                        if show_close_button && ui.button("Close").clicked() {
                            if tab_viewer.on_close(tab) {
                                self.to_remove.push((node_index, tab_index));
                            } else {
                                *active = tab_index;
                                self.new_focused = Some(node_index);
                            }
                        }
                    });
                }

                if close_clicked {
                    if tab_viewer.on_close(tab) {
                        self.to_remove.push((node_index, tab_index));
                    } else {
                        *active = tab_index;
                        self.new_focused = Some(node_index);
                    }
                }
                let response = tabs_ui.interact(response.rect, id, sense);
                if response.drag_started() {
                    state.drag_start = response.hover_pos();
                }

                if let Some(pos) = tabs_ui.input(|i| i.pointer.hover_pos()) {
                    // Use response.rect.contains instead of
                    // response.hovered as the dragged tab covers
                    // the underlying tab
                    if state.drag_start.is_some() && response.rect.contains(pos) {
                        self.tab_hover_rect = Some((response.rect, tab_index));
                    }
                }

                response
            };

            // Paint hline below each tab unless its active (or option says otherwise)
            let Node::Leaf { tabs, active, .. } = &mut self.tree[node_index] else {
                unreachable!()
            };
            let tab = &mut tabs[tab_index.0];
            let style = self.style.as_ref().unwrap();
            let tab_style = tab_viewer.tab_style_override(tab, &style.tab);
            let tab_style = tab_style.as_ref().unwrap_or(&style.tab);

            if !is_active || tab_style.hline_below_active_tab_name {
                let px = tabs_ui.ctx().pixels_per_point().recip();
                tabs_ui.painter().hline(
                    response.rect.x_range(),
                    tabbar_outer_rect.bottom() - px,
                    (px, style.tab_bar.hline_color),
                );
            }

            if response.clicked() {
                *active = tab_index;
                self.new_focused = Some(node_index);
            }

            if self.show_close_buttons && response.middle_clicked() {
                if tab_viewer.on_close(tab) {
                    self.to_remove.push((node_index, tab_index));
                } else {
                    *active = tab_index;
                    self.new_focused = Some(node_index);
                }
            }

            tab_viewer.on_tab_button(tab, &response);
        }
    }

    fn tab_plus(
        &mut self,
        ui: &mut Ui,
        node_index: NodeIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        tabbar_outer_rect: Rect,
        offset: f32,
    ) {
        let rect = Rect::from_min_max(
            tabbar_outer_rect.right_top() - vec2(Style::TAB_ADD_BUTTON_SIZE + offset, 0.0),
            tabbar_outer_rect.right_bottom() - vec2(offset, 2.0),
        );

        let ui = &mut ui.child_ui_with_id_source(
            rect,
            Layout::left_to_right(Align::Center),
            (node_index, "tab_add"),
        );

        let (rect, mut response) = ui.allocate_exact_size(ui.available_size(), Sense::click());

        response = response.on_hover_cursor(CursorIcon::PointingHand);

        let style = self.style.as_ref().unwrap();
        let color = if response.hovered() {
            ui.painter()
                .rect_filled(rect, Rounding::none(), style.buttons.add_tab_bg_fill);
            style.buttons.add_tab_active_color
        } else {
            style.buttons.add_tab_color
        };

        let mut plus_rect = rect;

        rect_set_size_centered(&mut plus_rect, Vec2::splat(Style::TAB_ADD_PLUS_SIZE));

        ui.painter().line_segment(
            [plus_rect.center_top(), plus_rect.center_bottom()],
            Stroke::new(1.0, color),
        );
        ui.painter().line_segment(
            [plus_rect.right_center(), plus_rect.left_center()],
            Stroke::new(1.0, color),
        );

        // Draw button left border
        ui.painter().vline(
            rect.left(),
            rect.y_range(),
            Stroke::new(
                ui.ctx().pixels_per_point().recip(),
                style.buttons.add_tab_border_color,
            ),
        );

        let popup_id = ui.id().with("tab_add_popup");
        popup_under_widget(ui, popup_id, &response, |ui| {
            tab_viewer.add_popup(ui, node_index);
        });

        if response.clicked() {
            if self.show_add_popup {
                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
            }
            tab_viewer.on_add(node_index);
        }
    }

    /// * `active` means "the tab that is opened in the parent panel".
    /// * `focused` means "the tab that was last interacted with".
    ///
    /// Returns the main button response plus the response of the close button, if any.
    #[allow(clippy::too_many_arguments)]
    fn tab_title(
        &mut self,
        ui: &mut Ui,
        tab_style: &TabStyle,
        id: Id,
        label: WidgetText,
        focused: bool,
        active: bool,
        is_being_dragged: bool,
        prefered_width: Option<f32>,
        show_close_button: bool,
    ) -> (Response, Option<Response>) {
        let style = self.style.as_ref().unwrap();
        let galley = label.into_galley(ui, None, f32::INFINITY, TextStyle::Button);
        let x_spacing = 8.0;
        let text_width = galley.size().x + 2.0 * x_spacing;
        let close_button_size = if self.show_close_buttons {
            Style::TAB_CLOSE_BUTTON_SIZE.min(style.tab_bar.height)
        } else {
            0.0
        };

        // Compute total width of the tab bar
        let minimum_width = tab_style
            .minimum_width
            .unwrap_or(0.0)
            .at_least(text_width + close_button_size);
        let tab_width = prefered_width.unwrap_or(0.0).at_least(minimum_width);

        let (rect, mut response) =
            ui.allocate_exact_size(vec2(tab_width, ui.available_height()), Sense::hover());
        if !ui.memory(|mem| mem.is_anything_being_dragged()) && self.draggable_tabs {
            response = response.on_hover_cursor(CursorIcon::PointingHand);
        }

        let tab_style = if focused || is_being_dragged {
            &tab_style.focused
        } else if active {
            &tab_style.active
        } else if response.hovered() {
            &tab_style.hovered
        } else {
            &tab_style.inactive
        };

        // Draw the full tab first and then the stroke ontop to avoid the stroke
        // mixing with the background color.
        ui.painter()
            .rect_filled(rect, tab_style.rounding, tab_style.bg_fill);
        let stroke_rect = rect_stroke_box(rect, 1.0);
        ui.painter().rect_stroke(
            stroke_rect,
            tab_style.rounding,
            Stroke::new(1.0, tab_style.outline_color),
        );
        if !is_being_dragged {
            // Make the tab name area connect with the tab ui area:
            ui.painter().hline(
                RangeInclusive::new(
                    stroke_rect.min.x + f32::max(tab_style.rounding.sw, 1.5),
                    stroke_rect.max.x - f32::max(tab_style.rounding.se, 1.5),
                ),
                stroke_rect.bottom(),
                Stroke::new(2.0, tab_style.bg_fill),
            );
        }

        let mut text_rect = rect;
        text_rect.set_width(tab_width - close_button_size);

        let text_pos = {
            let mut pos =
                Align2::CENTER_CENTER.pos_in_rect(&text_rect.shrink2(vec2(x_spacing, 0.0)));
            pos -= galley.size() / 2.0;
            pos
        };

        let override_text_color = (!galley.galley_has_color).then_some(tab_style.text_color);

        ui.painter().add(TextShape {
            pos: text_pos,
            galley: galley.galley,
            underline: Stroke::NONE,
            override_text_color,
            angle: 0.0,
        });

        let close_response = show_close_button.then(|| {
            let mut close_button_rect = rect;
            close_button_rect.set_left(text_rect.right());
            close_button_rect =
                Rect::from_center_size(close_button_rect.center(), Vec2::splat(close_button_size));

            let response = ui
                .interact(close_button_rect, id, Sense::click())
                .on_hover_cursor(CursorIcon::PointingHand);

            let color = if response.hovered() {
                style.buttons.close_tab_active_color
            } else {
                style.buttons.close_tab_color
            };

            if response.hovered() {
                let mut rounding = tab_style.rounding;
                rounding.nw = 0.0;
                rounding.sw = 0.0;

                ui.painter().rect_filled(
                    close_button_rect,
                    rounding,
                    style.buttons.add_tab_bg_fill,
                );
            }

            let mut x_rect = close_button_rect;
            rect_set_size_centered(&mut x_rect, Vec2::splat(Style::TAB_CLOSE_X_SIZE));
            ui.painter().line_segment(
                [x_rect.left_top(), x_rect.right_bottom()],
                Stroke::new(1.0, color),
            );
            ui.painter().line_segment(
                [x_rect.right_top(), x_rect.left_bottom()],
                Stroke::new(1.0, color),
            );

            response
        });

        (response, close_response)
    }

    fn tab_bar_scroll(
        &mut self,
        ui: &mut Ui,
        node_index: NodeIndex,
        actual_width: f32,
        available_width: f32,
        tabbar_response: &Response,
    ) {
        let Node::Leaf { scroll, .. } = &mut self.tree[node_index] else {
            unreachable!()
        };
        let overflow = (actual_width - available_width).at_least(0.0);
        let style = self.style.as_ref().unwrap();

        // Compare to 1.0 and not 0.0 to avoid drawing a scroll bar due
        // to floating point precision issue during tab drawing
        if overflow > 1.0 {
            if style.tab_bar.show_scroll_bar_on_overflow {
                // Draw scroll bar
                let bar_height = 7.5;
                let (scroll_bar_rect, _scroll_bar_response) = ui.allocate_exact_size(
                    vec2(ui.available_width(), bar_height),
                    Sense::click_and_drag(),
                );

                // Compute scroll bar handle position and size
                let overflow_ratio = actual_width / available_width;
                let scroll_ratio = -*scroll / overflow;

                let scroll_bar_handle_size = overflow_ratio.recip() * scroll_bar_rect.width();
                let scroll_bar_handle_start = lerp(
                    scroll_bar_rect.left()..=scroll_bar_rect.right() - scroll_bar_handle_size,
                    scroll_ratio,
                );
                let scroll_bar_handle_rect = Rect::from_min_size(
                    pos2(scroll_bar_handle_start, scroll_bar_rect.min.y),
                    vec2(scroll_bar_handle_size, bar_height),
                );

                let scroll_bar_handle_response = ui.interact(
                    scroll_bar_handle_rect,
                    self.id.with((node_index, "node")),
                    Sense::drag(),
                );

                // Coefficient to apply to input displacements so that we
                // move the scroll by the correct amount.
                let points_to_scroll_coefficient =
                    overflow / (scroll_bar_rect.width() - scroll_bar_handle_size);

                *scroll -= scroll_bar_handle_response.drag_delta().x * points_to_scroll_coefficient;

                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    if scroll_bar_rect.contains(pos) {
                        *scroll += ui.input(|i| i.scroll_delta.y + i.scroll_delta.x)
                            * points_to_scroll_coefficient;
                    }
                }

                // Draw the bar
                ui.painter()
                    .rect_filled(scroll_bar_rect, 0.0, ui.visuals().extreme_bg_color);

                ui.painter().rect_filled(
                    scroll_bar_handle_rect,
                    bar_height / 2.0,
                    ui.visuals()
                        .widgets
                        .style(&scroll_bar_handle_response)
                        .bg_fill,
                );
            }

            // Handle user input
            if tabbar_response.hovered() {
                *scroll += ui.input(|i| i.scroll_delta.y + i.scroll_delta.x);
            }
        }

        *scroll = scroll.clamp(-overflow, 0.0);
    }

    fn tab_body(
        &mut self,
        ui: &mut Ui,
        state: &mut State,
        node_index: NodeIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        spacing: Vec2,
        tabbar_response: Response,
    ) {
        let (body_rect, _body_response) =
            ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::click_and_drag());

        let Node::Leaf {
            rect,
            tabs,
            active,
            viewport,
            ..
        } = &mut self.tree[node_index]
        else {
            unreachable!();
        };

        if let Some(tab) = tabs.get_mut(active.0) {
            *viewport = body_rect;

            if ui.input(|i| i.pointer.any_click()) {
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    if body_rect.contains(pos) {
                        self.new_focused = Some(node_index);
                    }
                }
            }

            let style = self.style.as_ref().unwrap();
            let tabs_styles = tab_viewer.tab_style_override(tab, &style.tab);
            let tabs_style = tabs_styles.as_ref().unwrap_or(&style.tab);
            if tab_viewer.clear_background(tab) {
                ui.painter()
                    .rect_filled(body_rect, 0.0, tabs_style.tab_body.bg_fill);
            }

            // Construct a new ui with the correct tab id
            //
            // We are forced to use `Ui::new` because other methods (eg: push_id) always mix
            // the provided id with their own which would cause tabs to change id when moved
            // from node to node.
            let id = self.id.with(tab_viewer.id(tab));
            ui.ctx().check_for_id_clash(id, body_rect, "a tab with id");
            let ui = &mut Ui::new(
                ui.ctx().clone(),
                ui.layer_id(),
                id,
                body_rect,
                ui.clip_rect(),
            );
            ui.set_clip_rect(Rect::from_min_max(ui.cursor().min, ui.clip_rect().max));

            // Use initial spacing for ui
            ui.spacing_mut().item_spacing = spacing;

            // Offset the background rectangle up to hide the top border behind the clip rect.
            // To avoid anti-aliasing lines when the stroke width is not divisible by two, we
            // need to calulate the effective anti aliased stroke width.
            let effective_stroke_width = (tabs_style.tab_body.stroke.width / 2.0).ceil() * 2.0;
            let tab_body_rect = Rect::from_min_max(
                ui.clip_rect().min - vec2(0.0, effective_stroke_width),
                ui.clip_rect().max,
            );
            ui.painter().rect(
                rect_stroke_box(tab_body_rect, tabs_style.tab_body.stroke.width),
                tabs_style.tab_body.rounding,
                tabs_style.tab_body.bg_fill,
                tabs_style.tab_body.stroke,
            );

            if self.scroll_area_in_tabs {
                ScrollArea::both().show(ui, |ui| {
                    Frame::none()
                        .inner_margin(tabs_style.tab_body.inner_margin)
                        .show(ui, |ui| {
                            let available_rect = ui.available_rect_before_wrap();
                            ui.expand_to_include_rect(available_rect);
                            tab_viewer.ui(ui, tab);
                        });
                });
            } else {
                Frame::none()
                    .inner_margin(tabs_style.tab_body.inner_margin)
                    .show(ui, |ui| {
                        tab_viewer.ui(ui, tab);
                    });
            }
        }

        if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
            // Use rect.contains instead of
            // response.hovered as the dragged tab covers
            // the underlying responses
            if state.drag_start.is_some() && rect.contains(pointer) {
                self.hover_data = Some(HoverData {
                    rect: *rect,
                    dst: node_index,
                    tabs: tabbar_response.hovered().then_some(tabbar_response.rect),
                    tab: self.tab_hover_rect,
                    pointer,
                });
            }
        }
    }
}

impl<'tree, Tab> std::fmt::Debug for DockArea<'tree, Tab> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DockArea").finish_non_exhaustive()
    }
}
