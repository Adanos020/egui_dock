use std::ops::RangeInclusive;

use egui::{
    epaint::TextShape, lerp, pos2, vec2, Align, Align2, Button, CursorIcon, Frame, Id, LayerId,
    Layout, NumExt, Order, PointerButton, Rect, Response, Rounding, ScrollArea, Sense, Stroke,
    TextStyle, Ui, Vec2, WidgetText,
};

use crate::{
    dock_area::{
        drag_and_drop::{DragData, DragDropState, HoverData, TreeComponent},
        state::State,
    },
    utils::{fade_visuals, rect_set_size_centered, rect_stroke_box},
    DockArea, Node, NodeIndex, Style, SurfaceIndex, TabAddAlign, TabIndex, TabStyle, TabViewer,
};

use crate::popup::popup_under_widget;

impl<'tree, Tab> DockArea<'tree, Tab> {
    pub(super) fn show_leaf(
        &mut self,
        ui: &mut Ui,
        state: &mut State,
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        fade_style: Option<(&Style, f32)>,
    ) {
        assert!(self.dock_state[surface_index][node_index].is_leaf());

        let rect = self.dock_state[surface_index][node_index]
            .rect()
            .expect("This node must be a leaf");
        let ui = &mut ui.child_ui_with_id_source(
            rect,
            Layout::top_down_justified(Align::Min),
            (node_index, "node"),
        );
        let spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing = Vec2::ZERO;
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

        let tabs = self.dock_state[surface_index][node_index]
            .tabs_mut()
            .expect("This node must be a leaf here");
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

        let style = fade_style.unwrap_or_else(|| self.style.as_ref().unwrap());
        let (tabbar_outer_rect, tabbar_response) = ui.allocate_exact_size(
            vec2(ui.available_width(), style.tab_bar.height),
            Sense::click(),
        );
        ui.painter().rect_filled(
            tabbar_outer_rect,
            style.tab_bar.rounding,
            style.tab_bar.bg_fill,
        );

        let mut available_width = tabbar_outer_rect.width();

        // Reserve space for the add button at the end of the tab bar.
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

            // Desired size for tabs in "expanded" mode.
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

            // Draw hline from tab end to edge of tab bar.
            let px = ui.ctx().pixels_per_point().recip();
            let style = fade_style.unwrap_or_else(|| self.style.as_ref().unwrap());

            ui.painter().hline(
                tabs_ui.min_rect().right().min(clip_rect.right())..=tabbar_outer_rect.right(),
                tabbar_outer_rect.bottom() - px,
                (px, style.tab_bar.hline_color),
            );

            // Add button at the end of the tab bar.
            if self.show_add_buttons {
                let offset = match style.buttons.add_tab_align {
                    TabAddAlign::Left => {
                        (clip_rect.width() - tabs_ui.min_rect().width()).at_least(0.0)
                    }
                    TabAddAlign::Right => 0.0,
                };
                self.tab_plus(
                    ui,
                    surface_index,
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
            state,
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
            let tabs = self.dock_state[surface_index][node_index]
                .tabs()
                .expect("This node must be a leaf here");
            tabs.len()
        };

        for tab_index in 0..tabs_len {
            let id = self
                .id
                .with((surface_index, "surface"))
                .with((node_index, "node"))
                .with((tab_index, "tab"));
            let tab_index = TabIndex(tab_index);
            let is_being_dragged = tabs_ui.memory(|mem| mem.is_being_dragged(id))
                && tabs_ui.input(|i| i.pointer.primary_down() || i.pointer.primary_released())
                && self.draggable_tabs;

            if is_being_dragged {
                tabs_ui.output_mut(|o| o.cursor_icon = CursorIcon::Grabbing);
            }

            let (is_active, label, tab_style, closeable) = {
                let Node::Leaf { tabs, active, .. } =
                    &mut self.dock_state[surface_index][node_index]
                else {
                    unreachable!()
                };
                let style = fade.unwrap_or_else(|| self.style.as_ref().unwrap());
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
                            src: TreeComponent::Tab(surface_index, node_index, tab_index),
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

                if self.show_tab_name_on_hover {
                    let tabs = self.dock_state[surface_index][node_index]
                        .tabs_mut()
                        .expect("This node must be a leaf");
                    let tab = &mut tabs[tab_index.0];
                    response = response.on_hover_ui(|ui| {
                        ui.label(tab_viewer.title(tab));
                    });
                }

                if self.tab_context_menus {
                    let eject_button =
                        Button::new(&self.dock_state.translations.tab_context_menu.eject_button);
                    let close_button =
                        Button::new(&self.dock_state.translations.tab_context_menu.close_button);

                    let Node::Leaf { tabs, active, .. } =
                        &mut self.dock_state[surface_index][node_index]
                    else {
                        unreachable!()
                    };
                    let tab = &mut tabs[tab_index.0];

                    response = response.context_menu(|ui| {
                        tab_viewer.context_menu(ui, tab, surface_index, node_index);
                        if (surface_index.is_main() || !is_lonely_tab)
                            && ui.add(eject_button).clicked()
                        {
                            self.to_detach.push((surface_index, node_index, tab_index));
                            ui.close_menu();
                        }
                        if show_close_button && ui.add(close_button).clicked() {
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
                    let Node::Leaf { tabs, active, .. } =
                        &mut self.dock_state[surface_index][node_index]
                    else {
                        unreachable!()
                    };
                    let tab = &mut tabs[tab_index.0];

                    if tab_viewer.on_close(tab) {
                        self.to_remove
                            .push((surface_index, node_index, tab_index).into());
                    } else {
                        *active = tab_index;
                        self.new_focused = Some((surface_index, node_index));
                    }
                }
                let response = tabs_ui.interact(response.rect, id, sense);
                if response.drag_started_by(PointerButton::Primary) {
                    state.drag_start = response.hover_pos();
                }

                if let Some(pos) = state.last_hover_pos {
                    // Use response.rect.contains instead of
                    // response.hovered as the dragged tab covers
                    // the underlying tab
                    if state.drag_start.is_some() && response.rect.contains(pos) {
                        self.tab_hover_rect = Some((response.rect, tab_index));
                    }
                }

                response
            };

            // Paint hline below each tab unless its active (or option says otherwise).
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

            if self.show_close_buttons && tab_viewer.closeable(tab) && response.middle_clicked() {
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

    #[allow(clippy::too_many_arguments)]
    fn tab_plus(
        &mut self,
        ui: &mut Ui,
        surface_index: SurfaceIndex,
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

        let style = fade_style.unwrap_or_else(|| self.style.as_ref().unwrap());
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

        // Draw button left border.
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
            tab_viewer.add_popup(ui, surface_index, node_index);
        });

        if response.clicked() {
            if self.show_add_popup {
                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
            }
            tab_viewer.on_add(surface_index, node_index);
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
        let style = fade.unwrap_or_else(|| self.style.as_ref().unwrap());
        let galley = label.into_galley(ui, None, f32::INFINITY, TextStyle::Button);
        let x_spacing = 8.0;
        let text_width = galley.size().x + 2.0 * x_spacing;
        let close_button_size = if show_close_button {
            Style::TAB_CLOSE_BUTTON_SIZE.min(style.tab_bar.height)
        } else {
            0.0
        };

        // Compute total width of the tab bar.
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

        // Draw the full tab first and then the stroke on top to avoid the stroke
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
            // Make the tab name area connect with the tab ui area.
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
            let pos = Align2::CENTER_CENTER.pos_in_rect(&text_rect.shrink2(vec2(x_spacing, 0.0)));
            pos - galley.size() / 2.0
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

    #[allow(clippy::too_many_arguments)]
    fn tab_bar_scroll(
        &mut self,
        ui: &mut Ui,
        state: &mut State,
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
        let style = fade_style.unwrap_or_else(|| self.style.as_ref().unwrap());

        // Compare to 1.0 and not 0.0 to avoid drawing a scroll bar due
        // to floating point precision issue during tab drawing.
        if overflow > 1.0 {
            if style.tab_bar.show_scroll_bar_on_overflow {
                // Draw scroll bar
                let bar_height = 7.5;
                let (scroll_bar_rect, _scroll_bar_response) = ui.allocate_exact_size(
                    vec2(ui.available_width(), bar_height),
                    Sense::click_and_drag(),
                );

                // Compute scroll bar handle position and size.
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

                // Coefficient to apply to input displacements so that we move the scroll by the correct amount.
                let points_to_scroll_coefficient =
                    overflow / (scroll_bar_rect.width() - scroll_bar_handle_size);

                *scroll -= scroll_bar_handle_response.drag_delta().x * points_to_scroll_coefficient;

                if let Some(pos) = state.last_hover_pos {
                    if scroll_bar_rect.contains(pos) {
                        *scroll += ui.input(|i| i.scroll_delta.y + i.scroll_delta.x)
                            * points_to_scroll_coefficient;
                    }
                }

                // Draw the bar.
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

            // Handle user input.
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
            ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::hover());

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
                if let Some(pos) = state.last_hover_pos {
                    if body_rect.contains(pos) && Some(ui.layer_id()) == ui.ctx().layer_id_at(pos) {
                        self.new_focused = Some((surface_index, node_index));
                    }
                }
            }

            let (style, fade_factor) = fade.unwrap_or_else(|| (self.style.as_ref().unwrap(), 1.0));
            let tabs_styles = tab_viewer.tab_style_override(tab, &style.tab);

            let tabs_style = tabs_styles.as_ref().unwrap_or(&style.tab);

            if tab_viewer.clear_background(tab) {
                ui.painter()
                    .rect_filled(body_rect, 0.0, tabs_style.tab_body.bg_fill);
            }

            // Construct a new ui with the correct tab id.
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

            // Use initial spacing for ui.
            ui.spacing_mut().item_spacing = spacing;

            // Offset the background rectangle up to hide the top border behind the clip rect.
            // To avoid anti-aliasing lines when the stroke width is not divisible by two, we
            // need to calculate the effective anti-aliased stroke width.
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

            ScrollArea::new(tab_viewer.scroll_bars(tab)).show(ui, |ui| {
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
        }

        //change hover destination
        if let Some(pointer) = state.last_hover_pos {
            // Prevent borrow checker issues.
            let rect = rect.to_owned();

            //if the dragged tab isn't allowed in a window,
            //it's unneccesary to change the hover state
            let is_dragged_valid = match &state.dnd {
                Some(DragDropState {
                    drag: DragData { src, .. },
                    ..
                }) => match *src {
                    TreeComponent::Tab(d_surf, d_node, d_tab) => {
                        if let Node::Leaf { tabs, .. } = &mut self.dock_state[d_surf][d_node] {
                            tab_viewer.allowed_in_windows(&mut tabs[d_tab.0])
                                || surface_index == SurfaceIndex::main()
                        } else {
                            true
                        }
                    }
                    _ => unreachable!("collections of nodes can't be dragged (yet)"),
                },
                _ => true,
            };

            // Use rect.contains instead of response.hovered as the dragged tab covers
            // the underlying responses.
            if state.drag_start.is_some() && rect.contains(pointer) && is_dragged_valid {
                let on_title_bar = tabbar_response.rect.contains(pointer);
                let (dst, tab) = {
                    match self.tab_hover_rect {
                        Some((rect, tab_index)) => (
                            TreeComponent::Tab(surface_index, node_index, tab_index),
                            Some(rect),
                        ),
                        None => (
                            TreeComponent::Node(surface_index, node_index),
                            on_title_bar.then_some(tabbar_response.rect),
                        ),
                    }
                };

                self.hover_data = Some(HoverData { rect, dst, tab });
            }
        }
    }
}
