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
    /// Create a window from the tab

    /// Append the tab to the node.
    Append,
}
/// The destination for a tab which is being moved
pub enum TabDestination {
    /// Move to a new window with this rect
    Window(Rect),

    /// Move to a an existing node with this insetion
    Node(SurfaceIndex, NodeIndex, TabInsert),

    /// Move to an empty surface
    EmptySurface(SurfaceIndex),
}
impl Into<TabDestination> for (SurfaceIndex, NodeIndex, TabInsert) {
    fn into(self) -> TabDestination {
        TabDestination::Node(self.0, self.1, self.2)
    }
}
impl Into<TabDestination> for SurfaceIndex {
    fn into(self) -> TabDestination {
        TabDestination::EmptySurface(self)
    }
}

impl TabDestination {
    /// Returns if this tab destination is a [`Window`](TabDestination::Window).
    pub fn is_window(&self) -> bool {
        matches!(self, Self::Window(_))
    }
}

/// Binary tree representing the relationships between [`Node`]s.
///
/// # Implementation details
///
/// The binary tree is stored in a [`Vec`] indexed by [`NodeIndex`].
/// The root is always at index *0*.
/// For a given node *n*:
///  - left child of *n* will be at index *n * 2 + 1*.
///  - right child of *n* will be at index *n * 2 + 2*.
///
/// For "Horizontal" nodes:
///  - left child contains Left node.
///  - right child contains Right node.
///
/// For "Vertical" nodes:
///  - left child contains Top node.
///  - right child contains Bottom node.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Tree<Tab> {
    // Binary tree vector
    pub(super) tree: Vec<Node<Tab>>,
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
            tree: Vec::new(),
            focused_node: None,
        }
    }
}

impl<Tab> Index<NodeIndex> for Tree<Tab> {
    type Output = Node<Tab>;

    #[inline(always)]
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.tree[index.0]
    }
}

impl<Tab> IndexMut<NodeIndex> for Tree<Tab> {
    #[inline(always)]
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.tree[index.0]
    }
}

impl<Tab> Tree<Tab> {
    /// Creates a new `Tree` with given `Vec` of `Tab`s in its root node.
    #[inline(always)]
    pub fn new(tabs: Vec<Tab>) -> Self {
        let root = Node::leaf_with(tabs);
        Self {
            tree: vec![root],
            focused_node: None,
        }
    }

    /// Returns the viewport `Rect` and the `Tab` inside the first leaf node, or `None` of no leaf exists in the `Tree`.
    #[inline]
    pub fn find_active(&mut self) -> Option<(Rect, &mut Tab)> {
        self.tree.iter_mut().find_map(|node| match node {
            Node::Leaf {
                tabs,
                active,
                viewport,
                ..
            } => tabs.get_mut(active.0).map(|tab| (viewport.to_owned(), tab)),
            _ => None,
        })
    }

    /// Returns the number of nodes in the `Tree`.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    /// Returns `true` if the number of nodes in the tree is 0, `false` otherwise.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Returns an `Iterator` of the underlying collection of nodes.
    #[inline(always)]
    pub fn iter(&self) -> Iter<'_, Node<Tab>> {
        self.tree.iter()
    }

    /// Returns `IterMut` of the underlying collection of nodes.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<'_, Node<Tab>> {
        self.tree.iter_mut()
    }

    /// Returns an `Iterator` of [`NodeIndex`] ordered in a breadth first manner.
    #[inline(always)]
    pub(crate) fn breadth_first_index_iter(&self) -> impl Iterator<Item = NodeIndex> {
        (0..self.tree.len()).map(NodeIndex)
    }

    /// Returns an iterator over all tabs in arbitrary order
    #[inline(always)]
    pub fn tabs(&self) -> TabIter<'_, Tab> {
        TabIter::new(self)
    }

    /// Number of tabs
    #[inline]
    pub fn num_tabs(&self) -> usize {
        let mut count = 0;
        for node in self.tree.iter() {
            if let Node::Leaf { tabs, .. } = node {
                count += tabs.len();
            }
        }
        count
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
    /// inherits content of the `parent` from before the split, and the second (new) has `tabs`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node is will occupy after the
    /// split.
    ///
    /// The new node is placed above the old node.
    ///
    /// Returns the indices of the old node and the new node.
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
    /// inherits content of the `parent` from before the split, and the second (new) has `tabs`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node is will occupy after the
    /// split.
    ///
    /// The new node is placed below the old node.
    ///
    /// Returns the indices of the old node and the new node.
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
    /// inherits content of the `parent` from before the split, and the second (new) has `tabs`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node is will occupy after the
    /// split.
    ///
    /// The new node is placed to the left of the old node.
    ///
    /// Returns the indices of the old node and the new node.
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
    /// inherits content of the `parent` from before the split, and the second (new) has `tabs`.
    ///
    /// `fraction` (in range 0..=1) specifies how much of the `parent` node's area the old node is will occupy after the
    /// split.
    ///
    /// The new node is placed to the right of the old node.
    ///
    /// Returns the indices of the old node and the new node.
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
        parent: NodeIndex,
        split: Split,
        fraction: f32,
        new: Node<Tab>,
    ) -> [NodeIndex; 2] {
        let old = self[parent].split(split, fraction);
        assert!(old.is_leaf() || old.is_parent());

        // Resize vector to fit the new size of the binary tree.
        {
            let index = self.tree.iter().rposition(|n| !n.is_empty()).unwrap_or(0);
            let level = NodeIndex(index).level();
            self.tree
                .resize_with((1 << (level + 1)) - 1, || Node::Empty);
        }

        let index = match split {
            Split::Left | Split::Above => [parent.right(), parent.left()],
            Split::Right | Split::Below => [parent.left(), parent.right()],
        };

        // If the node were splitting is a parent, all it's children need to be moved.
        if old.is_parent() {
            let levels_to_move = NodeIndex(self.tree.len()).level() - index[0].level();

            // Level 0 is ourself, which is done when we assign self[index[0]] = old, so start at 1.
            for level in (1..levels_to_move).rev() {
                let old_start = parent.children_at(level).start;
                let new_start = index[0].children_at(level).start;
                let len = 1 << level;

                // Swap self[old_start..(old_start+len)] with self[new_start..(new_start+len)]
                // (the new part will only contain empty entries).
                let (old_range, new_range) = {
                    let (first_part, second_part) = self.tree.split_at_mut(new_start);
                    // Cut to length.
                    (
                        &mut first_part[old_start..old_start + len],
                        &mut second_part[..len],
                    )
                };
                old_range.swap_with_slice(new_range);
            }
        }

        self[index[0]] = old;
        self[index[1]] = new;

        self.focused_node = Some(index[1]);

        index
    }

    fn first_leaf(&self, top: NodeIndex) -> Option<NodeIndex> {
        let left = top.left();
        let right = top.right();
        match (self.tree.get(left.0), self.tree.get(right.0)) {
            (Some(&Node::Leaf { .. }), _) => Some(left),
            (_, Some(&Node::Leaf { .. })) => Some(right),

            (
                Some(Node::Horizontal { .. } | Node::Vertical { .. }),
                Some(Node::Horizontal { .. } | Node::Vertical { .. }),
            ) => self.first_leaf(left).or(self.first_leaf(right)),
            (Some(Node::Horizontal { .. } | Node::Vertical { .. }), _) => self.first_leaf(left),
            (_, Some(Node::Horizontal { .. } | Node::Vertical { .. })) => self.first_leaf(right),

            (None, None)
            | (Some(&Node::Empty), None)
            | (None, Some(&Node::Empty))
            | (Some(&Node::Empty), Some(&Node::Empty)) => None,
        }
    }

    /// Returns the viewport `Rect` and the `Tab` inside the focused leaf node or `None` if it does not exist.
    #[inline]
    pub fn find_active_focused(&mut self) -> Option<(Rect, &mut Tab)> {
        match self.focused_node.and_then(|idx| self.tree.get_mut(idx.0)) {
            Some(Node::Leaf {
                tabs,
                active,
                viewport,
                ..
            }) => tabs.get_mut(active.0).map(|tab| (*viewport, tab)),
            _ => None,
        }
    }

    /// Currently focused leaf.
    #[inline]
    pub fn focused_leaf(&self) -> Option<NodeIndex> {
        self.focused_node
    }

    /// Sets the currently focused leaf to `node_index` if the node at `node_index` is a leaf.
    #[inline]
    pub fn set_focused_node(&mut self, node_index: NodeIndex) {
        self.focused_node = self
            .tree
            .get(node_index.0)
            .filter(|node| node.is_leaf())
            .map(|_| node_index);
    }

    /// Removes the given node from the [`Tree`].
    pub fn remove_leaf(&mut self, node: NodeIndex) {
        assert!(self[node].is_leaf());

        let Some(parent) = node.parent() else {
            self.tree.clear();
            return;
        };

        if Some(node) == self.focused_node {
            self.focused_node = None;
            let mut node = node;
            while let Some(parent) = node.parent() {
                let next = if node.is_left() {
                    parent.right()
                } else {
                    parent.left()
                };
                if self.tree.get(next.0).is_some_and(|node| node.is_leaf()) {
                    self.focused_node = Some(next);
                    break;
                }
                if let Some(node) = self.first_leaf(next) {
                    self.focused_node = Some(node);
                    break;
                }
                node = parent;
            }
        }

        self[parent] = Node::Empty;
        self[node] = Node::Empty;

        let mut level = 0;

        if node.is_left() {
            'left_end: loop {
                let dst = parent.children_at(level);
                let src = parent.children_right(level + 1);
                for (dst, src) in dst.zip(src) {
                    if src >= self.tree.len() {
                        break 'left_end;
                    }
                    if Some(NodeIndex(src)) == self.focused_node {
                        self.focused_node = Some(NodeIndex(dst));
                    }
                    self.tree[dst] = std::mem::replace(&mut self.tree[src], Node::Empty);
                }
                level += 1;
            }
        } else {
            'right_end: loop {
                let dst = parent.children_at(level);
                let src = parent.children_left(level + 1);
                for (dst, src) in dst.zip(src) {
                    if src >= self.tree.len() {
                        break 'right_end;
                    }
                    if Some(NodeIndex(src)) == self.focused_node {
                        self.focused_node = Some(NodeIndex(dst));
                    }
                    self.tree[dst] = std::mem::replace(&mut self.tree[src], Node::Empty);
                }
                level += 1;
            }
        }
    }

    /// Push a tab to the first leaf it finds or creates a leaf if an empty spot is encountered.
    pub fn push_to_first_leaf(&mut self, tab: Tab) {
        for (index, node) in &mut self.tree.iter_mut().enumerate() {
            match node {
                Node::Leaf { tabs, active, .. } => {
                    *active = TabIndex(tabs.len());
                    tabs.push(tab);
                    self.focused_node = Some(NodeIndex(index));
                    return;
                }
                Node::Empty => {
                    *node = Node::leaf(tab);
                    self.focused_node = Some(NodeIndex(index));
                    return;
                }
                _ => {}
            }
        }
        assert!(self.tree.is_empty());
        self.tree.push(Node::leaf_with(vec![tab]));
        self.focused_node = Some(NodeIndex(0));
    }

    /// Sets which is the active tab within a specific node.
    #[inline]
    pub fn set_active_tab(&mut self, node_index: NodeIndex, tab_index: TabIndex) {
        if let Some(Node::Leaf { active, .. }) = self.tree.get_mut(node_index.0) {
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
                if self.tree.is_empty() {
                    self.tree.push(Node::leaf(tab));
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
                if self.tree.is_empty() {
                    self.tree.push(Node::leaf(tab));
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
}

impl<Tab> Tree<Tab>
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
    pub fn find_tab(&self, needle_tab: &Tab) -> Option<(NodeIndex, TabIndex)> {
        for (node_index, node) in self.tree.iter().enumerate() {
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
