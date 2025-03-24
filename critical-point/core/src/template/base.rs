use cirtical_point_csgen::CsEnum;
use enum_iterator::{cardinality, Sequence};
use std::fmt::Debug;
use std::ops::Deref;
use std::ptr::NonNull;
use std::{alloc, fmt, mem, slice};

use crate::sb;
use crate::utils::{xres, Castable, StrID, Symbol, XError, XResult};

#[repr(u16)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Sequence,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsEnum,
)]
#[archive_attr(derive(Debug))]
pub enum TmplType {
    Character,
    Style,
    Equipment,
    Entry,
    Perk,
    AccessoryPattern,
    Accessory,
    Jewel,

    ActionGeneral,
    ActionIdle,
    ActionMove,
    ActionDodge,
    ActionGuard,
    ActionAim,

    Stage,
}

impl From<TmplType> for u16 {
    #[inline]
    fn from(val: TmplType) -> Self {
        unsafe { mem::transmute::<TmplType, u16>(val) }
    }
}

impl TryFrom<u16> for TmplType {
    type Error = XError;

    #[inline]
    fn try_from(value: u16) -> XResult<Self> {
        if value as usize >= cardinality::<TmplType>() {
            return xres!(Overflow);
        }
        Ok(unsafe { mem::transmute::<u16, TmplType>(value) })
    }
}

#[typetag::deserialize(tag = "T")]
pub trait TmplAny: Debug {
    fn id(&self) -> StrID;
    fn typ(&self) -> TmplType;
}

impl Castable for dyn TmplAny {}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub enum TmplRare {
    Rare1 = 1,
    Rare2 = 2,
    Rare3 = 3,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[serde(untagged)]
pub enum TmplSwitch {
    Bool(bool),
    Symbol(Symbol),
}

impl Default for TmplSwitch {
    fn default() -> Self {
        TmplSwitch::Bool(false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplLevelRange {
    pub min: u32,
    pub max: u32,
}

impl TmplLevelRange {
    pub fn new(min: u32, max: u32) -> TmplLevelRange {
        TmplLevelRange { min, max }
    }
}

impl From<[u32; 2]> for TmplLevelRange {
    fn from(range: [u32; 2]) -> TmplLevelRange {
        TmplLevelRange::new(range[0], range[1])
    }
}

impl From<TmplLevelRange> for [u32; 2] {
    fn from(val: TmplLevelRange) -> Self {
        [val.min, val.max]
    }
}

impl From<(u32, u32)> for TmplLevelRange {
    fn from(range: (u32, u32)) -> TmplLevelRange {
        TmplLevelRange::new(range.0, range.1)
    }
}

impl From<TmplLevelRange> for (u32, u32) {
    fn from(val: TmplLevelRange) -> Self {
        (val.min, val.max)
    }
}

impl<'de> serde::Deserialize<'de> for TmplLevelRange {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<TmplLevelRange, D::Error> {
        let range: [u32; 2] = serde::Deserialize::deserialize(deserializer)?;
        Ok(TmplLevelRange::new(range[0], range[1]))
    }
}
