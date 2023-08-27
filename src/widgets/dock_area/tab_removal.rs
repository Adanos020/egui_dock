use crate::{NodeIndex, SurfaceIndex, TabIndex};

/// An enum expressing an entry in the `to_remove` field in [`DockArea`].
#[derive(Debug, Clone, Copy)]
pub(super) enum TabRemoval {
    Node(SurfaceIndex, NodeIndex, TabIndex),
    Window(SurfaceIndex),
}

impl From<SurfaceIndex> for TabRemoval {
    fn from(index: SurfaceIndex) -> Self {
        TabRemoval::Window(index)
    }
}

impl From<(SurfaceIndex, NodeIndex, TabIndex)> for TabRemoval {
    fn from((si, ni, ti): (SurfaceIndex, NodeIndex, TabIndex)) -> TabRemoval {
        TabRemoval::Node(si, ni, ti)
    }
}
