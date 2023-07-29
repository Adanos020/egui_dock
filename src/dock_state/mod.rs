/// Wrapper around indices to the collection of windows inside a [`Tree`].
pub mod surface_index;
pub mod tree;

/// Window states which tells floating tabs how to be displayed inside their window,
pub mod window_state;

pub use surface_index::SurfaceIndex;
pub use window_state::WindowState;

use egui::Rect;

use crate::{Node, NodeIndex, Split, TabDestination, TabIndex, Tree};

/// The basis for egui_dock
///
/// This tree starts with a collection of surfaces, that then breaks down into nodes, and then into tabs.
///
/// Indexing it will yield a [`Tree`](crate::Tree) which then contains nodes and tabs.

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
    ///An empty surface, with nothing inside (Practically, a Null surface)
    Empty,

    ///The root surface of a [`DockState`], only one should exist at surface index 0 at any one time.
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
        match self.surfaces[index.0].node_tree() {
            Some(tree) => tree,
            None => {
                panic!("There did not exist a tree at surface index {}", index.0);
            }
        }
    }
}

impl<Tab> std::ops::IndexMut<SurfaceIndex> for DockState<Tab> {
    #[inline(always)]
    fn index_mut(&mut self, index: SurfaceIndex) -> &mut Self::Output {
        match self.surfaces[index.0].node_tree_mut() {
            Some(tree) => tree,
            None => {
                panic!("There did not exist a tree at surface index {}", index.0);
            }
        }
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

    /// Get the [`WindowState`] which corresponds to a [`SurfaceIndex`]
    ///
    /// Returns None if the surface is an [`Empty`](crate::Surface::Empty), [`Root`](crate::Surface::Root), or doesn't exist.
    pub fn get_window_state_mut(&mut self, surface: SurfaceIndex) -> Option<&mut WindowState> {
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

    ///Get an exclusive borrow to the raw surface from a surface index
    #[inline]
    pub fn get_surface_mut(&mut self, surface: SurfaceIndex) -> Option<&mut Surface<Tab>> {
        self.surfaces.get_mut(surface.0)
    }

    ///Get a shared borrow to the raw surface from a surface index
    #[inline]
    pub fn get_surface(&self, surface: SurfaceIndex) -> Option<&Surface<Tab>> {
        self.surfaces.get(surface.0)
    }

    /// Returns true if the specified surface exists and isn't [`Empty`](crate::Surface::Empty)
    #[inline]
    pub fn is_surface_valid(&self, surface_index: SurfaceIndex) -> bool {
        self.surfaces
            .get(surface_index.0)
            .map_or(false, |surface| !surface.is_empty())
    }

    /// Returns an [`Iterator`](std::iter::Iterator) of all valid [`SurfaceIndex`]es.

    #[inline]
    pub(crate) fn surface_index_iter(&self) -> impl Iterator<Item = SurfaceIndex> {
        //the collection and "re-itering" may seem odd here, but it is justified since the FilterMap uses &self.
        //if we didn't do this it could end up in borrow checker issues, since the iterator would restrict the use
        //of &mut self until it is dropped.
        (0..self.surfaces.len())
            .filter_map(|index| {
                self.is_surface_valid(SurfaceIndex(index))
                    .then_some(SurfaceIndex(index))
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Remove a surface based on it's [`SurfaceIndex`], returning it if it existed, otherwise returning `None`.
    /// # Panics
    /// Panics if you try to remove the root surface ( surface index 0 )
    ///
    pub fn remove_surface(&mut self, surface_index: SurfaceIndex) -> Option<Surface<Tab>> {
        assert_ne!(surface_index, SurfaceIndex::root());
        if surface_index.0 < self.surfaces.len() {
            self.focused_surface = Some(SurfaceIndex::root());
            let surface = {
                if surface_index.0 == self.surfaces.len() - 1 {
                    self.surfaces.pop()?
                } else {
                    std::mem::replace(self.surfaces.get_mut(surface_index.0)?, Surface::Empty)
                }
            };
            Some(surface)
        } else {
            None
        }
    }

    /// Sets the currently focused leaf to `node_index` on the root surface if the node at `node_index` is a leaf.
    #[inline]
    pub fn root_set_focused_node(&mut self, node_index: NodeIndex) {
        if let Some(Node::Leaf { .. }) = self[SurfaceIndex::root()].tree.get(node_index.0) {
            self.focused_surface = Some(SurfaceIndex::root());
            self[SurfaceIndex::root()].set_focused_node(node_index);
        } else {
            self.focused_surface = None;
        }
    }

    /// Sets which is the active tab within a specific node on the root surface.
    #[inline]
    pub fn set_active_tab(
        &mut self,
        surface_index: SurfaceIndex,
        node_index: NodeIndex,
        tab_index: TabIndex,
    ) {
        if let Some(Node::Leaf { active, .. }) = self[surface_index].tree.get_mut(node_index.0) {
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
        if self.is_surface_valid(surface_index) {
            if node_index.0 < self[surface_index].len() {
                //i don't want this code to be evaluated until im absolutely sure the surface index is valid
                if self[surface_index][node_index].is_leaf() {
                    self.focused_surface = Some(surface_index);
                    self[surface_index].set_focused_node(node_index);
                    return;
                }
            }
        }
        self.focused_surface = None;
    }

    /// Get an exclusive borrow to the tree at the root surface
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
    pub fn root_split_left(
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
    pub fn root_split_right(
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
    pub fn root_split_above(
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
    pub fn root_split_below(
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
                if src_surface == SurfaceIndex::root() {
                    state.set_size(rect.size() * 0.8);
                }

                state.set_position(position);
            }
            TabDestination::Insert(index) => self[dst_surface][dst_node].insert_tab(index, tab),
            TabDestination::Append => self[dst_surface][dst_node].append_tab(tab),
        };

        if self[src_surface][src_node].is_leaf() && self[src_surface][src_node].tabs_count() == 0 {
            self[src_surface].remove_leaf(src_node);
        }

        if self[src_surface].is_empty() && src_surface != SurfaceIndex::root() {
            self.remove_surface(src_surface);
        }
    }

    /// Currently focused leaf.
    #[inline]
    pub fn focused_leaf(&self) -> Option<(SurfaceIndex, NodeIndex)> {
        let surface = self.focused_surface?;
        self[surface].focused_leaf().map(|leaf| (surface, leaf))
    }

    /// Remove a tab at the specified surface, node, and tab index.
    /// This method will yield the removed tab, if it exists.
    pub fn remove_tab(
        &mut self,
        (surface_index, node_index, tab_index): (SurfaceIndex, NodeIndex, TabIndex),
    ) -> Option<Tab> {
        self[surface_index].remove_tab((node_index, tab_index))
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

    /// Adds a window which contains it's own set of nodes and tabs to the [`DockState`].
    ///
    /// Returns the [`SurfaceIndex`] corresponding to the window, which will never change under the windows lifetime.
    pub fn add_window(&mut self, tabs: Vec<Tab>) -> SurfaceIndex {
        let surface = Surface::Window(Tree::new(tabs), WindowState::new());
        let index = self.find_empty_surface_index();
        if index.0 < self.surfaces.len() {
            self.surfaces[index.0] = surface;
        } else {
            self.surfaces.push(surface);
        }
        index
    }

    ///Finds the first empty surface index which may be used.
    ///
    /// WARNING!: in cases where one isn't found, ``SurfaceIndex(self.surfaces.len())`` is used.
    /// therefore it's not inherently safe to index the [`DockState`] with this index, as it may panic.
    fn find_empty_surface_index(&self) -> SurfaceIndex {
        //find the first possible empty surface to insert our window into.
        //skip the first entry as it's always the root.
        for i in self.surface_index_iter() {
            if self.surfaces[i.0].is_empty() {
                return i;
            }
        }
        SurfaceIndex(self.surfaces.len())
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
    #[deprecated = "Use `iter_root_surface_nodes` or `iter_nodes` instead"]
    pub fn iter(&self) -> std::slice::Iter<'_, Node<Tab>> {
        self.iter_root_surface_nodes()
    }
    /// Returns an `Iterator` of the underlying collection of nodes on the root surface.
    pub fn iter_root_surface_nodes(&self) -> std::slice::Iter<'_, Node<Tab>> {
        self[SurfaceIndex::root()].iter()
    }

    /// Returns an `Iterator` of **all** underlying nodes in the tree.
    pub fn iter_nodes(&self) -> impl Iterator<Item = &Node<Tab>> {
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
    ///
    /// See also: [`find_root_surface_tab`](crate::dock_state::DockState::find_root_surface_tab)
    pub fn find_tab(&self, needle_tab: &Tab) -> Option<(SurfaceIndex, NodeIndex, TabIndex)> {
        for surface_index in self.surface_index_iter() {
            if !self.surfaces[surface_index.0].is_empty() {
                if let Some((node_index, tab_index)) = self[surface_index].find_tab(needle_tab) {
                    return Some((surface_index, node_index, tab_index));
                }
            }
        }
        None
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
