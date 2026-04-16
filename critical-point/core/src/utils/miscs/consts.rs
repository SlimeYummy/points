use critical_point_csgen::CsEnum;
use enum_iterator::{Sequence, cardinality};
use std::mem;

use crate::utils::collection::Bitsetable;
use crate::utils::error::{XError, XResult, xres};
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

//
// ActionType
//

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence, serde::Serialize, serde::Deserialize, CsEnum)]
pub enum ActionType {
    Empty,
    Idle,
    Move,
    General,
    Dodge,
    Guard,
    Aim,
    Hit,
}

rkyv_self!(ActionType);

impl From<ActionType> for u16 {
    #[inline]
    fn from(val: ActionType) -> Self {
        unsafe { mem::transmute::<ActionType, u16>(val) }
    }
}

impl TryFrom<u16> for ActionType {
    type Error = XError;

    #[inline]
    fn try_from(value: u16) -> XResult<Self> {
        if value as usize >= cardinality::<ActionType>() {
            return xres!(Overflow);
        }
        Ok(unsafe { mem::transmute::<u16, ActionType>(value) })
    }
}

impl From<ActionType> for rkyv::primitive::ArchivedU16 {
    #[inline]
    fn from(val: ActionType) -> Self {
        unsafe { mem::transmute::<ActionType, u16>(val) }.into()
    }
}

impl TryFrom<rkyv::primitive::ArchivedU16> for ActionType {
    type Error = XError;

    #[inline]
    fn try_from(val: rkyv::primitive::ArchivedU16) -> XResult<Self> {
        if val.to_native() as usize >= cardinality::<ActionType>() {
            return xres!(Overflow);
        }
        Ok(unsafe { mem::transmute::<u16, ActionType>(val.to_native()) })
    }
}

//
// AiTaskType
//

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence, serde::Serialize, serde::Deserialize, CsEnum)]
pub enum AiTaskType {
    Idle,
    Patrol,
}

rkyv_self!(AiTaskType);

impl From<AiTaskType> for u16 {
    #[inline]
    fn from(val: AiTaskType) -> Self {
        unsafe { mem::transmute::<AiTaskType, u16>(val) }
    }
}

impl TryFrom<u16> for AiTaskType {
    type Error = XError;

    #[inline]
    fn try_from(value: u16) -> XResult<Self> {
        if value as usize >= cardinality::<AiTaskType>() {
            return xres!(Overflow);
        }
        Ok(unsafe { mem::transmute::<u16, AiTaskType>(value) })
    }
}

impl From<AiTaskType> for rkyv::primitive::ArchivedU16 {
    #[inline]
    fn from(val: AiTaskType) -> Self {
        unsafe { mem::transmute::<AiTaskType, u16>(val) }.into()
    }
}

impl TryFrom<rkyv::primitive::ArchivedU16> for AiTaskType {
    type Error = XError;

    #[inline]
    fn try_from(val: rkyv::primitive::ArchivedU16) -> XResult<Self> {
        if val.to_native() as usize >= cardinality::<AiTaskType>() {
            return xres!(Overflow);
        }
        Ok(unsafe { mem::transmute::<u16, AiTaskType>(val.to_native()) })
    }
}

//
// HitType
//

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence, serde::Serialize, serde::Deserialize)]
pub enum HitType {
    Attack = 1,
    Health = 2,
    Guard = 3,
    Counter = 4,
}

rkyv_self!(HitType);
