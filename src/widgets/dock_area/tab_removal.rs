use crate::{NodeIndex, SurfaceIndex, TabIndex};

/// An enum expressing an entry in the `to_remove` field in [`DockArea`].
#[derive(Debug, Clone, Copy)]
pub(super) enum TabRemoval {
    Tab(SurfaceIndex, NodeIndex, TabIndex, ForcedRemoval),
    Node(SurfaceIndex, NodeIndex),
    Window(SurfaceIndex),
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ForcedRemoval(pub bool);
