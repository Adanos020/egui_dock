use egui::{Pos2, Rect, Vec2};

/// The state of a [`Surface::Window`](crate::Surface::Window) surface.
///
/// Doubles as a handle for the surface, allowing the user to set its size and position.
#[derive(Debug, Clone)]
pub struct WindowState {
    /// The [`Rect`] that this window was last taking up.
    screen_rect: Rect,

    /// Was this window dragged in the last frame?
    dragged: bool,

    /// The next position this window should be set to next frame.
    next_position: Option<Pos2>,

    /// The next size this window should be set to next frame.
    next_size: Option<Vec2>,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            screen_rect: Rect::NOTHING,
            dragged: false,
            next_position: None,
            next_size: None,
        }
    }
}

impl WindowState {
    /// Create a default window state.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Set the position for this window in screen coordinates.
    pub fn set_position(&mut self, position: Pos2) -> &mut Self {
        self.next_position = Some(position);
        self
    }

    /// Set the size of this window in egui points.
    pub fn set_size(&mut self, size: Vec2) -> &mut Self {
        self.next_size = Some(size);
        self
    }

    /// Get the [`Rect`] which this window occupies.
    /// If this window hasn't been shown before, this will be [`Rect::NOTHING`].
    pub fn rect(&self) -> Rect {
        self.screen_rect
    }

    /// Returns if this window is currently being dragged or not.
    pub fn dragged(&self) -> bool {
        self.dragged
    }

    pub(crate) fn next_position(&mut self) -> Option<Pos2> {
        self.next_position.take()
    }

    pub(crate) fn next_size(&mut self) -> Option<Vec2> {
        self.next_size.take()
    }

    /// Returns if window was dragged this frame, indicating with the inside bool if the drag was just started or not.
    pub(crate) fn was_dragged(&mut self, ctx: &egui::Context, new_rect: Rect) -> Option<bool> {
        // We need to make sure we check the size hasn't changed, since it indicates a resize rather than a drag.
        ((new_rect != self.screen_rect && new_rect.size() == self.screen_rect.size())
            || self.dragged)
            .then(|| {
                self.screen_rect = new_rect;
                let something_dragged = ctx.memory(|mem| mem.is_anything_being_dragged());

                // This enforces the drag start pattern which tabs follow, that is it's Some for the first frame of the drag, then none.
                let did_drag_start = something_dragged && !self.dragged;
                self.dragged = something_dragged;
                did_drag_start
            })
    }
}
