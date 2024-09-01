use enum_iterator::{cardinality, Sequence};
use std::fmt::Debug;
use std::mem;

use crate::utils::{Castable, StrID, Symbol, XError};

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
)]
pub enum TmplClass {
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

impl Into<u16> for TmplClass {
    #[inline]
    fn into(self) -> u16 {
        return unsafe { mem::transmute::<TmplClass, u16>(self) };
    }
}

impl TryFrom<u16> for TmplClass {
    type Error = XError;

    #[inline]
    fn try_from(value: u16) -> Result<Self, XError> {
        if value as usize >= cardinality::<TmplClass>() {
            return Err(XError::overflow("TmplClass::try_from()"));
        }
        return Ok(unsafe { mem::transmute::<u16, TmplClass>(value) });
    }
}

#[typetag::deserialize(tag = "T")]
pub trait TmplAny: Debug {
    fn id(&self) -> StrID;
    fn class(&self) -> TmplClass;
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
        return TmplLevelRange { min, max };
    }
}

impl From<[u32; 2]> for TmplLevelRange {
    fn from(range: [u32; 2]) -> TmplLevelRange {
        return TmplLevelRange::new(range[0], range[1]);
    }
}

impl Into<[u32; 2]> for TmplLevelRange {
    fn into(self) -> [u32; 2] {
        return [self.min, self.max];
    }
}

impl From<(u32, u32)> for TmplLevelRange {
    fn from(range: (u32, u32)) -> TmplLevelRange {
        return TmplLevelRange::new(range.0, range.1);
    }
}

impl Into<(u32, u32)> for TmplLevelRange {
    fn into(self) -> (u32, u32) {
        return (self.min, self.max);
    }
}

impl<'de> serde::Deserialize<'de> for TmplLevelRange {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<TmplLevelRange, D::Error> {
        let range: [u32; 2] = serde::Deserialize::deserialize(deserializer)?;
        return Ok(TmplLevelRange::new(range[0], range[1]));
    }
}
