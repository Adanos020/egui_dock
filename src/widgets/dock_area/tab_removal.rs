use crate::{NodeIndex, SurfaceIndex, TabIndex};

/// An enum expressing an entry in the `to_remove` field in [`DockArea`].
#[derive(Debug, Clone, Copy)]
pub(super) enum TabRemoval {
    Node(SurfaceIndex, NodeIndex, TabIndex),
    Leaf(SurfaceIndex, NodeIndex),
    Window(SurfaceIndex),
}
