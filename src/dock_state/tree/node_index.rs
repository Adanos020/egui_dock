/// Wrapper around indices to the collection of nodes inside a [`Tree`](crate::Tree).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NodeIndex(pub usize);

impl From<usize> for NodeIndex {
    #[inline(always)]
    fn from(index: usize) -> Self {
        NodeIndex(index)
    }
}

impl NodeIndex {
    /// Returns the index of the root node.
    ///
    /// In the context of a [`Tree`](crate::Tree), this will be the node that contains all other nodes.
    ///
    /// # Examples
    ///
    /// Splitting the current tree in two.
    /// ```rust
    /// # use egui_dock::{DockState, NodeIndex};
    /// let mut dock_state = DockState::new(vec!["tab 1", "tab 2"]);
    /// let _ = dock_state.main_surface_mut().split_left(NodeIndex::root(), 0.5, vec!["tab 3", "tab 4"]);
    /// ```
    #[inline(always)]
    pub const fn root() -> Self {
        Self(0)
    }
}
