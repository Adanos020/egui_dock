use egui::{CollapsingHeader, CollapsingResponse, Frame, Rect, Response, Sense, Ui, Vec2, Widget};

use crate::{
    dock_area::{state::State, tab_removal::TabRemoval},
    utils::fade_visuals,
    DockArea, Style, SurfaceIndex, TabViewer,
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

        //fade window frame (if neccesary)
        let mut frame = Frame::window(ui.style());
        if fade_factor != 1.0 {
            frame.fill = frame.fill.linear_multiply(fade_factor);
            frame.stroke.color = frame.stroke.color.linear_multiply(fade_factor);
            frame.shadow.color = frame.shadow.color.linear_multiply(fade_factor);
        }

        let collapser_id = id.with("collapser");
        window.frame(frame).show(ui.ctx(), |ui| {
            //fade inner ui (if neccesary)
            if fade_factor != 1.0 {
                fade_visuals(ui.visuals_mut(), fade_factor);
            }
            if self.show_window_heads {
                let ch_response = CollapsingHeader::new("")
                    .id_source(collapser_id)
                    .open(new.then_some(true))
                    .show_unindented(ui, |ui| {
                        self.render_nodes(ui, tab_viewer, state, surf_index, fade_style);
                    });
                let rect = close_button_rect(ui.spacing(), ch_response);
                if ui.add(close_button(rect)).clicked() {
                    open = false;
                }
            } else {
                self.render_nodes(ui, tab_viewer, state, surf_index, fade_style);
            }
        });

        if !open {
            self.to_remove.push(TabRemoval::Window(surf_index));
            ui.data_mut(|data| data.remove::<()>(collapser_id));
        }
    }
}

fn close_button_rect(spacing: &egui::style::Spacing, response: CollapsingResponse<()>) -> Rect {
    let button_size = Vec2::new(-spacing.indent, response.header_response.rect.height());
    let default_pos = response.header_response.rect.right_top();
    let pos = response.body_response.map_or(default_pos, |response| {
        Rect::from_two_pos(default_pos, response.rect.right_top()).right_top()
    });
    Rect::from_two_pos(pos, pos + button_size)
}

fn close_button(rect: Rect) -> impl Widget {
    move |ui: &mut Ui| -> Response {
        let res = ui.interact(rect, ui.next_auto_id(), Sense::click());
        let rect =
            Rect::from_center_size(rect.center(), Vec2::splat(rect.width().min(rect.height()) * 0.5));
        let visuals = ui.style().interact(&res);
        let painter = ui.painter();
        painter.line_segment(
            [
                painter.round_pos_to_pixels(rect.left_top()),
                painter.round_pos_to_pixels(rect.right_bottom()),
            ],
            visuals.fg_stroke,
        );
        painter.line_segment(
            [
                painter.round_pos_to_pixels(rect.right_top()),
                painter.round_pos_to_pixels(rect.left_bottom()),
            ],
            visuals.fg_stroke,
        );
        res
    }
}
