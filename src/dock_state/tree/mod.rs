//! Binary tree representing the relationships between [`Node`]s.
//!
//! # Implementation details
//!
//! The binary tree is stored in a [`Vec`] indexed by [`NodeIndex`].
//! The root is always at index *0*.
//! For a given node *n*:
//!  - left child of *n* will be at index *n * 2 + 1*.
//!  - right child of *n* will be at index *n * 2 + 2*.

/// Iterates over all tabs in a [`Tree`].
pub mod tab_iter;

/// Identifies a tab within a [`Node`].
pub mod tab_index;

/// Represents an abstract node of a [`Tree`].
pub mod node;

/// Wrapper around indices to the collection of nodes inside a [`Tree`].
pub mod node_index;

pub use node::Node;
pub use node_index::NodeIndex;
pub use tab_index::TabIndex;
pub use tab_iter::TabIter;

use egui::Rect;
use std::{
    fmt,
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

use crate::SurfaceIndex;

// ----------------------------------------------------------------------------

/// Direction in which a new node is created relatively to the parent node at which the split occurs.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[allow(missing_docs)]
pub enum Split {
    Left,
    Right,
    Above,
    Below,
}

impl Split {
    /// Returns whether the split is vertical.
    pub const fn is_top_bottom(self) -> bool {
        matches!(self, Split::Above | Split::Below)
    }

    /// Returns whether the split is horizontal.
    pub const fn is_left_right(self) -> bool {
        matches!(self, Split::Left | Split::Right)
    }
}

/// Specify how a tab should be added to a Node.
pub enum TabInsert {
    /// Split the node in the given direction.
    Split(Split),

    /// Insert the tab at the given index.
    Insert(TabIndex),

    /// Append the tab to the node.
    Append,
}

/// The destination for a tab which is being moved.
pub enum TabDestination {
    /// Move to a new window with this rect.
    Window(Rect),

    /// Move to a an existing node with this insertion.
    Node(SurfaceIndex, NodeIndex, TabInsert),

    /// Move to an empty surface.
    EmptySurface(SurfaceIndex),
}

impl From<(SurfaceIndex, NodeIndex, TabInsert)> for TabDestination {
    fn from(value: (SurfaceIndex, NodeIndex, TabInsert)) -> TabDestination {
        TabDestination::Node(value.0, value.1, value.2)
    }
}

impl From<SurfaceIndex> for TabDestination {
    fn from(value: SurfaceIndex) -> TabDestination {
        TabDestination::EmptySurface(value)
    }
}

impl TabDestination {
    /// Returns if this tab destination is a [`Window`](TabDestination::Window).
    pub fn is_window(&self) -> bool {
        matches!(self, Self::Window(_))
    }
}

/// Tree representing the relationships between [`Node`]s.
///
/// ## Implementation details
///
/// The tree is stored in a [`Vec`] indexed by [`NodeIndex`].
/// The root is always at index *0*.
/// A [`Node`] contains the index of its child nodes.
///
/// The nodes are not guarenteed to be stored in any particular order.
///
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Tree<Tab> {
    // Binary tree vector
    nodes: Vec<Node<Tab>>,
    focused_node: Option<NodeIndex>,
}

impl<Tab> fmt::Debug for Tree<Tab> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tree").finish_non_exhaustive()
    }
}

impl<Tab> Default for Tree<Tab> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            focused_node: None,
        }
    }
}

impl<Tab> Index<NodeIndex> for Tree<Tab> {
    type Output = Node<Tab>;

    #[inline(always)]
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl<Tab> IndexMut<NodeIndex> for Tree<Tab> {
    #[inline(always)]
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.nodes[index.0]
    }
}

impl<Tab> Tree<Tab> {
    /// Creates a new [`Tree`] with given `Vec` of `Tab`s in its root node.
    #[inline(always)]
    pub fn new(tabs: Vec<Tab>) -> Self {
        let root = Node::leaf_with(tabs);
        Self {
            nodes: vec![root],
            focused_node: None,
        }
    }

    /// Returns the viewport [`Rect`] and the `Tab` inside the first leaf node,
    /// or `None` if no leaf exists in the [`Tree`].
    #[inline]
    pub fn find_active(&mut self) -> Option<(Rect, &mut Tab)> {
        self.nodes.iter_mut().find_map(|node| match node {
            Node::Leaf {
                tabs,
                active,
                viewport,
                ..
            } => tabs.get_mut(active.0).map(|tab| (viewport.to_owned(), tab)),
            _ => None,
        })
    }

    /// Returns the number of nodes in the [`Tree`].
    ///
    /// This includes [`Empty`](Node::Empty) nodes.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the number of nodes in the tree is 0, otherwise `false`.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns an [`Iterator`] of the underlying collection of nodes.
    ///
    /// This includes [`Empty`](Node::Empty) nodes.
    ///
    /// This does not guarentee any particular traversal order.
    #[inline(always)]
    pub fn iter(&self) -> Iter<'_, Node<Tab>> {
        self.nodes.iter()
    }

    /// Returns [`IterMut`] of the underlying collection of nodes.
    ///
    /// This includes [`Empty`](Node::Empty) nodes.
    ///
    /// This does not guarentee any particular traversal order.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<'_, Node<Tab>> {
        self.nodes.iter_mut()
    }

    /// Returns an [`Iterator`] of [`NodeIndex`] ordered in a breadth first manner.
    #[inline(always)]
    pub(crate) fn breadth_first_index_iter(&self) -> impl Iterator<Item = NodeIndex> {
        if self.nodes.is_empty() {
            return Vec::new().into_iter();
        }

        let mut breadth_first = Vec::new();
        let mut next_level = vec![NodeIndex::root()];
        while !next_level.is_empty() {
            let nodes_to_visit = next_level;
            next_level = Vec::new();
            nodes_to_visit.into_iter().for_each(|i| {
                breadth_first.push(i);
                match self[i] {
                    Node::Horizontal { left, right, .. } => {
                        next_level.push(left);
                        next_level.push(right);
                    }
                    Node::Vertical { above, below, .. } => {
                        next_level.push(above);
                        next_level.push(below);
                    }
                    _ => (),
                }
            });
        }
        breadth_first.into_iter()
    }

    /// Returns an iterator over all tabs in arbitrary order.
    #[inline(always)]
    pub fn tabs(&self) -> TabIter<'_, Tab> {
        TabIter::new(self)
    }

    /// Counts and returns the number of tabs in the whole tree.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_dock::{DockState, NodeIndex, TabIndex};
    /// let mut dock_state = DockState::new(vec!["node 1", "node 2", "node 3"]);
    /// assert_eq!(dock_state.main_surface().num_tabs(), 3);
    ///
    /// let [a, b] = dock_state.main_surface_mut().split_left(NodeIndex::root(), 0.5, vec!["tab 4", "tab 5"]);
    /// assert_eq!(dock_state.main_surface().num_tabs(), 5);
    ///
    /// dock_state.main_surface_mut().remove_leaf(a);
    /// assert_eq!(dock_state.main_surface().num_tabs(), 2);
    /// ```
    #[inline]
    pub fn num_tabs(&self) -> usize {
        let mut count = 0;
        for node in self.nodes.iter() {
            if let Node::Leaf { tabs, .. } = node {
                count += tabs.len();
            }
        }
        count
    }

    /// Acquire a immutable borrow to the [`Node`] at the root of the tree.
    /// Returns [`None`] if the tree is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_dock::DockState;
    /// let mut dock_state = DockState::new(vec!["single tab"]);
    /// let root_node = dock_state.main_surface().root_node().unwrap();
    ///
    /// assert_eq!(root_node.tabs(), Some(["single tab"].as_slice()));
    /// ```
    pub fn root_node(&self) -> Option<&Node<Tab>> {
        self.nodes.first()
    }

    /// Acquire a mutable borrow to the [`Node`] at the root of the tree.
    /// Returns [`None`] if the tree is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_dock::{DockState, Node};
    /// let mut dock_state = DockState::new(vec!["single tab"]);
    /// let root_node = dock_state.main_surface_mut().root_node_mut().unwrap();
    /// if let Node::Leaf { tabs, ..} = root_node {
    ///     tabs.push("partner tab");
    /// }
    /// assert_eq!(root_node.tabs(), Some(["single tab", "partner tab"].as_slice()));
    /// ```
    pub fn root_node_mut(&mut self) -> Option<&mut Node<Tab>> {
        self.nodes.first_mut()
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed relatively to the old node, in the direction specified by `split`.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use egui_dock::{DockState, SurfaceIndex, NodeIndex, Split};
    /// let mut dock_state = DockState::new(vec!["tab 1", "tab 2"]);
    ///
    /// // At this point, the main surface only contains the leaf with tab 1 and 2.
    /// assert!(dock_state.main_surface().root_node().unwrap().is_leaf());
    ///
    /// // Split the node, giving 50% of the space to the new nodes and 50% to the old ones.
    /// let [old, new] = dock_state.main_surface_mut()
    ///     .split_tabs(NodeIndex::root(), Split::Below, 0.5, vec!["tab 3"]);
    ///
    /// assert!(dock_state.main_surface().root_node().unwrap().is_parent());
    /// assert!(dock_state[SurfaceIndex::main()][old].is_leaf());
    /// assert!(dock_state[SurfaceIndex::main()][new].is_leaf());
    /// ```
    #[inline(always)]
    pub fn split_tabs(
        &mut self,
        parent: NodeIndex,
        split: Split,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, split, fraction, Node::leaf_with(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// This is a shorthand for using `split_tabs` with [`Split::Above`].
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed *above* the old node.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline(always)]
    pub fn split_above(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Above, fraction, Node::leaf_with(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// This is a shorthand for using `split_tabs` with [`Split::Below`].
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed *below* the old node.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline(always)]
    pub fn split_below(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Below, fraction, Node::leaf_with(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// This is a shorthand for using `split_tabs` with [`Split::Left`].
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed to the *left* of the old node.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline(always)]
    pub fn split_left(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Left, fraction, Node::leaf_with(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) gets the `tabs`.
    ///
    /// This is a shorthand for using `split_tabs` with [`Split::Right`].
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed to the *right* of the old node.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline(always)]
    pub fn split_right(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Right, fraction, Node::leaf_with(tabs))
    }

    /// Creates two new nodes by splitting a given `parent` node and assigns them as its children. The first (old) node
    /// inherits content of the `parent` from before the split, and the second (new) uses `new`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node will occupy after the
    /// split.
    ///
    /// The new node is placed relatively to the old node, in the direction specified by `split`.
    ///
    /// Returns the indices of the old node and the new node.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    ///
    /// If `new` is an [`Empty`](Node::Empty), [`Horizontal`](Node::Horizontal) or [`Vertical`](Node::Vertical) node.
    ///
    /// If `new` is a [`Leaf`](Node::Leaf) node without any tabs.
    ///
    /// If `parent` points to an [`Empty`](Node::Empty) node.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use egui_dock::{DockState, SurfaceIndex, NodeIndex, Split, Node};
    /// let mut dock_state = DockState::new(vec!["tab 1", "tab 2"]);
    ///
    /// // At this point, the main surface only contains the leaf with tab 1 and 2.
    /// assert!(dock_state.main_surface().root_node().unwrap().is_leaf());
    ///
    /// // Splits the node, giving 50% of the space to the new nodes and 50% to the old ones.
    /// let [old, new] = dock_state.main_surface_mut()
    ///     .split(NodeIndex::root(), Split::Below, 0.5, Node::leaf_with(vec!["tab 3"]));
    ///
    /// assert!(dock_state.main_surface().root_node().unwrap().is_parent());
    /// assert!(dock_state[SurfaceIndex::main()][old].is_leaf());
    /// assert!(dock_state[SurfaceIndex::main()][new].is_leaf());
    /// ```
    pub fn split(
        &mut self,
        parent: NodeIndex,
        split: Split,
        fraction: f32,
        new: Node<Tab>,
    ) -> [NodeIndex; 2] {
        assert!(self[parent].is_leaf() || self[parent].is_parent());
        assert_ne!(new.tabs_count(), 0);
        assert!((0.0..=1.0).contains(&fraction));

        let mut insert = |node: Node<Tab>| {
            if let Some(empty_node_index) = self.nodes.iter().position(|n| matches!(n, Node::Empty))
            {
                self.nodes[empty_node_index] = node;
                NodeIndex(empty_node_index)
            } else {
                let index = NodeIndex(self.nodes.len());
                self.nodes.push(node);
                index
            }
        };

        // insert old an new node back into the tree.
        let new_index = insert(new);
        let old_index = insert(Node::Empty);
        self.nodes.swap(parent.0, old_index.0);

        // Set the parent node to the correct type
        let rect = Rect::NOTHING;
        self[parent] = match split {
            Split::Left => Node::Horizontal {
                rect,
                fraction,
                left: new_index,
                right: old_index,
            },
            Split::Right => Node::Horizontal {
                rect,
                fraction,
                left: old_index,
                right: new_index,
            },
            Split::Above => Node::Vertical {
                rect,
                fraction,
                above: new_index,
                below: old_index,
            },
            Split::Below => Node::Vertical {
                rect,
                fraction,
                above: old_index,
                below: new_index,
            },
        };

        self.focused_node = Some(new_index);

        [old_index, new_index]
    }

    /// Returns the viewport [`Rect`] and the `Tab` inside the focused leaf node or [`None`] if it does not exist.
    #[inline]
    pub fn find_active_focused(&mut self) -> Option<(Rect, &mut Tab)> {
        match self.focused_node.and_then(|idx| self.nodes.get_mut(idx.0)) {
            Some(Node::Leaf {
                tabs,
                active,
                viewport,
                ..
            }) => tabs.get_mut(active.0).map(|tab| (*viewport, tab)),
            _ => None,
        }
    }

    /// Gets the node index of currently focused leaf node; returns [`None`] when no leaf is focused.
    #[inline]
    pub fn focused_leaf(&self) -> Option<NodeIndex> {
        self.focused_node
    }

    /// Sets the currently focused leaf to `node_index` if the node at `node_index` is a leaf.
    ///
    /// This method will not never panic and instead removes focus from all nodes when given an invalid index.
    #[inline]
    pub fn set_focused_node(&mut self, node_index: NodeIndex) {
        self.focused_node = self
            .nodes
            .get(node_index.0)
            .filter(|node| node.is_leaf())
            .map(|_| node_index);
    }

    /// Removes the given node from the [`Tree`].
    ///
    /// # Panics
    ///
    /// - If the tree is empty.
    /// - If the node at index `node` is not a [`Leaf`](Node::Leaf).
    pub fn remove_leaf(&mut self, node: NodeIndex) {
        assert!(!self.is_empty());
        assert!(self[node].is_leaf());

        self[node] = Node::Empty;

        if Some(node) == self.focused_node {
            self.focused_node = None;
        }

        let Some(parent) = self.parent(node) else {
            self.nodes.clear();
            return;
        };
        let sibling = match self[parent] {
            Node::Horizontal { left, right, .. } if left == node => right,
            Node::Horizontal { left, right, .. } if right == node => left,
            Node::Vertical { above, below, .. } if above == node => below,
            Node::Vertical { above, below, .. } if below == node => above,
            _ => unreachable!("The parent of a node must be a split node"),
        };

        self[parent] = Node::Empty;
        self.nodes.swap(parent.0, sibling.0);
    }

    /// Pushes a tab to the first `Leaf` it finds in breadth first order.
    ///
    /// If an `Empty` node is encountered the node is transformed into a `Lead` and
    /// the tab is insert there.
    pub fn push_to_first_leaf(&mut self, tab: Tab) {
        for index in &mut self.breadth_first_index_iter() {
            match &mut self[index] {
                Node::Leaf { tabs, active, .. } => {
                    *active = TabIndex(tabs.len());
                    tabs.push(tab);
                    self.focused_node = Some(index);
                    return;
                }
                node @ Node::Empty => {
                    *node = Node::leaf(tab);
                    self.focused_node = Some(index);
                    return;
                }
                _ => {}
            }
        }
        assert!(self.nodes.is_empty());
        self.nodes.push(Node::leaf_with(vec![tab]));
        self.focused_node = Some(NodeIndex(0));
    }

    /// Sets which is the active tab within a specific node.
    #[inline]
    pub fn set_active_tab(&mut self, node_index: NodeIndex, tab_index: TabIndex) {
        if let Some(Node::Leaf { active, .. }) = self.nodes.get_mut(node_index.0) {
            *active = tab_index;
        }
    }

    /// Pushes `tab` to the currently focused leaf.
    ///
    /// If no leaf is focused it will be pushed to the first available leaf.
    ///
    /// If no leaf is available then a new leaf will be created.
    pub fn push_to_focused_leaf(&mut self, tab: Tab) {
        match self.focused_node {
            Some(node) => {
                if self.nodes.is_empty() {
                    self.nodes.push(Node::leaf(tab));
                    self.focused_node = Some(NodeIndex::root());
                } else {
                    match &mut self[node] {
                        Node::Empty => {
                            self[node] = Node::leaf(tab);
                            self.focused_node = Some(node);
                        }
                        Node::Leaf { tabs, active, .. } => {
                            *active = TabIndex(tabs.len());
                            tabs.push(tab);
                            self.focused_node = Some(node);
                        }
                        _ => {
                            self.push_to_first_leaf(tab);
                        }
                    }
                }
            }
            None => {
                if self.nodes.is_empty() {
                    self.nodes.push(Node::leaf(tab));
                    self.focused_node = Some(NodeIndex::root());
                } else {
                    self.push_to_first_leaf(tab);
                }
            }
        }
    }

    /// Removes the tab at the given ([`NodeIndex`], [`TabIndex`]) pair.
    ///
    /// If the node is emptied after the tab is removed, the node will also be removed.
    ///
    /// Returns the removed tab if it exists, or `None` otherwise.
    pub fn remove_tab(&mut self, (node_index, tab_index): (NodeIndex, TabIndex)) -> Option<Tab> {
        let node = &mut self[node_index];
        let tab = node.remove_tab(tab_index);
        if node.tabs_count() == 0 {
            self.remove_leaf(node_index);
        }
        tab
    }

    /// Returns a new Tree while mapping the tab type
    pub fn map_tabs<F, NewTab>(&self, function: F) -> Tree<NewTab>
    where
        F: FnMut(&Tab) -> NewTab + Clone,
    {
        let Tree {
            focused_node,
            nodes,
        } = self;
        let nodes = nodes
            .iter()
            .map(|node| node.map_tabs(function.clone()))
            .collect();
        Tree {
            nodes,
            focused_node: *focused_node,
        }
    }

    /// Returns the index of the node to the left of the given one.
    ///
    /// For vertical splits this returns the node above the split.
    ///
    /// If the given node is not a parent this will return `None`
    #[inline(always)]
    pub fn left_of(&self, node: NodeIndex) -> Option<NodeIndex> {
        match self[node] {
            Node::Horizontal { left, .. } => Some(left),
            Node::Vertical { above, .. } => Some(above),
            Node::Empty | Node::Leaf { .. } => None,
        }
    }

    /// Returns the index of the node to the right of the given one.
    ///
    /// For vertical splits this returns the node below the split.
    ///
    /// If the given node is not a parent this will return `None`
    #[inline(always)]
    pub fn right_of(&self, node: NodeIndex) -> Option<NodeIndex> {
        match self[node] {
            Node::Horizontal { right, .. } => Some(right),
            Node::Vertical { below, .. } => Some(below),
            Node::Empty | Node::Leaf { .. } => None,
        }
    }

    /// Returns the index of the parent node or `None` if given node is the root.
    #[inline]
    pub fn parent(&self, node: NodeIndex) -> Option<NodeIndex> {
        self.breadth_first_index_iter().find(|i| match self[*i] {
            Node::Horizontal { left, right, .. } => left == node || right == node,
            Node::Vertical { above, below, .. } => above == node || below == node,
            _ => false,
        })
    }
}

impl<Tab> Tree<Tab>
where
    Tab: PartialEq,
{
    /// Find the given tab.
    ///
    /// Returns in which node and where in that node the tab is.
    ///
    /// The returned [`NodeIndex`] will always point to a [`Node::Leaf`].
    ///
    /// In case there are several hits, only the first is returned.
    pub fn find_tab(&self, needle_tab: &Tab) -> Option<(NodeIndex, TabIndex)> {
        for (node_index, node) in self.nodes.iter().enumerate() {
            if let Some(tabs) = node.tabs() {
                for (tab_index, tab) in tabs.iter().enumerate() {
                    if tab == needle_tab {
                        return Some((node_index.into(), tab_index.into()));
                    }
                }
            }
        }
        None
    }
}
