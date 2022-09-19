use egui::*;

/// Identifies a tab within a [`Node`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TabIndex(pub usize);

impl From<usize> for TabIndex {
    #[inline]
    fn from(index: usize) -> Self {
        TabIndex(index)
    }
}

// ----------------------------------------------------------------------------

/// Represents an abstract node of a `Tree`.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Node<Tab> {
    /// Empty node
    Empty,
    /// Contains the actual tabs
    Leaf {
        rect: Rect,
        viewport: Rect,
        tabs: Vec<Tab>,
        /// The opened tab.
        active: TabIndex,
    },
    /// Parent node in the vertical orientation
    Vertical { rect: Rect, fraction: f32 },
    /// Parent node in the horizontal orientation
    Horizontal { rect: Rect, fraction: f32 },
}

impl<Tab> Node<Tab> {
    /// Constructs a leaf node with a given `tab`.
    pub fn leaf(tab: Tab) -> Self {
        Self::Leaf {
            rect: Rect::NOTHING,
            viewport: Rect::NOTHING,
            tabs: vec![tab],
            active: TabIndex(0),
        }
    }

    /// Constructs a leaf node with a given list of `tabs`.
    pub const fn leaf_with(tabs: Vec<Tab>) -> Self {
        Self::Leaf {
            rect: Rect::NOTHING,
            viewport: Rect::NOTHING,
            tabs,
            active: TabIndex(0),
        }
    }

    /// Sets the area occupied by the node.
    #[inline(always)]
    pub fn set_rect(&mut self, new_rect: Rect) {
        match self {
            Self::Empty => (),
            Self::Leaf { rect, .. }
            | Self::Vertical { rect, .. }
            | Self::Horizontal { rect, .. } => *rect = new_rect,
        }
    }

    /// Returns `true` if the node is a `Empty`, `false` otherwise.
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns `true` if the node is a `Leaf`, `false` otherwise.
    pub const fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    /// Returns `true` if the node is a `Horizontal`, `false` otherwise.
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal { .. })
    }

    /// Returns `true` if the node is a `Vertical`, `false` otherwise.
    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical { .. })
    }

    /// Returns `true` if the node is either `Horizontal` or `Vertical`, `false` otherwise.
    pub const fn is_parent(&self) -> bool {
        self.is_horizontal() || self.is_vertical()
    }

    /// Replaces the node with a `Horizontal` or `Vertical` one (depending on `split`) and assigns it an empty rect.
    #[inline]
    pub fn split(&mut self, split: Split, fraction: f32) -> Self {
        let rect = Rect::NOTHING;
        let src = match split {
            Split::Left | Split::Right => Node::Horizontal { fraction, rect },
            Split::Above | Split::Below => Node::Vertical { fraction, rect },
        };
        std::mem::replace(self, src)
    }

    /// Adds a `tab` to the node.
    ///
    /// # Panics
    /// Panics if the new capacity of `tabs` exceeds isize::MAX bytes.
    #[track_caller]
    pub fn append_tab(&mut self, tab: Tab) {
        match self {
            Node::Leaf { tabs, active, .. } => {
                *active = TabIndex(tabs.len());
                tabs.push(tab);
            }
            _ => unreachable!(),
        }
    }

    /// Adds a `tab` to the node.
    ///
    /// # Panics
    /// Panics if the new capacity of `tabs` exceeds isize::MAX bytes.
    /// index > tabs_count()
    #[track_caller]
    pub fn insert_tab(&mut self, index: TabIndex, tab: Tab) {
        match self {
            Node::Leaf { tabs, active, .. } => {
                tabs.insert(index.0, tab);
                *active = index;
            }
            _ => unreachable!(),
        }
    }

    /// Removes a tab at given `index` from the node.
    /// Returns the removed tab if the node is a `Leaf`, or `None` otherwise.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn remove_tab(&mut self, tab_index: TabIndex) -> Option<Tab> {
        match self {
            Node::Leaf { tabs, .. } => Some(tabs.remove(tab_index.0)),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn tabs_count(&self) -> usize {
        match self {
            Node::Leaf { tabs, .. } => tabs.len(),
            _ => Default::default(),
        }
    }
}

// ----------------------------------------------------------------------------

/// Wrapper around indices to the collection of nodes inside a `Tree`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NodeIndex(pub usize);

impl From<usize> for NodeIndex {
    #[inline]
    fn from(index: usize) -> Self {
        NodeIndex(index)
    }
}

impl NodeIndex {
    /// Returns the index of the root node.
    pub const fn root() -> Self {
        Self(0)
    }

    /// Returns the index of the node to the left of the current one.
    pub const fn left(self) -> Self {
        Self(self.0 * 2 + 1)
    }

    /// Returns the index of the node to the right of the current one.
    pub const fn right(self) -> Self {
        Self(self.0 * 2 + 2)
    }

    /// Returns the index of the parent node or `None` if current node is the root.
    pub const fn parent(self) -> Option<Self> {
        if self.0 > 0 {
            Some(Self((self.0 - 1) / 2))
        } else {
            None
        }
    }

    /// Returns the number of nodes leading from the root to the current node, including `self`.
    pub const fn level(self) -> usize {
        (usize::BITS - (self.0 + 1).leading_zeros()) as usize
    }

    /// Returns true if current node is the left node of its parent, false otherwise.
    pub const fn is_left(self) -> bool {
        self.0 % 2 != 0
    }

    /// Returns true if current node is the right node of its parent, false otherwise.
    pub const fn is_right(self) -> bool {
        self.0 % 2 == 0
    }

    const fn children_at(self, level: usize) -> std::ops::Range<usize> {
        let base = 1 << level;
        let s = (self.0 + 1) * base - 1;
        let e = (self.0 + 2) * base - 1;
        s..e
    }

    const fn children_left(self, level: usize) -> std::ops::Range<usize> {
        let base = 1 << level;
        let s = (self.0 + 1) * base - 1;
        let e = (self.0 + 1) * base + base / 2 - 1;
        s..e
    }

    const fn children_right(self, level: usize) -> std::ops::Range<usize> {
        let base = 1 << level;
        let s = (self.0 + 1) * base + base / 2 - 1;
        let e = (self.0 + 2) * base - 1;
        s..e
    }
}

// ----------------------------------------------------------------------------

/// Direction in which a new node is created relatively to the parent node at which the split occurs.
#[derive(Clone, Copy)]
pub enum Split {
    Left,
    Right,
    Above,
    Below,
}

// ----------------------------------------------------------------------------

/// Binary tree representing the relationships between `Node`s.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Tree<Tab> {
    tree: Vec<Node<Tab>>,
    focused_node: Option<NodeIndex>,
}

impl<Tab> Default for Tree<Tab> {
    fn default() -> Self {
        Self {
            tree: Default::default(),
            focused_node: Default::default(),
        }
    }
}

impl<Tab> std::ops::Index<NodeIndex> for Tree<Tab> {
    type Output = Node<Tab>;

    #[inline(always)]
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.tree[index.0]
    }
}

impl<Tab> std::ops::IndexMut<NodeIndex> for Tree<Tab> {
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
    pub fn find_active(&mut self) -> Option<(Rect, &mut Tab)> {
        self.tree.iter_mut().find_map(|node| {
            if let Node::Leaf {
                tabs,
                active,
                viewport,
                ..
            } = node
            {
                tabs.get_mut(active.0).map(|tab| (*viewport, tab))
            } else {
                None
            }
        })
    }

    /// Returns the viewport `Rect` and the `Tab` inside the focused leaf node or `None` if it does not exist.
    pub fn find_active_focused(&mut self) -> Option<(Rect, &mut Tab)> {
        if let Some(Node::Leaf {
            tabs,
            active,
            viewport,
            ..
        }) = self.focused_node.and_then(|idx| self.tree.get_mut(idx.0))
        {
            tabs.get_mut(active.0).map(|tab| (*viewport, tab))
        } else {
            None
        }
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

    /// Returns `Iter` of the underlying collection of nodes.
    pub fn iter(&self) -> std::slice::Iter<'_, Node<Tab>> {
        self.tree.iter()
    }

    /// Returns `IterMut` of the underlying collection of nodes.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Node<Tab>> {
        self.tree.iter_mut()
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
        assert!(old.is_leaf());

        {
            let index = self.tree.iter().rposition(|n| !n.is_empty()).unwrap_or(0);
            let level = NodeIndex(index).level();
            self.tree.resize_with(1 << (level + 1), || Node::Empty);
        }

        let index = match split {
            Split::Right | Split::Above => [parent.right(), parent.left()],
            Split::Left | Split::Below => [parent.left(), parent.right()],
        };

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
            ) => match self.first_leaf(left) {
                ret @ Some(_) => ret,
                None => self.first_leaf(right),
            },
            (Some(Node::Horizontal { .. } | Node::Vertical { .. }), _) => self.first_leaf(left),
            (_, Some(Node::Horizontal { .. } | Node::Vertical { .. })) => self.first_leaf(right),

            (None, None)
            | (Some(&Node::Empty), None)
            | (None, Some(&Node::Empty))
            | (Some(&Node::Empty), Some(&Node::Empty)) => None,
        }
    }

    /// Removes the first node containing 0 tabs.
    pub fn remove_empty_leaf(&mut self) {
        let mut nodes = self.tree.iter().enumerate();
        let node = nodes.find_map(|(index, node)| match node {
            Node::Leaf { tabs, .. } if tabs.is_empty() => Some(index),
            _ => None,
        });

        let node = match node {
            Some(node) => NodeIndex(node),
            None => return,
        };

        let parent = match node.parent() {
            Some(val) => val,
            None => {
                self.tree.clear();
                return;
            }
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
                if let Some(Node::Leaf { .. }) = self.tree.get(next.0) {
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

    /// Currently focused leaf.
    pub fn focused_leaf(&self) -> Option<NodeIndex> {
        self.focused_node
    }

    /// Sets the currently focused leaf to `node_index` if the node at `node_index` is a leaf.
    pub fn set_focused_node(&mut self, node_index: NodeIndex) {
        if let Some(Node::Leaf { .. }) = self.tree.get(node_index.0) {
            self.focused_node = Some(node_index);
        } else {
            self.focused_node = None;
        }
    }

    /// Sets which is the active tab within a specific node.
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
            if let Node::Leaf { tabs, .. } = node {
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
