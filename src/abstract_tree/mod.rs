use egui::Rect;

use crate::{
    window::WindowState, Node, NodeIndex, NodeTree, Split, SurfaceIndex, TabDestination, TabIndex,
};

/// The basis for egui_dock
/// This tree starts with a collection of surfaces, that then breaks down into nodes, and then into tabs.
/// Indexing it will yield the NodeTree inside the surface which was indexed (panicing if it doesn't exist)
pub struct Tree<Tab> {
    surfaces: Vec<Surface<Tab>>,
    //part of the tree which is in focus
    focused_surface: Option<SurfaceIndex>,
}
/// A surface is the highest level component in a [`Tree`]
/// [`Surface`]s represent an area in which nodes are placed,
///  Typically you're only using one surface, which is the root surface,
/// However if you drag a tab out in a way which creates a window,
/// you also create a new surface in which nodes can appear.
pub enum Surface<Tab> {
    ///An empty surface, with nothing inside
    Empty,

    ///The root surface
    Root(NodeTree<Tab>),

    ///A windowed surface with a state
    Window(NodeTree<Tab>, WindowState),
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
    pub fn node_tree_mut(&mut self) -> Option<&mut NodeTree<Tab>> {
        match self {
            Surface::Empty => None,
            Surface::Root(tree) => Some(tree),
            Surface::Window(tree, _) => Some(tree),
        }
    }

    ///Get access to the node tree of this surface
    pub fn node_tree(&self) -> Option<&NodeTree<Tab>> {
        match self {
            Surface::Empty => None,
            Surface::Root(tree) => Some(tree),
            Surface::Window(tree, _) => Some(tree),
        }
    }
}
impl<Tab> std::ops::Index<SurfaceIndex> for Tree<Tab> {
    type Output = NodeTree<Tab>;

    #[inline(always)]
    fn index(&self, index: SurfaceIndex) -> &Self::Output {
        self.surfaces[index.0]
            .node_tree()
            .expect("index doesn't point to a surface!")
    }
}

impl<Tab> std::ops::IndexMut<SurfaceIndex> for Tree<Tab> {
    #[inline(always)]
    fn index_mut(&mut self, index: SurfaceIndex) -> &mut Self::Output {
        self.surfaces[index.0]
            .node_tree_mut()
            .expect("index doesn't point to a surface!")
    }
}

impl<Tab> Tree<Tab> {
    ///Access to the root node tree
    pub fn root(&self) -> &NodeTree<Tab> {
        &self[SurfaceIndex::root()]
    }

    ///Create a new tree with these tabs at the root
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self {
            surfaces: vec![Surface::Root(NodeTree::new(tabs))],
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
        assert!(surface_index != SurfaceIndex::root());
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

    /// Sets the currently focused leaf to `node_index` if the node at `node_index` is a leaf.
    #[inline]
    pub fn set_focused_node(&mut self, (surface_index, node_index): (SurfaceIndex, NodeIndex)) {
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
    pub fn root_surface_mut(&mut self) -> &mut NodeTree<Tab> {
        &mut self[SurfaceIndex::root()]
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
                self.get_window_state_mut(surface_index)
                    .unwrap()
                    .set_position(position);
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
        let surface = Surface::Window(NodeTree::new(tabs), WindowState::new());

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
}
