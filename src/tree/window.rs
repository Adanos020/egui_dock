use egui::{Id, Pos2, Rect, Ui};

use crate::{TabSource, TabViewer, WindowIndex};

/// A window which can display a detached tab
/// /// A window which can display a detached tab
#[derive(Debug, Clone)]
pub struct TabWindow<Tab> {
    pub(crate) tab: Tab,
    screen_rect: Rect,
    dragged: bool,
    next_position: Option<Pos2>,
}
impl<Tab> TabWindow<Tab> {
    ///Create a new Window with a given tab shown inside
    pub fn new(content: Tab) -> Self {
        Self {
            tab: content,
            screen_rect: Rect::NOTHING,
            dragged: false,
            next_position: None,
        }
    }

    ///Destroy the Window and get the Tab back
    pub fn into_tab(self) -> Tab {
        self.tab
    }

    ///Give the window a new location on the Screen
    pub fn set_position(&mut self, position: Pos2) {
        self.next_position = Some(position);
    }

    ///Show the window
    pub(crate) fn show(
        &mut self,
        ui: &mut Ui,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        window_index: WindowIndex,
        open: Option<&mut bool>,
    ) -> (Option<Pos2>, Option<TabSource>) {
        let title = tab_viewer.title(&mut self.tab);
        let id = Id::new(title.text()).with("TabWindow");
        let mut window = egui::Window::new(title).id(id);
        if let Some(position) = self.next_position.take() {
            window = window.current_pos(position);
        }
        if let Some(open) = open {
            window = window.open(open);
        }

        let response = window.show(ui.ctx(), |ui| {
            tab_viewer.ui(ui, &mut self.tab);
        });

        let screen_rect = {
            if let Some(response) = response {
                response.response.rect
            } else {
                Rect::NOTHING
            }
        };

        //using the min of the screen rect makes sure we don't catch resizings.
        //using self.dragged as an alternative condition makes sure the drag doesn't stop if we stop moving mid drag.
        if screen_rect.min != self.screen_rect.min || self.dragged {
            self.screen_rect = screen_rect;
            let something_dragged = ui.memory(|mem| mem.is_anything_being_dragged());

            //this enforces the drag start pattern which tabs follow, that is it's Some for the first frame of the drag, then none.
            let drag_start = if something_dragged && !self.dragged {
                Some(screen_rect.min)
            } else {
                None
            };

            self.dragged = something_dragged;
            (
                drag_start,
                something_dragged.then_some(TabSource::Window(window_index)),
            )
        } else {
            (None, None)
        }
    }
}
