use static_assertions::const_assert;
use std::mem;

use crate::utils::{XError, XResult};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub enum InputKey {
    Run,
    Dash,
    Walk,
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

    Move,
    X1,
    X2,
    X3,
    X4,
    X12,
    X34,

    Idle,
    Break1,
    Break2,
    Break3,
}

const_assert!(InputKey::Idle as u32 <= 64u32);

impl InputKey {
    pub fn from_game(value: u32) -> XResult<InputKey> {
        if value >= InputKey::Idle as u32 {
            return Err(XError::bad_argument("InputKey::from_game()"));
        }
        let key: InputKey = unsafe { mem::transmute(value as u8) };
        return Ok(key);
    }

    pub fn is_general(&self) -> bool {
        use InputKey::*;
        return matches!(
            self,
            Run | Dash | Walk | View | Dodge | Guard | Lock | LockSwitch | Interact
        );
    }

    pub fn is_action(&self) -> bool {
        use InputKey::*;
        return matches!(
            self,
            A1 | A2 | A3 | A4 | A5 | B1 | B2 | B3 | C1 | C2 | C3 | D1 | D2 | Shoot | Aim | Reload
        );
    }

    pub fn is_derive(&self) -> bool {
        use InputKey::*;
        return matches!(self, Move | X1 | X2 | X3 | X4 | X12 | X34);
    }

    pub fn is_virtual(&self) -> bool {
        use InputKey::*;
        return matches!(self, Idle | Break1 | Break2 | Break3);
    }

    pub fn is_non_virtual(&self) -> bool {
        use InputKey::*;
        return !matches!(self, Idle | Break1 | Break2 | Break3);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct InputKeyEvent {
    pub key: InputKey,
    pub pressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct InputAxesEvent {
    pub key: InputKey,
    pub value: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum InputEvent {
    Key(InputKeyEvent),
    Axes(InputAxesEvent),
}

impl InputEvent {
    #[inline]
    pub fn new_key(key: InputKey, pressed: bool) -> InputEvent {
        return InputEvent::Key(InputKeyEvent { key, pressed });
    }

    #[inline]
    pub fn new_axes(key: InputKey, value: [f32; 2]) -> InputEvent {
        return InputEvent::Axes(InputAxesEvent { key, value });
    }

    #[inline]
    pub fn key(&self) -> InputKey {
        return match self {
            InputEvent::Key(ev) => ev.key,
            InputEvent::Axes(ev) => ev.key,
        };
    }
}
