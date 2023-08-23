use std::time::Instant;

use egui::{
    CentralPanel, Color32, Context, CursorIcon, Frame, LayerId, Order, Pos2, Rect, Rounding, Sense,
    Ui, Vec2,
};

use duplicate::duplicate;
use paste::paste;

use crate::{
    utils::{expand_to_pixel, fade_dock_style, map_to_pixel},
    AllowedSplits, DockArea, Node, NodeIndex, OverlayType, Style, SurfaceIndex, TabDestination,
    TabViewer,
};

use super::{drag_and_drop::TreeComponent, state::State, tab_removal::TabRemoval};

mod leaf;
mod main_surface;
mod window_surface;

#[derive(Copy, Clone)]
struct LineSeparator {
    surface_index: SurfaceIndex,
    node_index: NodeIndex,
    separator: Rect,
    interact_rect: Rect,
}
struct CrossSeparator {
    related_line_seperators: Vec<LineSeparator>,
    interact_rect: Rect,
}

impl<'tree, Tab> DockArea<'tree, Tab> {
    /// Show the `DockArea` at the top level.
    ///
    /// This is the same as doing:
    ///
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
    ///
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
            .get_or_insert(Style::from_egui(ui.style().as_ref()));
        self.window_bounds.get_or_insert(ui.ctx().screen_rect());
        let mut state = State::load(ui.ctx(), self.id);
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

        for &surface_index in self.dock_state.valid_surface_indices().iter() {
            self.show_surface_inside(
                surface_index,
                ui,
                tab_viewer,
                &mut state,
                fade_style.as_ref().map(|(style, factor)| {
                    (style, *factor, fade_surface.unwrap_or(SurfaceIndex::main()))
                }),
            );
        }

        for index in self.to_remove.drain(..).rev() {
            match index {
                TabRemoval::Node(surface, node, tab) => {
                    self.dock_state[surface].remove_tab((node, tab));
                    if self.dock_state[surface].is_empty() && !surface.is_main() {
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
            self.dock_state.detach_tab(
                (surface_index, node_index, tab_index),
                Rect::from_min_size(
                    mouse_pos.unwrap_or(Pos2::ZERO),
                    self.dock_state[surface_index][node_index]
                        .rect()
                        .map_or(Vec2::new(100., 150.), |rect| rect.size()),
                ),
            );
        }

        if let Some(focused) = self.new_focused {
            self.dock_state.set_focused_node_and_surface(focused);
        }

        if let (Some(source), Some(hover)) = (self.drag_data.take(), self.hover_data.take()) {
            let style = self.style.as_ref().unwrap();
            state.set_drag_and_drop(source, hover, ui.ctx(), style);
            let tab_dst = {
                let layer_id = LayerId::new(Order::Foreground, "foreground".into());
                ui.with_layer_id(layer_id, |ui| {
                    self.show_drag_drop_overlay(ui, &mut state, tab_viewer)
                })
                .inner
            };
            if ui.input(|i| i.pointer.any_released()) {
                let source = {
                    match state.dnd.as_ref().unwrap().drag.src {
                        TreeComponent::Tab(src_surf, src_node, src_tab) => {
                            (src_surf, src_node, src_tab)
                        }
                        _ => todo!(
                            "collections of tabs, like nodes and surfaces can't be docked (yet)"
                        ),
                    }
                };
                if let Some(destination) = tab_dst {
                    self.dock_state.move_tab(source, destination);
                    state.reset_drag();
                }
            }
        }
        state.store(ui.ctx(), self.id);
    }

    /// Returns some when windows are fading, and what surface index is being hovered over
    #[inline(always)]
    fn hovered_window_surface(
        &self,
        state: &mut State,
        hold_time: f32,
        ctx: &Context,
    ) -> Option<SurfaceIndex> {
        if let Some(dnd_state) = &state.dnd {
            if dnd_state.is_locked(self.style.as_ref().unwrap(), ctx) {
                state.window_fade = Some((Instant::now(), dnd_state.hover.dst.surface_address()));
            }
        }

        state.window_fade.and_then(|(time, surface)| {
            ctx.request_repaint();
            (time.elapsed().as_secs_f32() < hold_time).then_some(surface)
        })
    }

    /// Resolve where a dragged tab would land given it's dropped this frame, returns `None` when the resulting drop is an invalid move.
    fn show_drag_drop_overlay(
        &mut self,
        ui: &Ui,
        state: &mut State,
        tab_viewer: &impl TabViewer<Tab = Tab>,
    ) -> Option<TabDestination> {
        let drag_state = state.dnd.as_mut().unwrap();
        let style = self.style.as_ref().unwrap();

        // If were hovering over ourselves, we're not moving anywhere.
        if drag_state.hover.dst.node_address() == drag_state.drag.src.node_address()
            && drag_state.is_on_title_bar()
        {
            return None;
        }

        let deserted_node = {
            match (
                drag_state.drag.src.node_address(),
                drag_state.hover.dst.node_address(),
            ) {
                ((src_surf, Some(src_node)), (dst_surf, Some(dst_node))) => {
                    src_surf == dst_surf
                        && src_node == dst_node
                        && self.dock_state[src_surf][src_node].tabs_count() == 1
                }
                _ => false,
            }
        };

        // Not all scenarios can house all splits.
        let restricted_splits = if drag_state.hover.dst.is_surface() || deserted_node {
            AllowedSplits::None
        } else {
            AllowedSplits::All
        };
        let allowed_splits = self.allowed_splits & restricted_splits;

        let allowed_in_window = match drag_state.drag.src {
            TreeComponent::Tab(surface, node, tab) => {
                let Node::Leaf { tabs, .. } = &mut self.dock_state[surface][node] else {
                    unreachable!("tab drags can only come from leaf nodes")
                };
                tab_viewer.allowed_in_windows(&mut tabs[tab.0])
            }
            _ => todo!("collections of tabs, like nodes or surfaces, can't be dragged! (yet)"),
        };

        if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
            drag_state.pointer = pointer;
        }

        let window_bounds = self.window_bounds.unwrap();
        if drag_state.is_on_title_bar()
            || style.overlay.overlay_type == OverlayType::HighlightedAreas
        {
            drag_state.resolve_traditional(
                ui,
                style,
                allowed_splits,
                allowed_in_window,
                window_bounds,
            )
        } else {
            drag_state.resolve_icon_based(
                ui,
                style,
                allowed_splits,
                allowed_in_window,
                window_bounds,
            )
        }
    }

    /// Show a single surface of a [`DockState`].
    fn show_surface_inside(
        &mut self,
        surf_index: SurfaceIndex,
        ui: &mut Ui,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        state: &mut State,
        fade_style: Option<(&Style, f32, SurfaceIndex)>,
    ) {
        if surf_index.is_main() {
            self.show_root_surface_inside(ui, tab_viewer, state);
        } else {
            self.show_window_surface(ui, surf_index, tab_viewer, state, fade_style);
        }
    }

    fn render_nodes(
        &mut self,
        ui: &mut Ui,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        state: &mut State,
        surf_index: SurfaceIndex,
        fade_style: Option<(&Style, f32)>,
    ) {
        // First compute all rect sizes in the node graph.
        let max_rect = self.allocate_area_for_root_node(ui, surf_index);
        for node_index in self.dock_state[surf_index].breadth_first_index_iter() {
            if self.dock_state[surf_index][node_index].is_parent() {
                self.compute_rect_sizes(ui, (surf_index, node_index), max_rect);
            }
        }

        // Then, draw the bodies of each leaves.
        for node_index in self.dock_state[surf_index].breadth_first_index_iter() {
            if self.dock_state[surf_index][node_index].is_leaf() {
                self.show_leaf(ui, state, (surf_index, node_index), tab_viewer, fade_style);
            }
        }

        // Finally, draw separators so that their "interaction zone" is above
        // bodies (see `SeparatorStyle::extra_interact_width`).
        let mut separators: Vec<LineSeparator> = vec![];
        let fade_style = fade_style.map(|(style, _)| style);
        for node_index in self.dock_state[surf_index].breadth_first_index_iter() {
            if self.dock_state[surf_index][node_index].is_parent() {
                separators.append(&mut self.show_separator(
                    ui,
                    (surf_index, node_index),
                    fade_style,
                ));
            }
        }

        // Finally draw cross section seperators
        self.show_cross_section_separators(ui, separators);
    }

    fn allocate_area_for_root_node(&mut self, ui: &mut Ui, surface: SurfaceIndex) -> Rect {
        let style = self.style.as_ref().unwrap();
        let mut rect = ui.available_rect_before_wrap();

        if let Some(margin) = style.dock_area_padding {
            rect.min += margin.left_top();
            rect.max -= margin.right_bottom();
        }

        ui.painter().rect_stroke(
            rect,
            style.main_surface_border_rounding,
            style.main_surface_border_stroke,
        );
        if surface == SurfaceIndex::main() {
            rect = rect.expand(-style.main_surface_border_stroke.width / 2.0);
        }
        ui.allocate_rect(rect, Sense::click());

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
                orientation   dim_point  dim_size  left_of    right_of;
                [Horizontal]  [x]        [width]   [left_of]  [right_of];
                [Vertical]    [y]        [height]  [above]    [below];
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
    ) -> Vec<LineSeparator> {
        assert!(self.dock_state[surface_index][node_index].is_parent());

        let style = fade_style.unwrap_or_else(|| self.style.as_ref().unwrap());
        let pixels_per_point = ui.ctx().pixels_per_point();
        let mut separators: Vec<LineSeparator> = vec![];

        duplicate! {
            [
                orientation   dim_point  dim_size;
                [Horizontal]  [x]        [width];
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

                separators.push(LineSeparator { surface_index,node_index, separator, interact_rect });

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

                // Update 'fraction' interaction after drawing separator,
                // otherwise it may overlap on other separator / bodies when
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

        separators
    }

    fn show_cross_section_separators(&mut self, ui: &mut Ui, separators: Vec<LineSeparator>) {
        let style = self.style.as_ref().unwrap();
        let mut cross_section_seperators: Vec<CrossSeparator> = vec![];

        // detect overlapping line separators
        for (i, sep1) in separators.iter().enumerate() {
            for sep2 in separators[i + 1..].iter() {
                let separator = sep1.interact_rect.intersect(sep2.interact_rect);
                let interact_rect = separator.expand(style.separator.extra_interact_width / 2.0);
                let cross_separator = CrossSeparator {
                    related_line_seperators: vec![*sep1, *sep2],
                    interact_rect,
                };
                cross_section_seperators.push(cross_separator);
            }
        }

        for separator in cross_section_seperators {
            let response = ui
                .allocate_rect(separator.interact_rect, Sense::click_and_drag())
                .on_hover_and_drag_cursor(CursorIcon::Move);

            // highlight all affected line separators
            if response.dragged() || response.hovered() {
                let color = if response.dragged() {
                    style.separator.color_dragged
                } else if response.hovered() {
                    style.separator.color_hovered
                } else {
                    style.separator.color_idle
                };

                for line_separator in separator.related_line_seperators.iter() {
                    ui.painter()
                        .rect_filled(line_separator.separator, Rounding::none(), color);
                }
            }

            for line_separator in separator.related_line_seperators.iter() {
                duplicate! {
                    [
                        orientation   dim_point  dim_size;
                        [Horizontal]  [x]        [width] ;
                        [Vertical]    [y]        [height];
                    ]
                    if let Node::orientation { fraction, ref rect } =
                        &mut self.dock_state[line_separator.surface_index][line_separator.node_index]
                    {
                        let midpoint = rect.min.dim_point + rect.dim_size() * *fraction;

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
                    }
                }
            }
        }
    }
}
