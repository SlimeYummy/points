use critical_point_csgen::{CsIn, CsOut};
use std::ops::RangeInclusive;

use crate::utils::id::TmplID;
use crate::utils::macros::{rkyv_self, serde_by};
use crate::utils::symbol::Symbol;

//
// TmplIDLevel
//

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, CsIn)]
pub struct TmplIDLevel {
    pub id: TmplID,
    pub level: u32,
}

rkyv_self!(TmplIDLevel);
serde_by!(TmplIDLevel, (TmplID, u32), TmplIDLevel::from, TmplIDLevel::to_tuple);

impl TmplIDLevel {
    #[inline]
    pub fn new(id: TmplID, level: u32) -> TmplIDLevel {
        TmplIDLevel { id: id.clone(), level }
    }

    #[inline]
    fn to_tuple(&self) -> (TmplID, u32) {
        (self.id, self.level)
    }
}

impl From<(TmplID, u32)> for TmplIDLevel {
    #[inline]
    fn from((id, level): (TmplID, u32)) -> Self {
        TmplIDLevel { id, level }
    }
}

impl From<TmplIDLevel> for (TmplID, u32) {
    #[inline]
    fn from(val: TmplIDLevel) -> Self {
        val.to_tuple()
    }
}

//
// TmplIDPlus
//

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, CsIn)]
pub struct TmplIDPlus {
    pub id: TmplID,
    pub plus: u32,
}

rkyv_self!(TmplIDPlus);
serde_by!(TmplIDPlus, (TmplID, u32), TmplIDPlus::from, TmplIDPlus::to_tuple);

impl TmplIDPlus {
    #[inline]
    pub fn new(id: TmplID, plus: u32) -> TmplIDPlus {
        TmplIDPlus { id: id.clone(), plus }
    }

    #[inline]
    pub fn to_tuple(&self) -> (TmplID, u32) {
        (self.id, self.plus)
    }
}

impl From<(TmplID, u32)> for TmplIDPlus {
    #[inline]
    fn from((id, plus): (TmplID, u32)) -> Self {
        TmplIDPlus { id, plus }
    }
}

impl From<TmplIDPlus> for (TmplID, u32) {
    #[inline]
    fn from(val: TmplIDPlus) -> Self {
        val.to_tuple()
    }
}

//
// LevelRange
//

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LevelRange {
    pub min: u32,
    pub max: u32,
}

rkyv_self!(LevelRange);
serde_by!(LevelRange, [u32; 2], LevelRange::from, LevelRange::to_array);

impl LevelRange {
    #[inline]
    pub fn new(min: u32, max: u32) -> LevelRange {
        LevelRange { min, max }
    }

    #[inline]
    pub fn to_array(&self) -> [u32; 2] {
        [self.min, self.max]
    }

    #[inline]
    pub fn to_tuple(&self) -> (u32, u32) {
        (*self).into()
    }

    #[inline]
    pub fn to_range(&self) -> RangeInclusive<u32> {
        self.min..=self.max
    }
}

impl From<[u32; 2]> for LevelRange {
    #[inline]
    fn from(range: [u32; 2]) -> LevelRange {
        LevelRange::new(range[0], range[1])
    }
}

impl From<LevelRange> for [u32; 2] {
    #[inline]
    fn from(val: LevelRange) -> Self {
        val.to_array()
    }
}

impl From<(u32, u32)> for LevelRange {
    #[inline]
    fn from(range: (u32, u32)) -> LevelRange {
        LevelRange::new(range.0, range.1)
    }
}

impl From<LevelRange> for (u32, u32) {
    #[inline]
    fn from(val: LevelRange) -> Self {
        val.to_tuple()
    }
}

impl From<LevelRange> for RangeInclusive<u32> {
    #[inline]
    fn from(val: LevelRange) -> Self {
        val.to_range()
    }
}

//
// PiecePlus
//

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PiecePlus {
    pub piece: u32,
    pub plus: u32,
}

rkyv_self!(PiecePlus);
serde_by!(PiecePlus, (u32, u32), PiecePlus::from, PiecePlus::to_tuple);

impl PiecePlus {
    #[inline]
    pub fn new(piece: u32, plus: u32) -> Self {
        Self { piece, plus }
    }

    #[inline]
    pub fn to_tuple(&self) -> (u32, u32) {
        (self.piece, self.plus)
    }
}

impl From<(u32, u32)> for PiecePlus {
    #[inline]
    fn from((piece, plus): (u32, u32)) -> Self {
        Self { piece, plus }
    }
}

impl From<PiecePlus> for (u32, u32) {
    #[inline]
    fn from(val: PiecePlus) -> Self {
        val.to_tuple()
    }
}

//
// JewelSlots
//

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct JewelSlots {
    pub special: u8,
    pub attack: u8,
    pub defense: u8,
}

rkyv_self!(JewelSlots);
serde_by!(JewelSlots, [u8; 3], JewelSlots::from, JewelSlots::to_array);

impl JewelSlots {
    #[inline]
    pub fn new(special: u8, attack: u8, defense: u8) -> JewelSlots {
        JewelSlots {
            special,
            attack,
            defense,
        }
    }

    #[inline]
    pub fn merge(&self, other: &JewelSlots) -> JewelSlots {
        JewelSlots {
            special: self.special + other.special,
            attack: self.attack + other.attack,
            defense: self.defense + other.defense,
        }
    }

    #[inline]
    pub fn append(&mut self, other: &JewelSlots) {
        *self = self.merge(other);
    }

    #[inline]
    pub fn to_tuple(&self) -> (u8, u8, u8) {
        (self.special, self.attack, self.defense)
    }

    #[inline]
    pub fn to_array(&self) -> [u8; 3] {
        [self.special, self.attack, self.defense]
    }
}

impl From<(u8, u8, u8)> for JewelSlots {
    #[inline]
    fn from((special, attack, defense): (u8, u8, u8)) -> Self {
        JewelSlots {
            special,
            attack,
            defense,
        }
    }
}

impl From<JewelSlots> for (u8, u8, u8) {
    #[inline]
    fn from(val: JewelSlots) -> Self {
        val.to_tuple()
    }
}

impl From<[u8; 3]> for JewelSlots {
    #[inline]
    fn from([special, attack, defense]: [u8; 3]) -> Self {
        JewelSlots {
            special,
            attack,
            defense,
        }
    }
}

impl From<JewelSlots> for [u8; 3] {
    #[inline]
    fn from(val: JewelSlots) -> Self {
        val.to_array()
    }
}

//
// CustomEvent
//

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, CsOut)]
#[cs_attr(Value, Partial)]
pub struct CustomEvent {
    pub source: TmplID,
    pub name: Symbol,
}

rkyv_self!(CustomEvent);
serde_by!(CustomEvent, (TmplID, Symbol), CustomEvent::from, CustomEvent::to_tuple);

impl CustomEvent {
    #[inline]
    pub fn new(source: TmplID, name: Symbol) -> Self {
        Self { source, name }
    }

    #[inline]
    pub fn to_tuple(&self) -> (TmplID, Symbol) {
        (self.source, self.name)
    }
}

impl From<(TmplID, Symbol)> for CustomEvent {
    #[inline]
    fn from((source, name): (TmplID, Symbol)) -> Self {
        Self { source, name }
    }
}
