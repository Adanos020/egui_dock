/// What directions can this dock be split in?
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum AllowedSplits {
    #[default]
    /// Allow splits in any direction (horizontal and vertical).
    All = 0b11,

    /// Only allow split in a horizontal directions.
    LeftRightOnly = 0b10,

    /// Only allow splits in a vertical directions.
    TopBottomOnly = 0b01,

    /// Don't allow splits at all.
    None = 0b00,
}

impl std::ops::BitAnd for AllowedSplits {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_u8(self as u8 & rhs as u8)
    }
}

impl AllowedSplits {
    /// Create allowed splits from a u8, panics if an invalid value is given.
    #[inline(always)]
    fn from_u8(u8: u8) -> Self {
        match u8 {
            0b11 => AllowedSplits::All,
            0b10 => AllowedSplits::LeftRightOnly,
            0b01 => AllowedSplits::TopBottomOnly,
            0b00 => AllowedSplits::None,
            _ => panic!("Provided an invalid value for allowed splits: {u8:0x}"),
        }
    }
}
