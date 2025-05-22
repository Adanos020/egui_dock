use egui::Rect;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SplitNode {
    /// The rectangle in which all children of this node are drawn.
    pub rect: Rect,

    /// The fraction taken by the top child of this node.
    pub fraction: f32,

    /// Whether all subnodes are collapsed.
    pub fully_collapsed: bool,

    /// The number of collapsed leaf subnodes.
    pub collapsed_leaf_count: i32,
}
impl SplitNode {
    pub const fn new(rect: Rect, fraction: f32, fully_collapsed: bool, collapsed_leaf_count: i32) -> Self {
        Self { rect, fraction, fully_collapsed, collapsed_leaf_count }
    }
    #[inline]
    pub fn set_rect(&mut self, new_rect: Rect) {
        self.rect = new_rect;
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }
}