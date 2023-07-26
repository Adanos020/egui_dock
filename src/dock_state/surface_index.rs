/// Wrapper around indices to the collection of Surfaces inside a [`DockState`](crate::DockState).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SurfaceIndex(pub usize);

impl From<usize> for SurfaceIndex {
    #[inline(always)]
    fn from(index: usize) -> Self {
        SurfaceIndex(index)
    }
}
impl SurfaceIndex {
    /// Returns the index of the root surface.
    #[inline(always)]
    pub const fn root() -> Self {
        Self(0)
    }
}
