use egui::{Sense, Ui};

use crate::{
    dock_area::{
        drag_and_drop::{HoverData, TreeComponent},
        state::State,
    },
    DockArea, SurfaceIndex, TabViewer,
};

impl<'tree, Tab> DockArea<'tree, Tab> {
    pub(super) fn show_root_surface_inside(
        &mut self,
        ui: &mut Ui,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        state: &mut State,
    ) {
        let surf_index = SurfaceIndex::main();

        if self.dock_state.main_surface().is_empty() {
            let rect = ui.available_rect_before_wrap();
            let response = ui.allocate_rect(rect, Sense::hover());
            if response.hovered() {
                self.hover_data = Some(HoverData {
                    rect,
                    dst: TreeComponent::Surface(surf_index),
                    tab: None,
                })
            }
            return;
        }

        self.render_nodes(ui, tab_viewer, state, surf_index, None);
    }
}
