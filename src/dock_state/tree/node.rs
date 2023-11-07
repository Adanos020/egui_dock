use crate::{Split, TabIndex};
use egui::Rect;

/// Represents an abstract node of a [`Tree`](crate::Tree).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Node<Tab> {
    /// Empty node.
    Empty,

    /// Contains the actual tabs.
    Leaf {
        /// The full rectangle - tab bar plus tab body.
        rect: Rect,

        /// The tab body rectangle.
        viewport: Rect,

        /// All the tabs in this node.
        tabs: Vec<Tab>,

        /// The opened tab.
        active: TabIndex,

        /// Scroll amount of the tab bar.
        scroll: f32,
    },

    /// Parent node in the vertical orientation.
    Vertical {
        /// The rectangle in which all children of this node are drawn.
        rect: Rect,

        /// The fraction taken by the top child of this node.
        fraction: f32,
    },

    /// Parent node in the horizontal orientation.
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

    /// Get a [`Rect`] occupied by the node, could be used e.g. to draw a highlight rect around a node.
    ///
    /// Returns [`None`] if node is of the [`Empty`](Node::Empty) variant.
    #[inline]
    pub fn rect(&self) -> Option<Rect> {
        match self {
            Node::Empty => None,
            Node::Leaf { rect, .. }
            | Node::Vertical { rect, .. }
            | Node::Horizontal { rect, .. } => Some(*rect),
        }
    }

    /// Returns `true` if the node is a [`Empty`](Node::Empty), otherwise `false`.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns `true` if the node is a [`Leaf`](Node::Leaf), otherwise `false`.
    #[inline(always)]
    pub const fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    /// Returns `true` if the node is a [`Horizontal`](Node::Horizontal), otherwise `false`.
    #[inline(always)]
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal { .. })
    }

    /// Returns `true` if the node is a [`Vertical`](Node::Vertical), otherwise `false`.
    #[inline(always)]
    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical { .. })
    }

    /// Returns `true` if the node is either [`Horizontal`](Node::Horizontal) or [`Vertical`](Node::Vertical),
    /// otherwise `false`.
    #[inline(always)]
    pub const fn is_parent(&self) -> bool {
        self.is_horizontal() || self.is_vertical()
    }

    /// Replaces the node with [`Horizontal`](Node::Horizontal) or [`Vertical`](Node::Vertical) (depending on `split`)
    /// and assigns an empty rect to it.
    ///
    /// # Panics
    ///
    /// If `fraction` isn't in range 0..=1.
    #[inline]
    pub fn split(&mut self, split: Split, fraction: f32) -> Self {
        assert!((0.0..=1.0).contains(&fraction));
        let rect = Rect::NOTHING;
        let src = match split {
            Split::Left | Split::Right => Node::Horizontal { fraction, rect },
            Split::Above | Split::Below => Node::Vertical { fraction, rect },
        };
        std::mem::replace(self, src)
    }

    /// Provides an immutable slice of the tabs inside this node.
    ///
    /// Returns [`None`] if the node is not a [`Leaf`](Node::Leaf).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_dock::{DockState, NodeIndex};
    /// let mut dock_state = DockState::new(vec![1, 2, 3, 4, 5, 6]);
    /// assert!(dock_state.main_surface().root_node().unwrap().tabs().unwrap().contains(&4));
    /// ```
    #[inline]
    pub fn tabs(&self) -> Option<&[Tab]> {
        match self {
            Node::Leaf { tabs, .. } => Some(tabs),
            _ => None,
        }
    }

    /// Provides an mutable slice of the tabs inside this node.
    ///
    /// Returns [`None`] if the node is not a [`Leaf`](Node::Leaf).
    ///
    /// # Examples
    ///
    /// Modifying tabs inside a node:
    /// ```rust
    /// # use egui_dock::{DockState, NodeIndex};
    /// let mut dock_state = DockState::new(vec![1, 2, 3, 4, 5, 6]);
    /// let mut tabs = dock_state
    ///     .main_surface_mut()
    ///     .root_node_mut()
    ///     .unwrap()
    ///     .tabs_mut()
    ///     .unwrap();
    ///
    /// tabs[0] = 7;
    /// tabs[5] = 8;
    ///
    /// assert_eq!(&tabs, &[7, 2, 3, 4, 5, 8]);
    /// ```
    #[inline]
    pub fn tabs_mut(&mut self) -> Option<&mut [Tab]> {
        match self {
            Node::Leaf { tabs, .. } => Some(tabs),
            _ => None,
        }
    }

    /// Returns an [`Iterator`] of tabs in this node.
    ///
    /// If this node is not a [`Leaf`](Self::Leaf), then the returned [`Iterator`] will be empty.
    #[inline]
    pub fn iter_tabs(&self) -> impl Iterator<Item = &Tab> {
        match self.tabs() {
            Some(tabs) => tabs.iter(),
            None => core::slice::Iter::default(),
        }
    }

    /// Returns a mutable [`Iterator`] of tabs in this node.
    ///
    /// If this node is not a [`Leaf`](Self::Leaf), then the returned [`Iterator`] will be empty.
    #[inline]
    pub fn iter_tabs_mut(&mut self) -> impl Iterator<Item = &mut Tab> {
        match self.tabs_mut() {
            Some(tabs) => tabs.iter_mut(),
            None => core::slice::IterMut::default(),
        }
    }

    /// Adds `tab` to the node and sets it as the active tab.
    ///
    /// # Panics
    ///
    /// If the new capacity of `tabs` exceeds `isize::MAX` bytes.
    ///
    /// If `self` is not a [`Leaf`](Node::Leaf) node.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use egui_dock::{DockState, NodeIndex};
    /// let mut dock_state = DockState::new(vec!["a tab"]);
    /// assert_eq!(dock_state.main_surface().root_node().unwrap().tabs_count(), 1);
    ///
    /// dock_state.main_surface_mut().root_node_mut().unwrap().append_tab("another tab");
    /// assert_eq!(dock_state.main_surface().root_node().unwrap().tabs_count(), 2);
    /// ```
    #[track_caller]
    #[inline]
    pub fn append_tab(&mut self, tab: Tab) {
        match self {
            Node::Leaf { tabs, active, .. } => {
                *active = TabIndex(tabs.len());
                tabs.push(tab);
            }
            _ => panic!("node was not a leaf"),
        }
    }

    /// Adds a `tab` to the node.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity of `tabs` exceeds `isize::MAX` bytes, or `index > tabs_count()`.
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
    ///
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

    /// Returns a new Node while mapping the tab type
    pub fn map_tabs<F, NewTab>(&self, function: F) -> Node<NewTab>
    where
        F: FnMut(&Tab) -> NewTab,
    {
        match self {
            Node::Leaf {
                rect,
                viewport,
                tabs,
                active,
                scroll,
            } => Node::Leaf {
                rect: *rect,
                viewport: *viewport,
                tabs: tabs.iter().map(function).collect(),
                active: *active,
                scroll: *scroll,
            },
            Node::Empty => Node::Empty,
            Node::Vertical { rect, fraction } => Node::Vertical {
                rect: *rect,
                fraction: *fraction,
            },
            Node::Horizontal { rect, fraction } => Node::Horizontal {
                rect: *rect,
                fraction: *fraction,
            },
        }
    }
}
