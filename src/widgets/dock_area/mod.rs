mod hover_data;
mod state;
mod style_fade;

use std::{ops::RangeInclusive, time::Instant};

use crate::{
    dock_state::DockState,
    utils::{expand_to_pixel, map_to_pixel, rect_set_size_centered, rect_stroke_box},
    widgets::popup::popup_under_widget,
    Node, NodeIndex, Style, SurfaceIndex, TabAddAlign, TabIndex, TabStyle, TabViewer, Tree,
};

use duplicate::duplicate;
use egui::{
    containers::*, emath::*, epaint::*, layers::*, Context, CursorIcon, Id, Layout, Response,
    Sense, TextStyle, Ui, WidgetText,
};
use hover_data::DragDropState;
use paste::paste;
use state::State;

use self::{
    hover_data::{DragData, DropPosition, HoverData},
    style_fade::{fade_dock_style, fade_visuals},
};

/// What directions can this dock split in?
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum AllowedSplits {
    #[default]
    /// Allow splits in any direction (horizontal and vertical).
    All = 0b11,
    /// Only allow split in a horizontal direction.
    LeftRightOnly = 0b10,
    /// Only allow splits in a vertical direction.
    TopBottomOnly = 0b01,
    /// Don't allow splits at all.
    None = 0b00,
}
impl std::ops::BitAnd for AllowedSplits {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_u8(self as u8 & rhs as u8)
    }
}
impl AllowedSplits {
    ///Create an allowedsplits from a u8,
    ///panics if an invalid value is given
    #[inline(always)]
    fn from_u8(u8: u8) -> Self {
        match u8 {
            0b11 => AllowedSplits::All,
            0b10 => AllowedSplits::LeftRightOnly,
            0b01 => AllowedSplits::TopBottomOnly,
            0b00 => AllowedSplits::None,
            _ => panic!(),
        }
    }
}
/// Displays a [`Tree`] in `egui`.
pub struct DockArea<'tree, Tab> {
    id: Id,
    dock_state: &'tree mut DockState<Tab>,
    style: Option<Style>,
    show_add_popup: bool,
    show_add_buttons: bool,
    show_close_buttons: bool,
    tab_context_menus: bool,
    draggable_tabs: bool,
    show_tab_name_on_hover: bool,
    scroll_area_in_tabs: bool,
    allowed_splits: AllowedSplits,

    drag_data: Option<DragData>,
    hover_data: Option<HoverData>,
    to_remove: Vec<TabRemoval>,
    to_detach: Vec<(SurfaceIndex, NodeIndex, TabIndex)>,
    new_focused: Option<(SurfaceIndex, NodeIndex)>,
    tab_hover_rect: Option<(Rect, TabIndex)>,
}

/// An enum expressing an entry in the `to_remove` field in [`DockArea`]
#[derive(Debug, Clone, Copy)]
enum TabRemoval {
    Node(SurfaceIndex, NodeIndex, TabIndex),
    Window(SurfaceIndex),
}

impl From<SurfaceIndex> for TabRemoval {
    fn from(index: SurfaceIndex) -> Self {
        TabRemoval::Window(index)
    }
}

impl From<(SurfaceIndex, NodeIndex, TabIndex)> for TabRemoval {
    fn from((si, ni, ti): (SurfaceIndex, NodeIndex, TabIndex)) -> TabRemoval {
        TabRemoval::Node(si, ni, ti)
    }
}

// Builder
impl<'tree, Tab> DockArea<'tree, Tab> {
    /// Creates a new [`DockArea`] from the provided [`Tree`].
    #[inline(always)]
    pub fn new(tree: &'tree mut DockState<Tab>) -> DockArea<'tree, Tab> {
        Self {
            id: Id::new("egui_dock::DockArea"),
            dock_state: tree,
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
            to_detach: Vec::new(),
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
    /// # use egui_dock::{DockArea, DockState};
    /// # use egui::{CentralPanel, Frame};
    /// # struct TabViewer {}
    /// # impl egui_dock::TabViewer for TabViewer {
    /// #     type Tab = String;
    /// #     fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {}
    /// #     fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText { (&*tab).into() }
    /// # }
    /// # let mut tree: DockState<String> = DockState::new(vec![]);
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
        //if let Some(hover_data) = state.drag.take() {
        //    if hover_data.locked.is_some() {
        //        self.hover_data = Some(hover_data);
        //    }
        //}
        let style = self.style.as_ref().unwrap();
        let fade_surface =
            self.hovered_window_surface(&mut state, style.overlay.feel.fade_hold_time, ui.ctx());
        let fade_style = {
            fade_surface.is_some().then(|| {
                let mut fade_style = style.clone();
                fade_dock_style(&mut fade_style, style.overlay.surface_fade_opacity);
                (fade_style, style.overlay.surface_fade_opacity)
            })
        };

        for surface_index in self.dock_state.surface_index_iter() {
            self.show_surface_inside(
                surface_index,
                ui,
                tab_viewer,
                &mut state,
                fade_style.as_ref().map(|(style, factor)| {
                    (style, *factor, fade_surface.unwrap_or(SurfaceIndex::root()))
                }),
            );
        }

        for index in self.to_remove.drain(..).rev() {
            match index {
                TabRemoval::Node(surface, node, tab) => {
                    self.dock_state[surface].remove_tab((node, tab));
                    if self.dock_state[surface].is_empty() && !surface.is_root() {
                        self.dock_state.remove_surface(surface);
                    }
                }
                TabRemoval::Window(index) => {
                    self.dock_state.remove_surface(index);
                }
            }
        }

        for (surface_index, node_index, tab_index) in self.to_detach.drain(..).rev() {
            let mouse_pos = ui.input(|input| input.pointer.hover_pos());
            let _new_surface = self.dock_state.detach_tab(
                surface_index,
                node_index,
                tab_index,
                mouse_pos.unwrap_or(Pos2::ZERO),
            );
        }

        if let Some(focused) = self.new_focused {
            self.dock_state.set_focused_node_and_surface(focused);
        }

        if let (Some(source), Some(hover)) = (self.drag_data, self.hover_data) {
            let (dst_surf, dst_node) = hover.dst.break_down();
            let style = self.style.as_ref().unwrap();
            state.set_drag_and_drop(source.clone(), hover, ui.ctx(), style);
            match source.src {
                DropPosition::Tab(src_surf, src_node, src_tab) => {
                    let (empty_destination_surface, valid_move) = {
                        //empty roots don't have destination nodes
                        match dst_node {
                            Some(dst_node) => {
                                let src = &self.dock_state[src_surf][src_node];
                                let dst = &self.dock_state[dst_surf][dst_node];
                                let not_invalid = src.is_leaf()
                                    && dst.is_leaf()
                                    && ((src_surf, src_node) != (dst_surf, dst_node)
                                        || src.tabs_count() > 1);
                                (false, not_invalid)
                            }
                            None => {
                                let not_invalid = self.dock_state[src_surf][src_node].is_leaf();
                                (true, not_invalid)
                            }
                        }
                    };
                    if valid_move {
                        let tab_dst = {
                            let layer_id = LayerId::new(Order::Foreground, "foreground".into());
                            ui.with_layer_id(layer_id, |ui| {
                                state
                                    .drag
                                    .as_mut()
                                    .and_then(|drag_drop_state| {
                                        Some(drag_drop_state.resolve(
                                            ui,
                                            style,
                                            self.allowed_splits,
                                            false,
                                        ))
                                    })
                                    .unwrap()
                            })
                            .inner
                        };

                        if ui.input(|i| i.pointer.any_released()) {
                            //primarily used to allow/deny tabs to become/ be put in windows.
                            let allowed_to_move = {
                                if !tab_dst.is_window() {
                                    true
                                } else if let Node::Leaf { tabs, .. } =
                                    &mut self.dock_state[src_surf][src_node]
                                {
                                    tab_viewer.allowed_in_windows(&mut tabs[src_tab.0])
                                } else {
                                    //we've already run `is_leaf()` on this node.
                                    unreachable!()
                                }
                            };
                            if allowed_to_move {
                                if empty_destination_surface {
                                    let tab = self
                                        .dock_state
                                        .remove_tab((src_surf, src_node, src_tab))
                                        .unwrap();
                                    self.dock_state[dst_surf] = Tree::new(vec![tab]);

                                    if self.dock_state[src_surf].is_empty() && !src_surf.is_root() {
                                        self.dock_state.remove_surface(src_surf);
                                    }
                                } else {
                                    self.dock_state.move_tab(
                                        (src_surf, src_node, src_tab),
                                        (dst_surf, dst_node.unwrap(), tab_dst),
                                    );
                                }

                                state.window_fade = None;
                            }
                        }
                    }
                }
                _ => {
                    todo!("Inserting surfaces or entire nodes into dock is not yet supported");
                }
            }
        }
        state.store(ui.ctx(), self.id);
    }

    /// Show a single surface of a [`DockState`]
    fn show_surface_inside(
        &mut self,
        surf_index: SurfaceIndex,
        ui: &mut Ui,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        state: &mut State,
        fade_style: Option<(&Style, f32, SurfaceIndex)>,
    ) {
        if surf_index.is_root() {
            self.show_root_surface_inside(ui, tab_viewer, state);
        } else {
            self.show_window_surface(ui, surf_index, tab_viewer, state, fade_style);
        }
    }

    fn show_root_surface_inside(
        &mut self,
        ui: &mut Ui,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        state: &mut State,
    ) {
        let surf_index = SurfaceIndex::root();

        if self.dock_state[surf_index].is_empty() {
            let rect = ui.available_rect_before_wrap();
            let response = ui.allocate_rect(rect, Sense::hover());
            if response.hovered() {
                if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
                    self.hover_data = Some(HoverData {
                        rect,
                        dst: DropPosition::Surface(surf_index),
                        tab: None,
                    })
                }
            }
            //all for loops will be empty, so theres no point going through them.
            return;
        }

        // First compute all rect sizes in the node graph.
        let max_rect = self.allocate_area_for_root_node(ui, surf_index);
        for node_index in self.dock_state[surf_index].breadth_first_index_iter() {
            if self.dock_state[surf_index][node_index].is_parent() {
                self.compute_rect_sizes(ui, (surf_index, node_index), max_rect);
            }
        }

        // Then draw the bodies of each leafs.
        for node_index in self.dock_state[surf_index].breadth_first_index_iter() {
            if self.dock_state[surf_index][node_index].is_leaf() {
                self.show_leaf(ui, state, (surf_index, node_index), tab_viewer, None);
            }
        }

        // Finaly draw separators so that their "interaction zone" is above
        // bodies (see `SeparatorStyle::extra_interact_width`).
        for node_index in self.dock_state[surf_index].breadth_first_index_iter() {
            if self.dock_state[surf_index][node_index].is_parent() {
                self.show_separator(ui, (surf_index, node_index), None);
            }
        }
    }

    fn show_window_surface(
        &mut self,
        ui: &mut Ui,
        surf_index: SurfaceIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        state: &mut State,
        fade_style: Option<(&Style, f32, SurfaceIndex)>,
    ) -> (Option<Pos2>, Option<DragData>) {
        let title: WidgetText = format!("window {surf_index:?}").into();

        // TODO: let the user dock entire windows,
        // part 0 here goes into state.drag_start, 1 goes into self.drag_data
        // from there implement move_window properly.
        let window = {
            let mut window_constructor = egui::Window::new(title).title_bar(false);
            let state = self.dock_state.get_window_state_mut(surf_index).unwrap();
            if let Some(position) = state.next_position() {
                window_constructor = window_constructor.current_pos(position);
            }
            if let Some(size) = state.next_size() {
                window_constructor = window_constructor.fixed_size(size);
            }
            window_constructor
        };

        let (fade_factor, fade_style) = {
            if let Some((style, factor, surface_index)) = fade_style {
                if surface_index == surf_index {
                    (1.0, None)
                } else {
                    (factor, Some((style, factor)))
                }
            } else {
                (1.0, None)
            }
        };
        let mut frame = Frame::window(ui.style());
        frame.fill = frame.fill.linear_multiply(fade_factor);
        frame.stroke.color = frame.stroke.color.linear_multiply(fade_factor);
        frame.shadow.color = frame.shadow.color.linear_multiply(fade_factor);
        let response = window.frame(frame).show(ui.ctx(), |ui| {
            ui.scope(|ui| {
                if fade_factor != 1.0 {
                    fade_visuals(ui.visuals_mut(), fade_factor);
                }

                let max_rect = self.allocate_area_for_root_node(ui, surf_index);
                for node_index in self.dock_state[surf_index].breadth_first_index_iter() {
                    if self.dock_state[surf_index][node_index].is_parent() {
                        self.compute_rect_sizes(ui, (surf_index, node_index), max_rect);
                    }
                }

                // Then draw the bodies of each leafs.
                for node_index in self.dock_state[surf_index].breadth_first_index_iter() {
                    if self.dock_state[surf_index][node_index].is_leaf() {
                        self.show_leaf(ui, state, (surf_index, node_index), tab_viewer, fade_style);
                    }
                }

                // Finaly draw separators so that their "interaction zone" is above
                // bodies (see `SeparatorStyle::extra_interact_width`).
                let fade_style = fade_style.map(|(style, _)| style);
                for node_index in self.dock_state[surf_index].breadth_first_index_iter() {
                    if self.dock_state[surf_index][node_index].is_parent() {
                        self.show_separator(ui, (surf_index, node_index), fade_style);
                    }
                }
            });

            ui.layer_id()
        });

        let screen_rect = {
            if let Some(response) = response {
                if let Some(layer_id) = response.inner {
                    self.dock_state
                        .get_window_state_mut(surf_index)
                        .unwrap()
                        .layer_id = Some(layer_id);
                }
                response.response.rect
            } else {
                Rect::NOTHING
            }
        };

        //this is our janky way of detecting drags on the window
        //Some indicates that we were dragged, with just started specifiying if this is the first frame of drag.
        match self
            .dock_state
            .get_window_state_mut(surf_index)
            .unwrap()
            .dragged(ui.ctx(), screen_rect)
        {
            Some(just_started) => (
                just_started.then_some(screen_rect.min),
                Some(DragData {
                    src: DropPosition::Surface(surf_index),
                    rect: self.dock_state[surf_index][NodeIndex::root()]
                        .rect()
                        .unwrap(),
                }),
            ),
            None => (None, None),
        }
    }

    fn allocate_area_for_root_node(&mut self, ui: &mut Ui, surface: SurfaceIndex) -> Rect {
        let style = self.style.as_ref().unwrap();
        let mut rect = ui.available_rect_before_wrap();

        if let Some(margin) = style.dock_area_padding {
            rect.min += margin.left_top();
            rect.max -= margin.right_bottom();
        }

        ui.painter().rect_stroke(rect, style.rounding, style.border);
        rect = rect.expand(-style.border.width / 2.0);
        ui.allocate_rect(rect, Sense::hover());

        if self.dock_state[surface].is_empty() {
            return rect;
        }
        self.dock_state[surface][NodeIndex::root()].set_rect(rect);
        rect
    }

    fn compute_rect_sizes(
        &mut self,
        ui: &Ui,
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
        max_rect: Rect,
    ) {
        assert!(self.dock_state[surface_index][node_index].is_parent());

        let style = self.style.as_ref().unwrap();
        let pixels_per_point = ui.ctx().pixels_per_point();

        duplicate! {
            [
                orientation   dim_point  dim_size  left_of    right_of  ;
                [Horizontal]  [x]        [width]   [left_of]  [right_of];
                [Vertical]    [y]        [height]  [above]    [below]   ;
            ]
            if let Node::orientation { fraction, rect } = &mut self.dock_state[surface_index][node_index] {
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
                    let left = rect.intersect(Rect::[<everything_ left_of>](left_separator_border)).intersect(max_rect);
                    let right = rect.intersect(Rect::[<everything_ right_of>](right_separator_border)).intersect(max_rect);
                }

                self.dock_state[surface_index][node_index.left()].set_rect(left);
                self.dock_state[surface_index][node_index.right()].set_rect(right);
            }
        }
    }

    fn show_separator(
        &mut self,
        ui: &mut Ui,
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
        fade_style: Option<&Style>,
    ) {
        assert!(self.dock_state[surface_index][node_index].is_parent());

        let style = if let Some(fade_style) = fade_style {
            fade_style
        } else {
            self.style.as_ref().unwrap()
        };
        let pixels_per_point = ui.ctx().pixels_per_point();

        duplicate! {
            [
                orientation   dim_point  dim_size;
                [Horizontal]  [x]        [width] ;
                [Vertical]    [y]        [height];
            ]
            if let Node::orientation { fraction, ref rect } = &mut self.dock_state[surface_index][node_index] {
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
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        fade_style: Option<(&Style, f32)>,
    ) {
        assert!(self.dock_state[surface_index][node_index].is_leaf());

        let rect = {
            let Node::Leaf { rect, .. } = &mut self.dock_state[surface_index][node_index] else {
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

        let tabbar_response = self.tab_bar(
            ui,
            state,
            (surface_index, node_index),
            tab_viewer,
            fade_style.map(|(style, _)| style),
        );
        self.tab_body(
            ui,
            state,
            (surface_index, node_index),
            tab_viewer,
            spacing,
            tabbar_response,
            fade_style,
        );

        let Node::Leaf { tabs, .. } = &mut self.dock_state[surface_index][node_index] else {
            unreachable!()
        };
        for (tab_index, tab) in tabs.iter_mut().enumerate() {
            if tab_viewer.force_close(tab) {
                self.to_remove
                    .push((surface_index, node_index, TabIndex(tab_index)).into());
            }
        }
    }

    fn tab_bar(
        &mut self,
        ui: &mut Ui,
        state: &mut State,
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        fade_style: Option<&Style>,
    ) -> Response {
        assert!(self.dock_state[surface_index][node_index].is_leaf());

        let style = if let Some(fade) = fade_style {
            fade
        } else {
            self.style.as_ref().unwrap()
        };
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
            let Node::Leaf { tabs, scroll, .. } = &mut self.dock_state[surface_index][node_index]
            else {
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
                (surface_index, node_index),
                tab_viewer,
                tabbar_outer_rect,
                prefered_width,
                fade_style,
            );

            // Draw hline from tab end to edge of tabbar
            let px = ui.ctx().pixels_per_point().recip();
            let style = if let Some(fade) = fade_style {
                fade
            } else {
                self.style.as_ref().unwrap()
            };

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
                self.tab_plus(
                    ui,
                    node_index,
                    tab_viewer,
                    tabbar_outer_rect,
                    offset,
                    fade_style,
                );
            }

            tabs_ui.min_rect().width()
        };

        self.tab_bar_scroll(
            ui,
            (surface_index, node_index),
            actual_width,
            available_width,
            &tabbar_response,
            fade_style,
        );

        tabbar_response
    }

    #[allow(clippy::too_many_arguments)]
    fn tabs(
        &mut self,
        tabs_ui: &mut Ui,
        state: &mut State,
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        tabbar_outer_rect: Rect,
        preferred_width: Option<f32>,
        fade: Option<&Style>,
    ) {
        assert!(self.dock_state[surface_index][node_index].is_leaf());

        let focused = self.dock_state.focused_leaf();
        let tabs_len = {
            let Node::Leaf { tabs, .. } = &mut self.dock_state[surface_index][node_index] else {
                unreachable!()
            };
            tabs.len()
        };

        for tab_index in 0..tabs_len {
            let id = self
                .id
                .with((surface_index, "surface"))
                .with((node_index, "node"))
                .with((tab_index, "tab"));
            let tab_index = TabIndex(tab_index);
            let is_being_dragged =
                tabs_ui.memory(|mem| mem.is_being_dragged(id)) && self.draggable_tabs;

            if is_being_dragged {
                tabs_ui.output_mut(|o| o.cursor_icon = CursorIcon::Grabbing);
            }

            let (is_active, label, tab_style, closeable) = {
                let Node::Leaf { tabs, active, .. } =
                    &mut self.dock_state[surface_index][node_index]
                else {
                    unreachable!()
                };
                let style = if let Some(fade) = fade {
                    fade
                } else {
                    self.style.as_ref().unwrap()
                };
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
                            is_active && Some((surface_index, node_index)) == focused,
                            is_active,
                            is_being_dragged,
                            preferred_width,
                            show_close_button,
                            fade,
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

                        self.drag_data = Some(DragData {
                            src: DropPosition::Tab(surface_index, node_index, tab_index),
                            rect: self.dock_state[surface_index][node_index].rect().unwrap(),
                        });
                    }
                }

                response
            } else {
                let (mut response, close_response) = self.tab_title(
                    tabs_ui,
                    &tab_style,
                    id,
                    label,
                    is_active && Some((surface_index, node_index)) == focused,
                    is_active,
                    is_being_dragged,
                    preferred_width,
                    show_close_button,
                    fade,
                );

                let (close_hovered, close_clicked) = close_response
                    .map(|res| (res.hovered(), res.clicked()))
                    .unwrap_or_default();

                let sense = if close_hovered {
                    Sense::click()
                } else {
                    Sense::click_and_drag()
                };

                let is_lonely_tab = self.dock_state[surface_index].num_tabs() == 1;

                let Node::Leaf { tabs, active, .. } =
                    &mut self.dock_state[surface_index][node_index]
                else {
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
                        if (surface_index.is_root() || !is_lonely_tab)
                            && ui.button("Eject").clicked()
                        {
                            self.to_detach.push((surface_index, node_index, tab_index));
                            ui.close_menu();
                        }
                        if show_close_button && ui.button("Close").clicked() {
                            if tab_viewer.on_close(tab) {
                                self.to_remove
                                    .push((surface_index, node_index, tab_index).into());
                            } else {
                                *active = tab_index;
                                self.new_focused = Some((surface_index, node_index));
                            }
                            ui.close_menu();
                        }
                    });
                }

                if close_clicked {
                    if tab_viewer.on_close(tab) {
                        self.to_remove
                            .push((surface_index, node_index, tab_index).into());
                    } else {
                        *active = tab_index;
                        self.new_focused = Some((surface_index, node_index));
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
            let Node::Leaf { tabs, active, .. } = &mut self.dock_state[surface_index][node_index]
            else {
                unreachable!()
            };
            let tab = &mut tabs[tab_index.0];
            let style = match fade {
                Some(fade) => fade,
                None => self.style.as_ref().unwrap(),
            };
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
                self.new_focused = Some((surface_index, node_index));
            }

            if self.show_close_buttons && response.middle_clicked() {
                if tab_viewer.on_close(tab) {
                    self.to_remove
                        .push((surface_index, node_index, tab_index).into());
                } else {
                    *active = tab_index;
                    self.new_focused = Some((surface_index, node_index));
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
        fade_style: Option<&Style>,
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

        let style = if let Some(fade_style) = fade_style {
            fade_style
        } else {
            self.style.as_ref().unwrap()
        };
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
        fade: Option<&Style>,
    ) -> (Response, Option<Response>) {
        let style = if let Some(fade) = fade {
            fade
        } else {
            self.style.as_ref().unwrap()
        };
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
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
        actual_width: f32,
        available_width: f32,
        tabbar_response: &Response,
        fade_style: Option<&Style>,
    ) {
        let Node::Leaf { scroll, .. } = &mut self.dock_state[surface_index][node_index] else {
            unreachable!()
        };
        let overflow = (actual_width - available_width).at_least(0.0);
        let style = if let Some(fade_style) = fade_style {
            fade_style
        } else {
            self.style.as_ref().unwrap()
        };

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

    #[allow(clippy::too_many_arguments)]
    fn tab_body(
        &mut self,
        ui: &mut Ui,
        state: &State,
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        spacing: Vec2,
        tabbar_response: Response,
        fade: Option<(&Style, f32)>,
    ) {
        let (body_rect, _body_response) =
            ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::click_and_drag());

        let Node::Leaf {
            rect,
            tabs,
            active,
            viewport,
            ..
        } = &mut self.dock_state[surface_index][node_index]
        else {
            unreachable!();
        };

        if let Some(tab) = tabs.get_mut(active.0) {
            *viewport = body_rect;

            if ui.input(|i| i.pointer.any_click()) {
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    if body_rect.contains(pos) {
                        self.new_focused = Some((surface_index, node_index));
                    }
                }
            }

            let (style, fade_factor) = if let Some(fade) = fade {
                fade
            } else {
                (self.style.as_ref().unwrap(), 1.0)
            };
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
                            if fade_factor != 1.0 {
                                fade_visuals(ui.visuals_mut(), fade_factor);
                            }
                            let available_rect = ui.available_rect_before_wrap();
                            ui.expand_to_include_rect(available_rect);
                            tab_viewer.ui(ui, tab);
                        });
                });
            } else {
                Frame::none()
                    .inner_margin(tabs_style.tab_body.inner_margin)
                    .show(ui, |ui| {
                        if fade_factor != 1.0 {
                            fade_visuals(ui.visuals_mut(), fade_factor);
                        }
                        tab_viewer.ui(ui, tab);
                    });
            }
        }

        if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
            //prevent borrow checker issues
            let rect = *rect;

            // Use rect.contains instead of
            // response.hovered as the dragged tab covers
            // the underlying responses
            if state.drag_start.is_some() && rect.contains(pointer) {
                let on_title_bar = tabbar_response.rect.contains(pointer);
                let (dst, tab) = {
                    match self.tab_hover_rect {
                        Some((rect, tab_index)) => (
                            DropPosition::Tab(surface_index, node_index, tab_index),
                            Some(rect),
                        ),
                        None => (
                            DropPosition::Node(surface_index, node_index),
                            on_title_bar.then_some(tabbar_response.rect),
                        ),
                    }
                };

                self.hover_data = Some(HoverData { rect, dst, tab });
            }
        }
    }

    /// Returns some when windows are fading, and what surface index is being hovered over
    #[inline(always)]
    fn hovered_window_surface(
        &self,
        state: &mut State,
        hold_time: f32,
        ctx: &Context,
    ) -> Option<SurfaceIndex> {
        if let Some(hover_data) = &self.hover_data {
            if state.is_drag_drop_lock_some() {
                state.window_fade = Some((Instant::now(), hover_data.dst.surface_index()))
            }
        }
        state.window_fade.and_then(|(time, surface)| {
            ctx.request_repaint();
            (time.elapsed().as_secs_f32() < hold_time).then_some(surface)
        })
    }
}

impl<'tree, Tab> std::fmt::Debug for DockArea<'tree, Tab> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DockArea").finish_non_exhaustive()
    }
}
