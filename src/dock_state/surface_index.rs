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
    /// Returns the index of the main surface.
    #[inline(always)]
    pub const fn main() -> Self {
        Self(0)
    }

    /// Compares this surface index with `SurfaceIndex::root()`.
    #[inline(always)]
    pub const fn is_main(self) -> bool {
        self.0 == Self::main().0
    }
}
