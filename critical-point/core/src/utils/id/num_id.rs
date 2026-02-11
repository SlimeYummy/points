use std::fmt;
use std::ops::{Add, Sub};

use crate::consts::MAX_PLAYER;

#[repr(transparent)]
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug, Clone, Copy, PartialEq, Eq, Hash))]
pub struct NumID(pub u32);

impl NumID {
    pub const INVALID: NumID = NumID(u32::MAX);
    pub const GAME: NumID = NumID(1);
    pub const STAGE: NumID = NumID(2);
    pub const MIN_PLAYER: NumID = NumID(100);
    pub const MAX_PLAYER: NumID = NumID(100 + (MAX_PLAYER as u32));
    pub const MIN_AUTO_GEN: NumID = NumID(1000);

    #[inline]
    pub const fn new(id: u32) -> NumID {
        NumID(id)
    }

    pub fn from_rkyv(value: ArchivedNumID) -> NumID {
        NumID(value.0.to_native())
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        *self != Self::INVALID
    }

    #[inline]
    pub fn is_invalid(&self) -> bool {
        *self == Self::INVALID
    }

    #[inline]
    pub fn is_player(&self) -> bool {
        *self >= Self::MIN_PLAYER && *self <= Self::MAX_PLAYER
    }
}

impl Default for NumID {
    fn default() -> Self {
        Self::INVALID
    }
}

impl From<u32> for NumID {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<NumID> for u32 {
    fn from(value: NumID) -> Self {
        value.0
    }
}

impl PartialEq<u32> for NumID {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl PartialEq<NumID> for u32 {
    fn eq(&self, other: &NumID) -> bool {
        *self == other.0
    }
}

impl Add<u32> for NumID {
    type Output = NumID;
    fn add(self, rhs: u32) -> Self::Output {
        NumID(self.0 + rhs)
    }
}

impl Sub<u32> for NumID {
    type Output = NumID;
    fn sub(self, rhs: u32) -> Self::Output {
        NumID(self.0 - rhs)
    }
}

impl fmt::Debug for NumID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for NumID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
