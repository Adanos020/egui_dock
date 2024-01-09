use std::convert::identity;
use std::sync::Arc;

use egui::{
    CollapsingHeader, CollapsingResponse, Frame, Galley, Id, Layout, Rect, Response, Sense,
    TextStyle, Ui, Vec2, Widget,
};

use crate::{
    dock_area::{state::State, tab_removal::TabRemoval},
    utils::fade_visuals,
    DockArea, Node, Style, SurfaceIndex, TabViewer,
};

impl<'tree, Tab> DockArea<'tree, Tab> {
    pub(super) fn show_window_surface(
        &mut self,
        ui: &Ui,
        surf_index: SurfaceIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        state: &mut State,
        fade_style: Option<(&Style, f32, SurfaceIndex)>,
    ) {
        //construct egui window
        let id = format!("window {surf_index:?}").into();
        let bounds = self.window_bounds.unwrap();
        let mut open = true;
        let (window, new) = self
            .dock_state
            .get_window_state_mut(surf_index)
            .unwrap()
            .create_window(id, bounds);

        //calculate fading of the window (if any)
        let (fade_factor, fade_style) = match fade_style {
            Some((style, factor, surface_index)) => {
                if surface_index == surf_index {
                    (1.0, None)
                } else {
                    (factor, Some((style, factor)))
                }
            }
            None => (1.0, None),
        };

        // Get galley of currently selected node as a window title
        let title = {
            let node_id = self.dock_state[surf_index]
                .focused_leaf()
                .unwrap_or_else(|| {
                    for node_index in self.dock_state[surf_index].breadth_first_index_iter() {
                        if self.dock_state[surf_index][node_index].is_leaf() {
                            return node_index;
                        }
                    }
                    unreachable!("a window surface should never be empty")
                });
            let Node::Leaf { tabs, active, .. } = &mut self.dock_state[surf_index][node_id] else {
                unreachable!()
            };
            tab_viewer
                .title(&mut tabs[active.0])
                .color(ui.visuals().widgets.noninteractive.fg_stroke.color)
                .into_galley(ui, Some(false), 0.0, TextStyle::Button)
        };

        // Fade window frame (if necessary)
        let mut frame = Frame::window(ui.style());
        if fade_factor != 1.0 {
            frame.fill = frame.fill.linear_multiply(fade_factor);
            frame.stroke.color = frame.stroke.color.linear_multiply(fade_factor);
            frame.shadow.color = frame.shadow.color.linear_multiply(fade_factor);
        }

        window
            .frame(frame)
            .min_width(min_window_width(&title, ui.spacing().indent))
            .show(ui.ctx(), |ui| {
                //fade inner ui (if necessary)
                if fade_factor != 1.0 {
                    fade_visuals(ui.visuals_mut(), fade_factor);
                }

                let collapser_id = id.with("collapser");
                let collapser_state = new.then_some(true);
                let ch_res = self.show_window_body(
                    ui,
                    surf_index,
                    tab_viewer,
                    state,
                    fade_style,
                    collapser_state,
                    collapser_id,
                    title,
                );
                if self.show_window_close_buttons {
                    // Finds out if theres a reason for the close button to be disabled
                    // by iterating over the tree and finding if theres any non-closable nodes.
                    let disabled = !self.dock_state[surf_index]
                        .iter_mut()
                        .filter_map(|node| {
                            if let Node::Leaf { tabs, .. } = node {
                                Some(
                                    tabs.iter_mut()
                                        .map(|tab| tab_viewer.closeable(tab))
                                        .all(identity),
                                )
                            } else {
                                None
                            }
                        })
                        .all(identity);

                    self.show_close_button(ui, &mut open, ch_res, disabled);
                }
            });

        if !open {
            self.to_remove.push(TabRemoval::Window(surf_index));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn show_window_body(
        &mut self,
        ui: &mut Ui,
        surf_index: SurfaceIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        state: &mut State,
        fade_style: Option<(&Style, f32)>,
        open: Option<bool>,
        id: Id,
        title: Arc<Galley>,
    ) -> Option<CollapsingResponse<()>> {
        if self.show_window_collapse_buttons {
            let ch_response = CollapsingHeader::new("")
                .id_source(id)
                .open(open)
                .show_unindented(ui, |ui| {
                    ui.set_min_size(Vec2::splat(100.0));
                    self.render_nodes(ui, tab_viewer, state, surf_index, fade_style);
                });
            ui.set_width(min_window_width(&title, ui.spacing().indent));

            if ch_response.fully_closed() {
                let rect = Rect::from_min_size(
                    ch_response.header_response.rect.left_top(),
                    Vec2::new(
                        ui.min_rect().size().x,
                        ch_response.header_response.rect.height(),
                    ),
                );
                ui.painter().galley(
                    rect.center() - (title.size() * 0.5),
                    title,
                    ui.visuals().widgets.noninteractive.fg_stroke.color,
                );
            }
            Some(ch_response)
        } else {
            // in case we don't render with a collapsing header we need to make a "blank"
            // window head in preparation for adding the close button.
            if self.show_window_close_buttons {
                ui.add_space(ui.spacing().icon_width + ui.spacing().item_spacing.y);
            }
            self.render_nodes(ui, tab_viewer, state, surf_index, fade_style);
            None
        }
    }

    fn show_close_button(
        &mut self,
        ui: &mut Ui,
        open: &mut bool,
        collapse_response: Option<CollapsingResponse<()>>,
        disabled: bool,
    ) {
        let rect = {
            let (top_right, height) = match collapse_response {
                Some(collapse) => (
                    Rect::from_two_pos(
                        collapse.header_response.rect.right_top(),
                        ui.max_rect().right_top(),
                    )
                    .right_top(),
                    collapse.header_response.rect.height(),
                ),
                None => (ui.max_rect().right_top(), ui.spacing().icon_width),
            };

            Rect::from_min_size(top_right, Vec2::new(0.0, height))
        };
        let close_button = close_button(
            disabled.then_some(
                self.dock_state
                    .translations
                    .window
                    .close_button_tooltip
                    .as_str(),
            ),
        );
        ui.allocate_ui_at_rect(rect, |ui| {
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                ui.set_height(rect.height());
                if ui.add(close_button).clicked() {
                    *open = false;
                }
            });
        });
    }
}

fn min_window_width(title: &Galley, button_width: f32) -> f32 {
    (button_width * 2.) + title.size().x
}

fn close_button(disabled: Option<&str>) -> impl Widget + '_ {
    move |ui: &mut Ui| -> Response {
        let sense = disabled.map_or(Sense::click(), |_disabled| Sense::hover());

        //this is how CollapsableHeader decides on space,
        //don't know why it doesn't atually end up the same size.
        let size = Vec2::new(ui.spacing().indent, ui.spacing().icon_width);
        let (rect, res) = ui.allocate_exact_size(size, sense);

        let visuals = ui.style().interact(&res);
        let painter = ui.painter();

        let rect = Rect::from_center_size(
            rect.center(),
            Vec2::splat(rect.width().min(rect.height()) * 0.5),
        )
        .expand(visuals.expansion);

        let stroke = match disabled.is_some() {
            true => visuals.bg_stroke,
            false => visuals.fg_stroke,
        };
        painter.line_segment(
            [
                painter.round_pos_to_pixels(rect.left_top()),
                painter.round_pos_to_pixels(rect.right_bottom()),
            ],
            stroke,
        );
        painter.line_segment(
            [
                painter.round_pos_to_pixels(rect.right_top()),
                painter.round_pos_to_pixels(rect.left_bottom()),
            ],
            stroke,
        );
        match disabled {
            Some(reason) => res.on_hover_text(reason),
            None => res,
        }
    }
}
