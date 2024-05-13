use egui::{Frame, Galley, TextStyle, TextWrapMode, Ui};

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
        let open = true;
        let window = self
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
                .into_galley(ui, Some(TextWrapMode::Extend), 0.0, TextStyle::Button)
        };

        // Fade window frame (if necessary)
        let mut frame = Frame::window(ui.style());
        if fade_factor != 1.0 {
            frame.fill = frame.fill.linear_multiply(fade_factor);
            frame.stroke.color = frame.stroke.color.linear_multiply(fade_factor);
            frame.shadow.color = frame.shadow.color.linear_multiply(fade_factor);
        }

        let tab_bar_height = self.style.as_ref().unwrap().tab_bar.height;
        if self.dock_state[surf_index].is_collapsed() {
            let height = self.dock_state[surf_index].collapsed_leaf_count() as f32 * tab_bar_height;
            window
                .resizable([true, false])
                .max_height(height)
                .min_height(height)
        } else {
            window
        }
        .frame(frame)
        .min_width(min_window_width(&title, ui.spacing().indent))
        .show(ui.ctx(), |ui| {
            //fade inner ui (if necessary)
            if fade_factor != 1.0 {
                fade_visuals(ui.visuals_mut(), fade_factor);
            }
            self.render_nodes(ui, tab_viewer, state, surf_index, fade_style);
        });

        if !open {
            self.to_remove.push(TabRemoval::Window(surf_index));
        }
    }
}

fn min_window_width(title: &Galley, button_width: f32) -> f32 {
    (button_width * 2.) + title.size().x
}
