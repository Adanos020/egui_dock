mod hover_data;
mod state;

use crate::{
    utils::{expand_to_pixel, map_to_pixel, rect_set_size_centered},
    widgets::popup::popup_under_widget,
    Node, NodeIndex, Style, TabAddAlign, TabIndex, TabViewer, Tree,
};

use duplicate::duplicate;
use egui::{
    containers::*, emath::*, epaint::*, layers::*, Context, CursorIcon, Id, Layout, Response,
    Sense, TextStyle, Ui, WidgetText,
};
use hover_data::HoverData;
use paste::paste;
use state::State;

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

    drag_data: Option<(NodeIndex, TabIndex)>,
    hover_data: Option<HoverData>,
    to_remove: Vec<(NodeIndex, TabIndex)>,
    new_focused: Option<NodeIndex>,
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
            drag_data: None,
            hover_data: None,
            to_remove: Vec::new(),
            new_focused: None,
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
        let style = self
            .style
            .take()
            .unwrap_or_else(|| Style::from_egui(ui.style().as_ref()));

        let mut state = State::load(ui.ctx(), self.id);

        self.allocate_area_for_root(ui, &style);

        for node_index in (0..self.tree.len()).map(NodeIndex) {
            if self.tree[node_index].is_parent() {
                self.split(ui, &style, node_index);
            }
        }

        for node_index in (0..self.tree.len()).map(NodeIndex) {
            if self.tree[node_index].is_leaf() {
                self.process_leaf(ui, &style, &mut state, node_index, tab_viewer);
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

            if self.tree[src].is_leaf()
                && self.tree[dst].is_leaf()
                && (src != dst || self.tree[dst].tabs_count() > 1)
            {
                let (overlay, tab_dst) = hover.resolve();
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

    fn allocate_area_for_root(&mut self, ui: &mut Ui, style: &Style) {
        let mut rect = ui.available_rect_before_wrap();

        if let Some(margin) = style.dock_area_padding {
            rect.min += margin.left_top();
            rect.max -= margin.right_bottom();
            ui.painter().rect(
                rect,
                margin.top,
                style.separator.color_idle,
                Stroke::new(margin.top, style.border.color),
            );
        }

        if self.tree.is_empty() {
            ui.allocate_rect(rect, Sense::hover());
            return;
        }

        self.tree[NodeIndex::root()].set_rect(rect);
    }

    #[allow(clippy::needless_return)]
    fn split(&mut self, ui: &mut Ui, style: &Style, node_index: NodeIndex) {
        assert!(self.tree[node_index].is_parent());

        let pixels_per_point = ui.ctx().pixels_per_point();

        duplicate! {
            [
                orientation     dimension_point dimension_size  left_rect              right_rect           left_point  right_point;
                [Horizontal]    [x]             [width]         [everything_right_of]  [everything_left_of] [max]       [min];
                [Vertical]      [y]             [height]        [everything_above]     [everything_below]   [min]       [max];
            ]
            if let Node::orientation { fraction, rect } = &mut self.tree[node_index] {
                let rect = expand_to_pixel(*rect, ui.ctx().pixels_per_point());

                let mut separator = rect;

                let midpoint = rect.min.dimension_point + rect.dimension_size() * *fraction;
                separator.min.dimension_point = midpoint - style.separator.width * 0.5;
                separator.max.dimension_point = midpoint + style.separator.width * 0.5;

                let response = ui
                    .allocate_rect(separator, Sense::click_and_drag())
                    .on_hover_and_drag_cursor(paste!{ CursorIcon::[<Resize orientation>]});

                if let Some(pos) = response.interact_pointer_pos() {
                    let dimension_point = pos.dimension_point;
                    let delta = response.drag_delta().dimension_point;

                    if (delta > 0. && dimension_point > midpoint && dimension_point < rect.max.dimension_point)
                        || (delta < 0. && dimension_point < midpoint && dimension_point > rect.min.dimension_point)
                    {
                        let range = rect.max.dimension_point - rect.min.dimension_point;
                        let min = (style.separator.extra / range).min(1.0);
                        let max = 1.0 - min;
                        let (min, max) = (min.min(max), max.max(min));
                        *fraction = (*fraction + delta / range).clamp(min, max);
                    }
                }

                let midpoint = rect.min.dimension_point + rect.dimension_size() * *fraction;
                separator.min.dimension_point = map_to_pixel(
                    midpoint - style.separator.width * 0.5,
                    pixels_per_point,
                    f32::round,
                );
                separator.max.dimension_point = map_to_pixel(
                    midpoint + style.separator.width * 0.5,
                    pixels_per_point,
                    f32::round,
                );

                let left = rect.intersect(Rect::left_rect(separator.left_point.dimension_point));
                let right = rect.intersect(Rect::right_rect(separator.right_point.dimension_point));

                let color = if response.dragged() {
                    style.separator.color_dragged
                } else if response.hovered() {
                    style.separator.color_hovered
                } else {
                    style.separator.color_idle
                };

                ui.painter().rect_filled(separator, Rounding::none(), color);

                self.tree[node_index.left()].set_rect(left);
                self.tree[node_index.right()].set_rect(right);
            }
        }
    }

    fn process_leaf(
        &mut self,
        ui: &mut Ui,
        style: &Style,
        state: &mut State,
        node_index: NodeIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
    ) {
        let focused = self.tree.focused_leaf();
        let px = ui.ctx().pixels_per_point().recip();

        let Node::Leaf {
            rect,
            tabs,
            active,
            viewport,
            scroll,
        } = &mut self.tree[node_index] else {
            unreachable!();
        };

        let ui = &mut ui.child_ui_with_id_source(
            *rect,
            Layout::top_down_justified(Align::Min),
            (node_index, "node"),
        );
        let id = self.id.with((node_index, "node"));
        let spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        ui.set_clip_rect(*rect);

        let (tabbar_outer_rect, tabbar_response) = ui.allocate_exact_size(
            vec2(ui.available_width(), style.tab_bar.height),
            Sense::hover(),
        );
        ui.painter().rect_filled(
            tabbar_outer_rect,
            style.tabs.rounding,
            style.tab_bar.bg_fill,
        );

        let mut tab_hover_rect = None;

        // tabs
        let actual_width = {
            let tabbar_inner_rect = Rect::from_min_size(
                (tabbar_outer_rect.min - pos2(-*scroll, 0.0)).to_pos2(),
                vec2(f32::INFINITY, tabbar_outer_rect.height()),
            );

            let tabs_ui = &mut ui.child_ui_with_id_source(
                tabbar_inner_rect,
                Layout::left_to_right(Align::Center),
                "tabs",
            );

            let mut available_width = tabbar_outer_rect.width();
            let mut clip_rect = tabbar_outer_rect;

            // Reserve space for the add button at the end of the tab bar
            if self.show_add_buttons {
                clip_rect.set_right(tabbar_outer_rect.right() - Style::TAB_ADD_BUTTON_SIZE);
                tabs_ui.set_clip_rect(clip_rect);
                available_width -= Style::TAB_ADD_BUTTON_SIZE;
            }

            tabs_ui.set_clip_rect(clip_rect);

            // Desired size for tabs in "expanded" mode
            let expanded_width = available_width / (tabs.len() as f32);

            for (tab_index, tab) in tabs.iter_mut().enumerate() {
                let id = id.with((tab_index, "tab"));
                let tab_index = TabIndex(tab_index);
                let is_being_dragged =
                    tabs_ui.memory(|mem| mem.is_being_dragged(id)) && self.draggable_tabs;

                if is_being_dragged {
                    tabs_ui.output_mut(|o| o.cursor_icon = CursorIcon::Grabbing);
                }

                let is_active = *active == tab_index || is_being_dragged;
                let label = tab_viewer.title(tab);

                let response = if is_being_dragged {
                    let layer_id = LayerId::new(Order::Tooltip, id);
                    let mut response = tabs_ui
                        .with_layer_id(layer_id, |ui| {
                            Self::tab_title(
                                ui,
                                style,
                                label,
                                is_active,
                                is_active && Some(node_index) == focused,
                                is_being_dragged,
                                id,
                                expanded_width,
                                self.show_close_buttons,
                            )
                        })
                        .response;

                    let sense = Sense::click_and_drag();
                    response = tabs_ui.interact(response.rect, id, sense);

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
                    let (mut response, close_response) = Self::tab_title(
                        tabs_ui,
                        style,
                        label,
                        is_active && Some(node_index) == focused,
                        is_active,
                        is_being_dragged,
                        id,
                        expanded_width,
                        self.show_close_buttons,
                    );

                    let (close_hovered, close_clicked) = close_response
                        .map(|res| (res.hovered(), res.clicked()))
                        .unwrap_or_default();

                    let sense = if close_hovered {
                        Sense::click()
                    } else {
                        Sense::click_and_drag()
                    };

                    if self.show_tab_name_on_hover {
                        response = response.on_hover_ui(|ui| {
                            ui.label(tab_viewer.title(tab));
                        });
                    }

                    if self.tab_context_menus {
                        response = response.context_menu(|ui| {
                            tab_viewer.context_menu(ui, tab);
                            if self.show_close_buttons && ui.button("Close").clicked() {
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
                            tab_hover_rect = Some((response.rect, tab_index));
                        }
                    }

                    response
                };

                // Paint hline below each tab unless its active (or option says overwise)
                if !is_active || style.tabs.hline_below_active_tab_name {
                    tabs_ui.painter().hline(
                        response.rect.x_range(),
                        tabbar_outer_rect.bottom() - px,
                        (px, style.tabs.hline_color),
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

            // Draw hline from tab end to edge of tabbar
            ui.painter().hline(
                tabs_ui.min_rect().right().min(clip_rect.right())..=tabbar_outer_rect.right(),
                tabbar_outer_rect.bottom() - px,
                (px, style.tabs.hline_color),
            );

            // Add button at the end of the tab bar
            if self.show_add_buttons {
                let offset = match style.buttons.add_tab_align {
                    TabAddAlign::Left => {
                        (clip_rect.width() - tabs_ui.min_rect().width()).at_least(0.0)
                    }
                    TabAddAlign::Right => 0.0,
                };

                let rect = Rect::from_min_max(
                    tabbar_outer_rect.right_top() - vec2(Style::TAB_ADD_BUTTON_SIZE + offset, 0.0),
                    tabbar_outer_rect.right_bottom() - vec2(offset, 0.0),
                );

                let ui = &mut ui.child_ui_with_id_source(
                    rect,
                    Layout::left_to_right(Align::Center),
                    (node_index, "tab_add"),
                );

                let response = Self::tab_plus(ui, style);

                // Draw button left border
                ui.painter().vline(
                    rect.left(),
                    rect.y_range(),
                    Stroke::new(px, style.tabs.hline_color),
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
            };

            tabs_ui.min_rect().width()
        };

        let overflow = (actual_width - tabbar_outer_rect.width()).at_least(0.0);
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
                let overflow_ratio = actual_width / scroll_bar_rect.width();
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

                let scroll_bar_handle_response =
                    ui.interact(scroll_bar_handle_rect, id, Sense::drag());

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

        // tab body
        let (body_rect, _body_response) =
            ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::click_and_drag());

        if let Some(tab) = tabs.get_mut(active.0) {
            *viewport = body_rect;

            if ui.input(|i| i.pointer.any_click()) {
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    if body_rect.contains(pos) {
                        self.new_focused = Some(node_index);
                    }
                }
            }

            if tab_viewer.clear_background(tab) {
                ui.painter().rect_filled(body_rect, 0.0, style.tabs.bg_fill);
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

            // Use initial spacing for ui
            ui.spacing_mut().item_spacing = spacing;

            if self.scroll_area_in_tabs {
                ScrollArea::both().show(ui, |ui| {
                    Frame::none()
                        .inner_margin(tab_viewer.inner_margin_override(style))
                        .show(ui, |ui| {
                            let available_rect = ui.available_rect_before_wrap();
                            ui.expand_to_include_rect(available_rect);
                            tab_viewer.ui(ui, tab);
                        });
                });
            } else {
                Frame::none()
                    .inner_margin(tab_viewer.inner_margin_override(style))
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
                    tab: tab_hover_rect,
                    pointer,
                });
            }
        }

        for (tab_index, tab) in tabs.iter_mut().enumerate() {
            if tab_viewer.force_close(tab) {
                self.to_remove.push((node_index, TabIndex(tab_index)));
            }
        }
    }

    fn tab_plus(ui: &mut Ui, style: &Style) -> Response {
        let (rect, mut response) = ui.allocate_exact_size(ui.available_size(), Sense::click());

        response = response.on_hover_cursor(CursorIcon::PointingHand);

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

        response
    }

    /// * `active` means "the tab that is opened in the parent panel".
    /// * `focused` means "the tab that was last interacted with".
    ///
    /// Returns the main button response plus the response of the close button, if any.
    #[allow(clippy::too_many_arguments)]
    fn tab_title(
        ui: &mut Ui,
        style: &Style,
        label: WidgetText,
        focused: bool,
        active: bool,
        is_being_dragged: bool,
        id: Id,
        expanded_width: f32,
        show_close: bool,
    ) -> (Response, Option<Response>) {
        let rounding = style.tabs.rounding;

        let galley = label.into_galley(ui, None, f32::INFINITY, TextStyle::Button);

        let x_spacing = 8.0;

        let text_width = galley.size().x + 2.0 * x_spacing;
        let close_button_size = if show_close {
            Style::TAB_CLOSE_BUTTON_SIZE.min(style.tab_bar.height)
        } else {
            0.0
        };
        let minimum_width = text_width + close_button_size;

        // Compute total width of the tab bar
        let tab_width = if style.tabs.fill_tab_bar {
            expanded_width
        } else {
            minimum_width
        }
        .at_least(minimum_width);

        let (rect, mut response) =
            ui.allocate_exact_size(vec2(tab_width, ui.available_height()), Sense::hover());
        if !ui.memory(|mem| mem.is_anything_being_dragged()) {
            response = response.on_hover_cursor(CursorIcon::Grab);
        }

        if active {
            if is_being_dragged {
                ui.painter().rect_stroke(
                    rect,
                    rounding,
                    Stroke::new(1.0, style.tabs.outline_color),
                );
            } else {
                let stroke = Stroke::new(1.0, style.tabs.outline_color);
                ui.painter()
                    .rect(rect, rounding, style.tabs.bg_fill, stroke);

                // Make the tab name area connect with the tab ui area:
                ui.painter().hline(
                    rect.x_range(),
                    rect.bottom(),
                    Stroke::new(2.0, style.tabs.bg_fill),
                );
            }
        }

        let mut text_rect = rect;
        text_rect.set_width(tab_width - close_button_size);

        let text_pos = if style.tabs.fill_tab_bar {
            let mut pos =
                Align2::CENTER_CENTER.pos_in_rect(&text_rect.shrink2(vec2(x_spacing, 0.0)));
            pos -= galley.size() / 2.0;
            pos
        } else {
            let mut pos = Align2::LEFT_CENTER.pos_in_rect(&text_rect.shrink2(vec2(x_spacing, 0.0)));
            pos.y -= galley.size().y / 2.0;
            pos
        };

        let override_text_color = if galley.galley_has_color {
            None // respect the color the user has chosen
        } else {
            Some(match (active, focused) {
                (false, false) => style.tabs.text_color_unfocused,
                (false, true) => style.tabs.text_color_focused,
                (true, false) => style.tabs.text_color_active_unfocused,
                (true, true) => style.tabs.text_color_active_focused,
            })
        };

        ui.painter().add(TextShape {
            pos: text_pos,
            galley: galley.galley,
            underline: Stroke::NONE,
            override_text_color,
            angle: 0.0,
        });

        let close_response = if show_close {
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
                let mut rounding = rounding;
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

            Some(response)
        } else {
            None
        };

        (response, close_response)
    }
}

impl<'tree, Tab> std::fmt::Debug for DockArea<'tree, Tab> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DockArea").finish_non_exhaustive()
    }
}
