use egui::Rect;

use crate::TabIndex;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LeafNode<Tab> {
    /// The full rectangle - tab bar plus tab body.
    pub rect: Rect,

    /// The tab body rectangle.
    pub viewport: Rect,

    /// All the tabs in this node.
    pub tabs: Vec<Tab>,

    /// The opened tab.
    pub active: TabIndex,

    /// Scroll amount of the tab bar.
    pub scroll: f32,

    /// Whether the leaf is collapsed.
    pub collapsed: bool,
}
impl<Tab> LeafNode<Tab> {
    pub const fn new(tabs: Vec<Tab>) -> Self {
        LeafNode {
            rect: Rect::NOTHING,
            viewport: Rect::NOTHING,
            tabs,
            active: TabIndex(0),
            scroll: 0.0,
            collapsed: false,
        }
    }

    #[inline]
    pub fn set_rect(&mut self, new_rect: Rect) {
        self.rect = new_rect;
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    #[inline]
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    #[inline]
    pub fn tabs_mut(&mut self) -> &mut [Tab] {
        &mut self.tabs
    }

    #[track_caller]
    #[inline]
    pub fn append_tab(&mut self, tab: Tab) {
        self.active = TabIndex(self.tabs.len());
        self.tabs.push(tab);
    }

    #[track_caller]
    #[inline]
    pub fn insert_tab(&mut self, index: TabIndex, tab: Tab) {
        self.tabs.insert(index.0, tab);
        self.active = index;
    }

    #[inline]
    pub fn remove_tab(&mut self, tab_index: TabIndex) -> Option<Tab> {
        if tab_index <= self.active {
            self.active.0 = self.active.0.saturating_sub(1);
        }
        Some(self.tabs.remove(tab_index.0))
    }

    pub fn retain_tabs<F>(&mut self, predicate: F)
    where
        F: FnMut(&mut Tab) -> bool,
    {
        self.tabs.retain_mut(predicate);
    }

    #[inline]
    pub fn active_focused(&mut self) -> Option<(Rect, &mut Tab)> {
        self.tabs.get_mut(self.active.0).map(|tab| (self.viewport, tab))
    }

}