use enum_iterator::Sequence;

use crate::utils::collection::Bitsetable;
use crate::utils::macros::rkyv_self;

pub const LEVEL_IDLE: u16 = 0;
pub const LEVEL_MOVE: u16 = 50;
pub const LEVEL_ATTACK: u16 = 100;
pub const LEVEL_SKILL: u16 = 200;
pub const LEVEL_DERIVE: u16 = 300;
pub const LEVEL_ULTIMATE: u16 = 400;
pub const LEVEL_ACTION: u16 = 500;
pub const LEVEL_UNBREAKABLE: u16 = 600;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RareLevel {
    Rare1 = 1,
    Rare2 = 2,
    Rare3 = 3,
}

rkyv_self!(RareLevel);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Sequence)]
pub enum DeriveContinue {
    Dodge,
    PerfectDodge,
    Guard,
    PerfectGuard,
}

rkyv_self!(DeriveContinue);

unsafe impl Bitsetable for DeriveContinue {
    #[inline]
    fn ordinal(&self) -> usize {
        *self as usize
    }
}
