use crate::{Node, NodeIndex, Tree, WindowState};

/// A [`Surface`] is the highest level component in a [`DockState`](crate::DockState). [`Surface`]s represent an area
/// in which nodes are placed. Typically, you're only using one surface, which is the main surface. However, if you drag
/// a tab out in a way which creates a window, you also create a new surface in which nodes can appear.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Surface<Tab> {
    /// The main surface of a [`DockState`](crate::DockState), only one should exist at surface index 0 at any one time.
    Main(Tree<Tab>),

    /// A windowed surface with a state.
    Window(Tree<Tab>, WindowState),
}

impl<Tab> Surface<Tab> {
    /// Get access to the node tree of this surface.
    pub fn node_tree(&self) -> &Tree<Tab> {
        match self {
            Surface::Main(tree) => tree,
            Surface::Window(tree, _) => tree,
        }
    }

    /// Get mutable access to the node tree of this surface.
    pub fn node_tree_mut(&mut self) -> &mut Tree<Tab> {
        match self {
            Surface::Main(tree) => tree,
            Surface::Window(tree, _) => tree,
        }
    }

    /// Returns an [`Iterator`] of nodes in this surface's tree.
    pub fn iter_nodes(&self) -> impl Iterator<Item = &Node<Tab>> {
        self.node_tree().iter()
    }

    /// Returns a mutable [`Iterator`] of nodes in this surface's tree.
    pub fn iter_nodes_mut(&mut self) -> impl Iterator<Item = &mut Node<Tab>> {
        self.node_tree_mut().iter_mut()
    }

    /// Returns an [`Iterator`] of **all** tabs in this surface's tree,
    /// and indices of containing nodes.
    pub fn iter_all_tabs(&self) -> impl Iterator<Item = (NodeIndex, &Tab)> {
        self.iter_nodes()
            .enumerate()
            .flat_map(|(index, node)| node.iter_tabs().map(move |tab| (NodeIndex(index), tab)))
    }

    /// Returns a mutable [`Iterator`] of **all** tabs in this surface's tree,
    /// and indices of containing nodes.
    pub fn iter_all_tabs_mut(&mut self) -> impl Iterator<Item = (NodeIndex, &mut Tab)> {
        self.iter_nodes_mut()
            .enumerate()
            .flat_map(|(index, node)| node.iter_tabs_mut().map(move |tab| (NodeIndex(index), tab)))
    }

    /// Returns a new [`Surface`] while mapping and filtering the tab type.
    /// TODO(LennysLounge): correct this doc comment.
    pub fn filter_map_tabs<F, NewTab>(&self, function: F) -> Surface<NewTab>
    where
        F: Clone + FnMut(&Tab) -> Option<NewTab>,
    {
        match self {
            Surface::Main(tree) => Surface::Main(tree.filter_map_tabs(function)),
            Surface::Window(tree, window_state) => {
                Surface::Window(tree.filter_map_tabs(function), window_state.clone())
            }
        }
    }

    /// Returns a new [`Surface`] while mapping the tab type.
    pub fn map_tabs<F, NewTab>(&self, mut function: F) -> Surface<NewTab>
    where
        F: Clone + FnMut(&Tab) -> NewTab,
    {
        self.filter_map_tabs(move |tab| Some(function(tab)))
    }

    /// Returns a new [`Surface`] while filtering the tab type.
    /// TODO(LennysLounge): correct this doc comment.
    pub fn filter_tabs<F>(&self, mut predicate: F) -> Surface<Tab>
    where
        F: Clone + FnMut(&Tab) -> bool,
        Tab: Clone,
    {
        self.filter_map_tabs(move |tab| predicate(tab).then(|| tab.clone()))
    }

    /// Removes all tabs for which `predicate` returns `false`.
    /// TODO(LennysLounge): correct this doc comment.
    pub fn retain_tabs<F>(&mut self, predicate: F)
    where
        F: Clone + FnMut(&mut Tab) -> bool,
    {
        let (Surface::Main(tree) | Surface::Window(tree, _)) = self;
        tree.retain_tabs(predicate);
    }
}
