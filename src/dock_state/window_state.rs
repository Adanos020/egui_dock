use egui::{Id, Pos2, Rect, Vec2};

#[cfg(feature = "multi-viewport")]
use egui::ViewportBuilder;

#[cfg(not(feature = "multi-viewport"))]
use egui::Window;

/// The state of a [`Surface::Window`](crate::Surface::Window).
///
/// Doubles as a handle for the surface, allowing the user to set its size and position.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WindowState {
    /// The [`Rect`] that this window was last taking up.
    screen_rect: Option<Rect>,

    /// Was this window dragged in the last frame?
    dragged: bool,

    /// The next position this window should be set to next frame.
    next_position: Option<Pos2>,

    /// The next size this window should be set to next frame.
    next_size: Option<Vec2>,

    /// true the first frame this window is drawn.
    /// handles opening collapsing header, etc.
    new: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            screen_rect: None,
            dragged: false,
            next_position: None,
            next_size: None,
            new: true,
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
        // The reason why we're unwrapping an Option with a default value instead of
        // just storing Rect::NOTHING for the None variant is that deserializing Rect::NOTHING
        // with serde_json causes a panic, because f32::INFINITY serializes into null in JSON.
        self.screen_rect.unwrap_or(Rect::NOTHING)
    }

    /// Returns if this window is currently being dragged or not.
    pub fn dragged(&self) -> bool {
        self.dragged
    }

    #[inline(always)]
    pub(crate) fn next_position(&mut self) -> Option<Pos2> {
        self.next_position.take()
    }

    #[inline(always)]
    pub(crate) fn next_size(&mut self) -> Option<Vec2> {
        self.next_size.take()
    }

    #[cfg(feature = "multi-viewport")]
    pub(crate) fn create_window(&mut self, id: Id, bounds: Rect) -> (ViewportBuilder, bool) {
        let new = self.new;
        let mut viewport_builder = ViewportBuilder::default()
            .with_decorations(false)
            .with_resizable(true)
            .with_drag_and_drop(true);

        if let Some(position) = self.next_position() {
            viewport_builder = viewport_builder.with_position(position);
        }
        if let Some(size) = self.next_size() {
            viewport_builder = viewport_builder.with_inner_size(size);
        }

        self.new = false;
        (viewport_builder, new)
    }

    // The 'static in this case means that the `open` field is always `None`
    #[cfg(not(feature = "multi-viewport"))]
    pub(crate) fn create_window(&mut self, id: Id, bounds: Rect) -> (Window<'static>, bool) {
        let new = self.new;
        let mut window_constructor = Window::new("").id(id).constrain_to(bounds).title_bar(false);

        if let Some(position) = self.next_position() {
            window_constructor = window_constructor.current_pos(position);
        }
        if let Some(size) = self.next_size() {
            window_constructor = window_constructor.fixed_size(size);
        }
        self.new = false;
        (window_constructor, new)
    }
}
