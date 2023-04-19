#[allow(unused_imports)]
use crate::Tree; // For cleaner doc comments

use crate::{Split, TabIndex};
use egui::Rect;

/// Represents an abstract node of a [`Tree`].
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Node<Tab> {
    /// Empty node
    Empty,
    /// Contains the actual tabs
    Leaf {
        /// The full rectangle - tab bar plus tab body
        rect: Rect,

        /// The tab body rectangle
        viewport: Rect,

        /// All the tabs in this node.
        tabs: Vec<Tab>,

        /// The opened tab.
        active: TabIndex,

        /// Scroll amount of the tab bar.
        scroll: f32,
    },
    /// Parent node in the vertical orientation
    Vertical {
        /// The rectangle in which all children of this node are drawn.
        rect: Rect,

        /// The fraction taken by the top child of this node.
        fraction: f32,
    },
    /// Parent node in the horizontal orientation
    Horizontal {
        /// The rectangle in which all children of this node are drawn.
        rect: Rect,

        /// The fraction taken by the left child of this node.
        fraction: f32,
    },
}

impl<Tab> Node<Tab> {
    /// Constructs a leaf node with a given `tab`.
    #[inline(always)]
    pub fn leaf(tab: Tab) -> Self {
        Self::Leaf {
            rect: Rect::NOTHING,
            viewport: Rect::NOTHING,
            tabs: vec![tab],
            active: TabIndex(0),
            scroll: 0.0,
        }
    }

    /// Constructs a leaf node with a given list of `tabs`.
    #[inline(always)]
    pub const fn leaf_with(tabs: Vec<Tab>) -> Self {
        Self::Leaf {
            rect: Rect::NOTHING,
            viewport: Rect::NOTHING,
            tabs,
            active: TabIndex(0),
            scroll: 0.0,
        }
    }

    /// Sets the area occupied by the node.
    #[inline]
    pub fn set_rect(&mut self, new_rect: Rect) {
        match self {
            Self::Empty => (),
            Self::Leaf { rect, .. }
            | Self::Vertical { rect, .. }
            | Self::Horizontal { rect, .. } => *rect = new_rect,
        }
    }

    /// Returns `true` if the node is a `Empty`, `false` otherwise.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns `true` if the node is a `Leaf`, `false` otherwise.
    #[inline(always)]
    pub const fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    /// Returns `true` if the node is a `Horizontal`, `false` otherwise.
    #[inline(always)]
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal { .. })
    }

    /// Returns `true` if the node is a `Vertical`, `false` otherwise.
    #[inline(always)]
    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical { .. })
    }

    /// Returns `true` if the node is either `Horizontal` or `Vertical`, `false` otherwise.
    #[inline(always)]
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
    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn remove_tab(&mut self, tab_index: TabIndex) -> Option<Tab> {
        match self {
            Node::Leaf { tabs, active, .. } => {
                if tab_index <= *active {
                    active.0 = active.0.saturating_sub(1);
                }

                Some(tabs.remove(tab_index.0))
            }
            _ => None,
        }
    }

    /// Gets the number of tabs in the node.
    #[inline]
    pub fn tabs_count(&self) -> usize {
        match self {
            Node::Leaf { tabs, .. } => tabs.len(),
            _ => Default::default(),
        }
    }
}
