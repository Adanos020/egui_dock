/// Wrapper around indices to the collection of Windows inside a [`Tree`](crate::Tree).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WindowIndex(pub usize);

impl From<usize> for WindowIndex {
    #[inline(always)]
    fn from(index: usize) -> Self {
        WindowIndex(index)
    }
}
