
use egui::*;

pub type Tab = Box<dyn crate::Tab>;
pub type Tabs = Vec<Tab>;

/// Represents an abstract node of a `Tree`.
pub enum Node {
    /// Empty node
    None,
    /// Contains the actual tabs
    Leaf {
        rect: Rect,
        viewport: Rect,
        tabs: Tabs,
        active: usize,
    },
    /// Parent node in the vertical orientation
    Vertical { rect: Rect, fraction: f32 },
    /// Parent node in the horizontal orientation
    Horizontal { rect: Rect, fraction: f32 },
}

impl Node {
    /// Constructs a leaf node with a given `tab`.
    pub fn leaf(tab: Tab) -> Self {
        Self::Leaf {
            rect: Rect::NOTHING,
            viewport: Rect::NOTHING,
            tabs: vec![tab],
            active: Default::default(),
        }
    }

    /// Constructs a leaf node with a given list of `tabs`.
    pub const fn leaf_with(tabs: Vec<Tab>) -> Self {
        Self::Leaf {
            rect: Rect::NOTHING,
            viewport: Rect::NOTHING,
            tabs,
            active: 0,
        }
    }

    /// Sets the area occupied by the node.
    #[inline(always)]
    pub fn set_rect(&mut self, new_rect: Rect) {
        match self {
            Self::None => (),
            Self::Leaf { rect, .. }
            | Self::Vertical { rect, .. }
            | Self::Horizontal { rect, .. } => *rect = new_rect,
        }
    }

    /// Returns `true` if the node is a `None`, `false` otherwise.
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Returns `true` if the node is a `Leaf`, `false` otherwise.
    pub const fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
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
                tabs.push(tab);
                *active = tabs.len() - 1;
            },
            _ => unreachable!(),
        }
        
    }

    /// Adds a `tab` to the node.
    ///
    /// # Panics
    /// Panics if the new capacity of `tabs` exceeds isize::MAX bytes.
    /// index > tabs_count()
    #[track_caller]
    pub fn insert_tab(&mut self, index: usize, tab: Tab) {
        match self {
            Node::Leaf { tabs, active, .. } => {
                tabs.insert(index, tab);
                *active = index;

            },
            _ => unreachable!(),
        }
    }

    /// Removes a tab at given `index` from the node.
    /// Returns the removed tab if the node is a `Leaf`, or `None` otherwise.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn remove_tab(&mut self, index: usize) -> Option<Tab> {
        match self {
            Node::Leaf { tabs, .. } => Some(tabs.remove(index)),
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

/// Wrapper around indices to the collection of nodes inside a `Tree`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeIndex(pub usize);

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

/// Direction in which a new node is created relatively to the parent node at which the split occurs.
#[derive(Clone, Copy)]
pub enum Split {
    Left,
    Right,
    Above,
    Below,
}

/// Binary tree representing the relationships between `Node`s.
#[derive(Default)]
pub struct Tree {
    tree: Vec<Node>,
    focused_node: Option<NodeIndex>,
}

impl std::ops::Index<NodeIndex> for Tree {
    type Output = Node;

    #[inline(always)]
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.tree[index.0]
    }
}

impl std::ops::IndexMut<NodeIndex> for Tree {
    #[inline(always)]
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.tree[index.0]
    }
}

impl Tree {
    /// Creates a new `Tree` with given `Vec` of `Tab`s in its root node.
    #[inline(always)]
    pub fn new(tabs: Tabs) -> Self {
        let root = Node::leaf_with(tabs);
        Self { tree: vec![root], focused_node: None }
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
                tabs.get_mut(*active).map(|tab| (*viewport, tab))
            } else {
                None
            }
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

    /// Returns `Iter` of the underlying collection of nodes.
    pub fn iter(&self) -> std::slice::Iter<'_, Node> {
        self.tree.iter()
    }

    /// Returns `IterMut` of the underlying collection of nodes.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Node> {
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
        tabs: Tabs,
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
    pub fn split_above(&mut self, parent: NodeIndex, fraction: f32, tabs: Tabs) -> [NodeIndex; 2] {
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
    pub fn split_below(&mut self, parent: NodeIndex, fraction: f32, tabs: Tabs) -> [NodeIndex; 2] {
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
    pub fn split_left(&mut self, parent: NodeIndex, fraction: f32, tabs: Tabs) -> [NodeIndex; 2] {
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
    pub fn split_right(&mut self, parent: NodeIndex, fraction: f32, tabs: Tabs) -> [NodeIndex; 2] {
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
        new: Node,
    ) -> [NodeIndex; 2] {
        let old = self[parent].split(split, fraction);
        assert!(old.is_leaf());

        {
            let index = self.tree.iter().rposition(|n| !n.is_none()).unwrap_or(0);
            let level = NodeIndex(index).level();
            self.tree.resize_with(1 << (level + 1), || Node::None);
        }

        let index = match split {
            Split::Right | Split::Above => [parent.right(), parent.left()],
            Split::Left | Split::Below => [parent.left(), parent.right()],
        };

        self[index[0]] = old;
        self[index[1]] = new;

        self.focused_node = Option::Some(index[1]);

        index
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

        let parent = match node.parent(){
            Some(val) => val,
            None => {
                self.tree.clear();
                return;
            },
        };


        fn first_leaf(tree: &Tree, top: NodeIndex) -> Option<NodeIndex>{
            let left = top.left();
            let right = top.right();
            match (tree.tree.get(left.0), tree.tree.get(right.0)){
                (Some(&Node::Leaf{..}), _) => Option::Some(left),
                (_, Some(&Node::Leaf{..})) => Option::Some(left),
                
                (Some(Node::Horizontal{..} | Node::Vertical{..}), Some(Node::Horizontal{..} | Node::Vertical{..})) => {
                    match first_leaf(tree, left){
                        ret @ Some(_) => ret,
                        None => first_leaf(tree, right),
                    }
                },
                (Some(Node::Horizontal{..} | Node::Vertical{..}), _) => first_leaf(tree, left),
                (_, Some(Node::Horizontal{..} | Node::Vertical{..})) => first_leaf(tree, right),
                
                (None, None) 
                | (Some(&Node::None), None)
                | (None, Some(&Node::None))
                | (Some(&Node::None), Some(&Node::None)) => None,
            }
        }

        if Option::Some(node) == self.focused_node{
            self.focused_node = Option::None;
            let mut node = node;
            while let Option::Some(parent) = node.parent(){
                let next = if node.is_left(){
                    parent.right()
                }else{
                    parent.left()
                };
                if let Option::Some(Node::Leaf{..}) = self.tree.get(next.0){
                    self.focused_node = Option::Some(next);
                    break;
                }
                if let Option::Some(node) = first_leaf(&self, next){
                    self.focused_node = Option::Some(node);
                    break;
                }
                node = parent;
            }
        }

        self[parent] = Node::None;
        self[node] = Node::None;


        let mut level = 0;

        if node.is_left() {
            'left_end: loop {
                let dst = parent.children_at(level);
                let src = parent.children_right(level + 1);
                for (dst, src) in dst.zip(src) {
                    if src >= self.tree.len() {
                        break 'left_end;
                    }
                    if Option::Some(NodeIndex(src)) == self.focused_node{
                        self.focused_node = Option::Some(NodeIndex(dst));
                    }
                    self.tree[dst] = std::mem::replace(&mut self.tree[src], Node::None);
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
                    if Option::Some(NodeIndex(src)) == self.focused_node{
                        self.focused_node = Option::Some(NodeIndex(dst));
                    }
                    self.tree[dst] = std::mem::replace(&mut self.tree[src], Node::None);
                }
                level += 1;
            }
        }
    }

    pub fn push_to_first_leaf(&mut self, tab: Tab){
        for (index, node) in &mut self.tree.iter_mut().enumerate(){
            match node{
                Node::Leaf { tabs, active, ..} => {
                    tabs.push(tab);
                    self.focused_node = Option::Some(NodeIndex(index));
                    *active = tabs.len() - 1;
                    return;
                }, 
                Node::None => {
                    *node = Node::leaf(tab);
                    self.focused_node = Option::Some(NodeIndex(index));
                    return;
                },
                _ => {}
            }
        }
        panic!();
    }

    pub fn focused_leaf(&self) -> Option<NodeIndex>{
        self.focused_node
    }
    pub fn set_focused(&mut self, node: NodeIndex){
        if let Option::Some(Node::Leaf{..}) = self.tree.get(node.0){
            self.focused_node = Option::Some(node);
        }else{
           self.focused_node = None; 
        }
    }

    pub fn push_to_active_leaf(&mut self, tab: Tab){
        match self.focused_node{
            Some(node) => {
                if self.tree.is_empty(){
                    self.tree.push(Node::leaf(tab));
                    self.focused_node = Option::Some(NodeIndex::root());
                }else{
                    match &mut self[node]{
                        Node::None => {
                            self[node] = Node::leaf(tab);
                            self.focused_node = Option::Some(node);
                        },
                        Node::Leaf { tabs, active, ..} => {
                            tabs.push(tab);
                            *active = tabs.len() - 1;
                            self.focused_node = Option::Some(node);
                        },
                        _ => {
                            self.push_to_first_leaf(tab);
                        }
                    }
                }
            },
            None => {
                if self.tree.is_empty(){
                    self.tree.push(Node::leaf(tab));
                    self.focused_node = Option::Some(NodeIndex::root());
                }else{
                    self.push_to_first_leaf(tab);
                }
            },
        }
    }
}
