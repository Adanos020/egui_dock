pub mod tree;
pub use tree::*;

use egui::Rect;

use crate::{
    window_state::WindowState, Node, NodeIndex, Split, SurfaceIndex, TabDestination, TabIndex, Tree,
};

/// The basis for egui_dock
/// This tree starts with a collection of surfaces, that then breaks down into nodes, and then into tabs.
/// Indexing it will yield the NodeTree inside the surface which was indexed (panicing if it doesn't exist)
pub struct DockState<Tab> {
    surfaces: Vec<Surface<Tab>>,
    //part of the tree which is in focus
    focused_surface: Option<SurfaceIndex>,
}
/// A surface is the highest level component in a [`DockState`]
/// [`Surface`]s represent an area in which nodes are placed,
///  Typically you're only using one surface, which is the root surface,
/// However if you drag a tab out in a way which creates a window,
/// you also create a new surface in which nodes can appear.
pub enum Surface<Tab> {
    ///An empty surface, with nothing inside
    Empty,

    ///The root surface
    Root(Tree<Tab>),

    ///A windowed surface with a state
    Window(Tree<Tab>, WindowState),
}
impl<Tab> Surface<Tab> {
    ///Is this surface Empty? (in practice null)
    pub const fn is_empty(&self) -> bool {
        if let Self::Empty = self {
            true
        } else {
            false
        }
    }

    /// Get mutable access to the node tree of this surface
    pub fn node_tree_mut(&mut self) -> Option<&mut Tree<Tab>> {
        match self {
            Surface::Empty => None,
            Surface::Root(tree) => Some(tree),
            Surface::Window(tree, _) => Some(tree),
        }
    }

    ///Get access to the node tree of this surface
    pub fn node_tree(&self) -> Option<&Tree<Tab>> {
        match self {
            Surface::Empty => None,
            Surface::Root(tree) => Some(tree),
            Surface::Window(tree, _) => Some(tree),
        }
    }
}
impl<Tab> std::ops::Index<SurfaceIndex> for DockState<Tab> {
    type Output = Tree<Tab>;

    #[inline(always)]
    fn index(&self, index: SurfaceIndex) -> &Self::Output {
        self.surfaces[index.0]
            .node_tree()
            .expect("index doesn't point to a surface!")
    }
}

impl<Tab> std::ops::IndexMut<SurfaceIndex> for DockState<Tab> {
    #[inline(always)]
    fn index_mut(&mut self, index: SurfaceIndex) -> &mut Self::Output {
        self.surfaces[index.0]
            .node_tree_mut()
            .expect("index doesn't point to a surface!")
    }
}

impl<Tab> DockState<Tab> {
    ///Access to the root node tree
    pub fn root(&self) -> &Tree<Tab> {
        &self[SurfaceIndex::root()]
    }

    ///Access to the root node tree
    pub fn root_mut(&mut self) -> &mut Tree<Tab> {
        &mut self[SurfaceIndex::root()]
    }

    ///Create a new tree with these tabs at the root
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self {
            surfaces: vec![Surface::Root(Tree::new(tabs))],
            focused_surface: None,
        }
    }

    pub(crate) fn get_window_state_mut(
        &mut self,
        surface: SurfaceIndex,
    ) -> Option<&mut WindowState> {
        if let Surface::Window(_, state) = &mut self.surfaces[surface.0] {
            Some(state)
        } else {
            None
        }
    }
    /// Returns the viewport `Rect` and the `Tab` inside the focused leaf node or `None` if it does not exist.
    #[inline]
    pub fn find_active_focused(&mut self) -> Option<(Rect, &mut Tab)> {
        self.focused_surface
            .and_then(|surface| self[surface].find_active_focused())
    }

    ///Get the mutable borrow to the raw surface from a surface index
    pub fn get_surface_mut(&mut self, surface: SurfaceIndex) -> Option<&mut Surface<Tab>> {
        self.surfaces.get_mut(surface.0)
    }

    /// Returns an `Iterator` of all valid [`SurfaceIndex`]es.
    #[inline]
    pub(crate) fn surface_index_iter(&self) -> impl Iterator<Item = SurfaceIndex> {
        (0..self.surfaces.len()).map(SurfaceIndex)
    }

    /// Remove a surface based on it's [`SurfaceIndex`], returning it if it existed, otherwise returning `None`.
    /// # Panics
    /// Panics if you try to remove the root surface *( surface index 0 )*
    ///
    pub fn remove_surface(&mut self, surface_index: SurfaceIndex) -> Option<Surface<Tab>> {
        assert_ne!(surface_index, SurfaceIndex::root());
        if surface_index.0 < self.surfaces.len() {
            let surface = {
                if surface_index.0 == self.surfaces.len() - 1 {
                    self.surfaces.pop()?
                } else {
                    std::mem::replace(self.surfaces.get_mut(surface_index.0)?, Surface::Empty)
                }
            };
            self.focused_surface = Some(SurfaceIndex::root());
            Some(surface)
        } else {
            None
        }
    }

    /// Sets the currently focused leaf to `node_index` on the root surface if the node at `node_index` is a leaf.
    #[inline]
    pub fn set_focused_node(&mut self, node_index: NodeIndex) {
        if let Some(Node::Leaf { .. }) = self[SurfaceIndex::root()].tree.get(node_index.0) {
            self.focused_surface = Some(SurfaceIndex::root());
            self[SurfaceIndex::root()].set_focused_node(node_index);
        } else {
            self.focused_surface = None;
        }
    }

    /// Sets which is the active tab within a specific node on the root surface.
    #[inline]
    pub fn set_active_tab(&mut self, node_index: NodeIndex, tab_index: TabIndex) {
        if let Some(Node::Leaf { active, .. }) =
            self[SurfaceIndex::root()].tree.get_mut(node_index.0)
        {
            *active = tab_index;
        }
    }

    /// Sets which is the active tab within a specific node on the root surface.
    #[inline]
    pub fn set_active_tab_on_surface(
        &mut self,
        (surface_index, node_index, tab_index): (SurfaceIndex, NodeIndex, TabIndex),
    ) {
        if let Some(Node::Leaf { active, .. }) = self[surface_index].tree.get_mut(node_index.0) {
            *active = tab_index;
        }
    }

    /// Sets the currently focused leaf to `node_index` if the node at `node_index` is a leaf.
    #[inline]
    pub fn set_focused_node_and_surface(
        &mut self,
        (surface_index, node_index): (SurfaceIndex, NodeIndex),
    ) {
        if surface_index.0 >= self.surfaces.len() {
            return;
        }
        if let Some(Node::Leaf { .. }) = self[surface_index].tree.get(node_index.0) {
            self.focused_surface = Some(surface_index);
            self[surface_index].set_focused_node(node_index);
        } else {
            self.focused_surface = None;
        }
    }

    /// Get mutable access to the tree at the root surface
    pub fn root_surface_mut(&mut self) -> &mut Tree<Tab> {
        &mut self[SurfaceIndex::root()]
    }

    /// Creates two new nodes by splitting a given `parent` node at the root surface and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) has `tabs`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node is will occupy after the
    /// split.
    ///
    /// The new node is placed below the old node.
    ///
    /// Returns the indices of the old node and the new node.
    pub fn split_left(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self[SurfaceIndex::root()].split_left(parent, fraction, tabs)
    }

    /// Creates two new nodes by splitting a given `parent` node at the root surface and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) has `tabs`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node is will occupy after the
    /// split.
    ///
    /// The new node is placed below the old node.
    ///
    /// Returns the indices of the old node and the new node.
    pub fn split_right(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self[SurfaceIndex::root()].split_right(parent, fraction, tabs)
    }

    /// Creates two new nodes by splitting a given `parent` node at the root surface and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) has `tabs`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node is will occupy after the
    /// split.
    ///
    /// The new node is placed below the old node.
    ///
    /// Returns the indices of the old node and the new node.
    pub fn split_above(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self[SurfaceIndex::root()].split_above(parent, fraction, tabs)
    }

    /// Creates two new nodes by splitting a given `parent` node at the root surface and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) has `tabs`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node is will occupy after the
    /// split.
    ///
    /// The new node is placed below the old node.
    ///
    /// Returns the indices of the old node and the new node.
    pub fn split_below(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self[SurfaceIndex::root()].split_below(parent, fraction, tabs)
    }
    /// Moves a tab from a node to another node, you specify how the tab should
    /// be moved with [`TabDestination`].
    pub fn move_tab(
        &mut self,
        (src_surface, src_node, src_tab): (SurfaceIndex, NodeIndex, TabIndex),
        (dst_surface, dst_node, dst_tab): (SurfaceIndex, NodeIndex, TabDestination),
    ) {
        // Moving a single tab inside its own node is a no-op
        if src_surface == dst_surface
            && src_node == dst_node
            && self[src_surface][src_node].tabs_count() == 1
        {
            return;
        }

        // Call `Node::remove_tab` to avoid auto remove of the node by
        // `Tree::remove_tab` from Tree.

        let tab = self[src_surface][src_node].remove_tab(src_tab).unwrap();

        match dst_tab {
            TabDestination::Split(split) => {
                self[dst_surface].split(dst_node, split, 0.5, Node::leaf(tab));
            }
            TabDestination::Window(position) => {
                let surface_index = self.add_window(vec![tab]);

                let rect = {
                    match self[src_surface][src_node] {
                        Node::Empty => panic!(),
                        Node::Leaf { rect, .. } => rect,
                        Node::Vertical { rect, .. } => rect,
                        Node::Horizontal { rect, .. } => rect,
                    }
                };

                let state = self.get_window_state_mut(surface_index).unwrap();

                state.set_size(rect.size() * 0.8);
                state.set_position(position);
            }
            TabDestination::Insert(index) => self[dst_surface][dst_node].insert_tab(index, tab),
            TabDestination::Append => self[dst_surface][dst_node].append_tab(tab),
        };

        if self[src_surface][src_node].is_leaf() && self[src_surface][src_node].tabs_count() == 0 {
            self[src_surface].remove_leaf(src_node);
        }

        if self[src_surface].is_empty() {
            self.remove_surface(src_surface);
        }
    }

    /// Currently focused leaf.
    #[inline]
    pub fn focused_leaf(&self) -> Option<(SurfaceIndex, NodeIndex)> {
        let surface = self.focused_surface?;
        self[surface].focused_leaf().map(|leaf| (surface, leaf))
    }

    pub fn remove_tab(&mut self, tab: (NodeIndex, TabIndex)) -> Option<Tab> {
        self[SurfaceIndex::root()].remove_tab(tab)
    }
    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) has `tabs`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node is will occupy after the
    /// split.
    ///
    /// The new node is placed relatively to the old node, in the direction specified by `split`.
    ///
    /// Returns the indices of the old node and the new node.
    pub fn split(
        &mut self,
        surface: SurfaceIndex,
        parent: NodeIndex,
        split: Split,
        fraction: f32,
        new: Node<Tab>,
    ) -> [NodeIndex; 2] {
        let index = self[surface].split(parent, split, fraction, new);
        self.focused_surface = Some(surface);
        index
    }

    /// Add a window to the tree, which is disconnected from the rest of the nodes, returns it's surface index
    pub fn add_window(&mut self, tabs: Vec<Tab>) -> SurfaceIndex {
        let surface = Surface::Window(Tree::new(tabs), WindowState::new());

        //find the first possible empty surface to insert our window into.
        //skip the first entry as it's always the root.
        for (i, item) in self.surfaces[1..].iter().enumerate() {
            if item.is_empty() {
                self.surfaces[i] = surface;
                return SurfaceIndex(i);
            }
        }

        self.surfaces.push(surface);
        SurfaceIndex(self.surfaces.len() - 1)
    }

    /// Pushes `tab` to the currently focused leaf.
    ///
    /// If no leaf is focused it will be pushed to the first available leaf.
    ///
    /// If no leaf is available then a new leaf will be created.
    pub fn push_to_focused_leaf(&mut self, tab: Tab) {
        if let Some(surface) = self.focused_surface {
            self[surface].push_to_focused_leaf(tab)
        } else {
            self[SurfaceIndex::root()].push_to_focused_leaf(tab)
        }
    }
    /// Returns an `Iterator` of the underlying collection of nodes on the root surface.
    #[cfg(feature = "surfaces")]
    #[deprecated = "Use `iter_root_surface_nodes` or `iter_nodes` instead"]
    pub fn iter(&self) -> std::slice::Iter<'_, Node<Tab>> {
        self.iter_root_surface_nodes()
    }
    /// Returns an `Iterator` of the underlying collection of nodes on the root surface.
    pub fn iter_root_surface_nodes(&self) -> std::slice::Iter<'_, Node<Tab>> {
        self[SurfaceIndex::root()].iter()
    }
}

impl<'a, Tab> DockState<Tab> {
    /// Returns an `Iterator` of **all** underlying nodes in the tree.
    pub fn iter_nodes(&'a self) -> impl Iterator<Item = &'a Node<Tab>> {
        self.surfaces
            .iter()
            .filter_map(|tree| tree.node_tree())
            .flat_map(|nodes| nodes.iter())
    }
}

impl<Tab> DockState<Tab>
where
    Tab: PartialEq,
{
    /// Find the given tab.
    ///
    /// Returns which node the tab is in, and where in that node the tab is in.
    ///
    /// The returned [`NodeIndex`] will always point to a [`Node::Leaf`].
    ///
    /// In case there are several hits, only the first is returned.
    #[deprecated = "Use `find_root_surface_tab` instead"]
    pub fn find_tab(&self, needle_tab: &Tab) -> Option<(NodeIndex, TabIndex)> {
        self[SurfaceIndex::root()].find_tab(needle_tab)
    }
    /// Find the given tab on the root surface.
    ///
    /// Returns which node the tab is in, and where in that node the tab is in.
    ///
    /// The returned [`NodeIndex`] will always point to a [`Node::Leaf`].
    ///
    /// In case there are several hits, only the first is returned.
    pub fn find_root_surface_tab(&self, needle_tab: &Tab) -> Option<(NodeIndex, TabIndex)> {
        self[SurfaceIndex::root()].find_tab(needle_tab)
    }
}
