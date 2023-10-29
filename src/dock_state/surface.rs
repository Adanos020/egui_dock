use crate::{Node, NodeIndex, Tree, WindowState};

/// A [`Surface`] is the highest level component in a [`DockState`](crate::DockState). [`Surface`]s represent an area
/// in which nodes are placed. Typically, you're only using one surface, which is the main surface. However, if you drag
/// a tab out in a way which creates a window, you also create a new surface in which nodes can appear.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
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

    /// Get access to the node tree of this surface.
    pub fn node_tree(&self) -> Option<&Tree<Tab>> {
        match self {
            Surface::Empty => None,
            Surface::Main(tree) => Some(tree),
            Surface::Window(tree, _) => Some(tree),
        }
    }

    /// Get mutable access to the node tree of this surface.
    pub fn node_tree_mut(&mut self) -> Option<&mut Tree<Tab>> {
        match self {
            Surface::Empty => None,
            Surface::Main(tree) => Some(tree),
            Surface::Window(tree, _) => Some(tree),
        }
    }

    /// Returns an `Iterator` of nodes in this surface's tree.
    pub fn iter_nodes(&self) -> impl Iterator<Item = &Node<Tab>> {
        match self.node_tree() {
            Some(tree) => tree.iter(),
            None => core::slice::Iter::default(),
        }
    }

    /// Returns a mutable `Iterator` of nodes in this surface's tree.
    pub fn iter_nodes_mut(&mut self) -> impl Iterator<Item = &mut Node<Tab>> {
        match self.node_tree_mut() {
            Some(tree) => tree.iter_mut(),
            None => core::slice::IterMut::default(),
        }
    }

    /// Returns an `Iterator` of **all** tabs in this surface's tree,
    /// and indices of containing nodes.
    pub fn iter_all_tabs(&self) -> impl Iterator<Item = (NodeIndex, &Tab)> {
        self.iter_nodes()
            .enumerate()
            .filter_map(|(index, node)| {
                node.tabs()
                    .map(|tabs| tabs.iter().map(move |tab| (NodeIndex(index), tab)))
            })
            .flatten()
    }

    /// Returns a mutable `Iterator` of **all** tabs in this surface's tree,
    /// and indices of containing nodes.
    pub fn iter_all_tabs_mut(&mut self) -> impl Iterator<Item = (NodeIndex, &mut Tab)> {
        self.iter_nodes_mut()
            .enumerate()
            .filter_map(|(index, node)| {
                node.tabs_mut()
                    .map(|tabs| tabs.iter_mut().map(move |tab| (NodeIndex(index), tab)))
            })
            .flatten()
    }
}
