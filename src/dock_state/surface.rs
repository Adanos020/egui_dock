use crate::{Tree, WindowState};

/// A [`Surface`] is the highest level component in a [`DockState`](crate::DockState). [`Surface`]s represent an area
/// in which nodes are placed. Typically, you're only using one surface, which is the main surface. However, if you drag
/// a tab out in a way which creates a window, you also create a new surface in which nodes can appear.
pub enum Surface<Tab> {
    /// An empty surface, with nothing inside (practically, a null surface).
    Empty,

    /// The main surface of a [`DockState`](crate::DockState), only one should exist at surface index 0 at any one time.
    Main(Tree<Tab>),

    /// A windowed surface with a state.
    Window(Tree<Tab>, WindowState),
}

impl<Tab> Surface<Tab> {
    /// Is this surface [`Empty`](Self::Empty) (in practice null)?
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Get mutable access to the node tree of this surface.
    pub fn node_tree_mut(&mut self) -> Option<&mut Tree<Tab>> {
        match self {
            Surface::Empty => None,
            Surface::Main(tree) => Some(tree),
            Surface::Window(tree, _) => Some(tree),
        }
    }

    /// Get access to the node tree of this surface.
    pub fn node_tree(&self) -> Option<&Tree<Tab>> {
        match self {
            Surface::Empty => None,
            Surface::Main(tree) => Some(tree),
            Surface::Window(tree, _) => Some(tree),
        }
    }
}
