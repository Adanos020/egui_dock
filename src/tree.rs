use crate::Tab;
use egui::*;

pub type Tabs<Context> = Vec<Box<dyn Tab<Context>>>;

pub enum Node<Context> {
    None,
    Leaf {
        rect: Rect,
        viewport: Rect,
        tabs: Tabs<Context>,
        active: usize,
    },
    Vertical {
        rect: Rect,
        fraction: f32,
    },
    Horizontal {
        rect: Rect,
        fraction: f32,
    },
}

impl<Context> Node<Context> {
    pub fn leaf(tab: Box<dyn Tab<Context>>) -> Self {
        Self::Leaf {
            rect: Rect::NOTHING,
            viewport: Rect::NOTHING,
            tabs: vec![tab],
            active: 0,
        }
    }

    pub fn leaf_with(tabs: Vec<Box<dyn Tab<Context>>>) -> Self {
        Self::Leaf {
            rect: Rect::NOTHING,
            viewport: Rect::NOTHING,
            tabs,
            active: 0,
        }
    }

    pub fn set_rect(&mut self, new_rect: Rect) {
        match self {
            Self::None => (),
            Self::Leaf { rect, .. }
            | Self::Vertical { rect, .. }
            | Self::Horizontal { rect, .. } => *rect = new_rect,
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    pub fn split(&mut self, split: Split, fraction: f32) -> Self {
        let rect = Rect::NOTHING;
        let src = match split {
            Split::Left | Split::Right => Node::Horizontal { fraction, rect },
            Split::Above | Split::Below => Node::Vertical { fraction, rect },
        };
        std::mem::replace(self, src)
    }

    #[track_caller]
    pub fn append_tab(&mut self, tab: Box<dyn Tab<Context>>) {
        match self {
            Node::Leaf { tabs, .. } => tabs.push(tab),
            _ => unreachable!(),
        }
    }

    pub fn remove_tab(&mut self, index: usize) -> Option<Box<dyn Tab<Context>>> {
        match self {
            Node::Leaf { tabs, .. } => Some(tabs.remove(index)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeIndex(pub usize);

impl NodeIndex {
    pub const fn root() -> Self {
        Self(0)
    }

    pub const fn left(self) -> Self {
        Self(self.0 * 2 + 1)
    }

    pub const fn right(self) -> Self {
        Self(self.0 * 2 + 2)
    }

    pub const fn parent(self) -> Option<Self> {
        if self.0 > 0 {
            Some(Self((self.0 - 1) / 2))
        } else {
            None
        }
    }

    pub const fn level(self) -> usize {
        (usize::BITS - (self.0 + 1).leading_zeros()) as usize
    }

    pub const fn is_left(self) -> bool {
        self.0 % 2 != 0
    }

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

#[derive(Clone, Copy)]
pub enum Split {
    Left,
    Right,
    Above,
    Below,
}

pub struct Tree<Context> {
    tree: Vec<Node<Context>>,
}

impl<Context> std::ops::Index<NodeIndex> for Tree<Context> {
    type Output = Node<Context>;

    #[inline(always)]
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.tree[index.0]
    }
}

impl<Context> std::ops::IndexMut<NodeIndex> for Tree<Context> {
    #[inline(always)]
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.tree[index.0]
    }
}

impl<Context> Default for Tree<Context> {
    fn default() -> Self {
        Self { tree: vec![] }
    }
}

impl<Context> Tree<Context> {
    pub fn new(tabs: Tabs<Context>) -> Self {
        let root = Node::leaf_with(tabs);
        Self { tree: vec![root] }
    }

    pub fn find_active<T: Tab<Context> + 'static>(&mut self) -> Option<(Rect, &mut T)> {
        self.tree.iter_mut().find_map(|node| {
            if let Node::Leaf {
                tabs,
                active,
                viewport,
                ..
            } = node
            {
                tabs.get_mut(*active)
                    .and_then(|tab| tab.downcast_mut::<T>())
                    .map(|tab| (*viewport, tab))
            } else {
                None
            }
        })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Node<Context>> {
        self.tree.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Node<Context>> {
        self.tree.iter_mut()
    }

    pub fn split_tabs(
        &mut self,
        parent: NodeIndex,
        split: Split,
        fraction: f32,
        tabs: Vec<Box<dyn Tab<Context>>>,
    ) -> [NodeIndex; 2] {
        self.split(parent, split, fraction, Node::leaf_with(tabs))
    }

    pub fn split_above(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Tabs<Context>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Above, fraction, Node::leaf_with(tabs))
    }

    pub fn split_below(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Tabs<Context>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Below, fraction, Node::leaf_with(tabs))
    }

    pub fn split_left(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Tabs<Context>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Left, fraction, Node::leaf_with(tabs))
    }

    pub fn split_right(
        &mut self,
        parent: NodeIndex,
        fraction: f32,
        tabs: Tabs<Context>,
    ) -> [NodeIndex; 2] {
        self.split(parent, Split::Right, fraction, Node::leaf_with(tabs))
    }

    pub fn split(
        &mut self,
        parent: NodeIndex,
        split: Split,
        fraction: f32,
        new: Node<Context>,
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

        index
    }

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

        let parent = node.parent().unwrap();

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
                    self.tree[dst] = std::mem::replace(&mut self.tree[src], Node::None);
                }
                level += 1;
            }
        }
    }
}
