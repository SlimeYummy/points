use crate::utils::{XError, XResult};
use enum_iterator::{cardinality, Sequence};
use std::mem;

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
pub enum OperationKey {
    Move = 1,
    View,
    Dodge,
    Guard,
    Lock,
    LockSwitch,
    Interact,

    A1,
    A2,
    A3,
    A4,
    A5,
    B1,
    B2,
    B3,
    C1,
    C2,
    C3,
    D1,
    D2,
    Aim,
    Shoot,
    Reload,

    X1,
    X2,
    X3,
    X4,
}

impl TryFrom<u32> for OperationKey {
    type Error = XError;

    fn try_from(value: u32) -> XResult<OperationKey> {
        if value < 1 || value >= cardinality::<OperationKey>() as u32 {
            return Err(XError::BadArgument);
        }
        let key: OperationKey = unsafe { mem::transmute(value as u8) };
        return Ok(key);
    }
}

impl OperationKey {
    pub fn is_general(&self) -> bool {
        use OperationKey::*;
        return matches!(
            self,
            Move | View | Dodge | Guard | Lock | LockSwitch | Interact
        );
    }

    pub fn is_action(&self) -> bool {
        use OperationKey::*;
        return matches!(
            self,
            A1 | A2 | A3 | A4 | A5 | B1 | B2 | B3 | C1 | C2 | C3 | D1 | D2 | Shoot | Aim | Reload
        );
    }

    pub fn is_derive(&self) -> bool {
        use OperationKey::*;
        return matches!(self, X1 | X2 | X3 | X4);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OperationKeyEvent {
    pub idx: u32,
    pub key: OperationKey,
    pub pressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OperationAxesEvent {
    pub idx: u32,
    pub key: OperationKey,
    pub value: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationEvent {
    Key(OperationKeyEvent),
    Axes(OperationAxesEvent),
}

impl OperationEvent {
    pub fn opt_key(&self) -> OperationKey {
        return match self {
            OperationEvent::Key(ev) => ev.key,
            OperationEvent::Axes(ev) => ev.key,
        };
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum LogicKey {
    Move = 1,
    View,
    Dodge,
    Guard,
    Lock,
    LockSwitch,
    Interact,

    A1,
    A2,
    A3,
    A4,
    A5,
    B1,
    B2,
    B3,
    C1,
    C2,
    C3,
    D1,
    D2,
    Aim,
    Shoot,
    Reload,

    X1,
    X2,
    X3,
    X4,
    X12,
    X34,

    Idle,
    Ready,
    Walk,
    Run,
    Dash,
}

impl From<OperationKey> for LogicKey {
    fn from(key: OperationKey) -> Self {
        return match key {
            OperationKey::Move => LogicKey::Move,
            OperationKey::View => LogicKey::View,
            OperationKey::Dodge => LogicKey::Dodge,
            OperationKey::Guard => LogicKey::Guard,
            OperationKey::Lock => LogicKey::Lock,
            OperationKey::LockSwitch => LogicKey::LockSwitch,
            OperationKey::Interact => LogicKey::Interact,
            OperationKey::A1 => LogicKey::A1,
            OperationKey::A2 => LogicKey::A2,
            OperationKey::A3 => LogicKey::A3,
            OperationKey::A4 => LogicKey::A4,
            OperationKey::A5 => LogicKey::A5,
            OperationKey::B1 => LogicKey::B1,
            OperationKey::B2 => LogicKey::B2,
            OperationKey::B3 => LogicKey::B3,
            OperationKey::C1 => LogicKey::C1,
            OperationKey::C2 => LogicKey::C2,
            OperationKey::C3 => LogicKey::C3,
            OperationKey::D1 => LogicKey::D1,
            OperationKey::D2 => LogicKey::D2,
            OperationKey::Aim => LogicKey::Aim,
            OperationKey::Shoot => LogicKey::Shoot,
            OperationKey::Reload => LogicKey::Reload,
            OperationKey::X1 => LogicKey::X1,
            OperationKey::X2 => LogicKey::X2,
            OperationKey::X3 => LogicKey::X3,
            OperationKey::X4 => LogicKey::X4,
        };
    }
}

impl LogicKey {
    pub fn is_primary(&self) -> bool {
        use LogicKey::*;
        return matches!(
            self,
            Move | View | Dodge | Guard | Lock | LockSwitch | Interact | A1 | A2 | A3 | A4 | A5
        );
    }

    pub fn is_derive(&self) -> bool {
        use LogicKey::*;
        return matches!(self, X1 | X2 | X3 | X4 | X12 | X34);
    }

    pub fn is_operation(&self) -> bool {
        return !self.is_virtual();
    }

    pub fn is_virtual(&self) -> bool {
        use LogicKey::*;
        return matches!(self, X12 | X34 | Idle | Ready | Walk | Run | Dash);
    }
}
