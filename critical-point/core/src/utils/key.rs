use cirtical_point_csgen::CsGen;
use static_assertions::const_assert;
use std::mem;

use crate::utils::{XError, XResult};

#[repr(u8)]
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
    CsGen,
)]
pub enum KeyCode {
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

const_assert!(KeyCode::Idle as u32 <= 64u32);

impl KeyCode {
    pub fn from_game(value: u32) -> XResult<KeyCode> {
        if value >= KeyCode::Idle as u32 {
            return Err(XError::bad_argument("KeyCode::from_game()"));
        }
        let key: KeyCode = unsafe { mem::transmute(value as u8) };
        Ok(key)
    }

    pub fn is_motion(&self) -> bool {
        use KeyCode::*;
        matches!(self, Run | Dash | Walk | View)
    }

    pub fn is_general(&self) -> bool {
        use KeyCode::*;
        matches!(
            self,
            Run | Dash | Walk | View | Dodge | Guard | Lock | LockSwitch | Interact
        )
    }

    pub fn is_action(&self) -> bool {
        use KeyCode::*;
        matches!(
            self,
            A1 | A2 | A3 | A4 | A5 | B1 | B2 | B3 | C1 | C2 | C3 | D1 | D2 | Shoot | Aim | Reload
        )
    }

    pub fn is_derive(&self) -> bool {
        use KeyCode::*;
        matches!(self, Move | X1 | X2 | X3 | X4 | X12 | X34)
    }

    pub fn is_virtual(&self) -> bool {
        use KeyCode::*;
        matches!(self, Idle | Break1 | Break2 | Break3)
    }

    pub fn is_non_virtual(&self) -> bool {
        use KeyCode::*;
        !matches!(self, Idle | Break1 | Break2 | Break3)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsGen,
)]
#[cs_attr(Cs, Class)]
pub struct KeyEvent {
    pub key: KeyCode,
    pub pressed: bool,
    pub motion: [f32; 2],
}

impl KeyEvent {
    #[inline]
    pub fn new_button(key: KeyCode, pressed: bool) -> KeyEvent {
        KeyEvent {
            key,
            pressed,
            motion: [0.0, 0.0],
        }
    }

    #[inline]
    pub fn new_motion(key: KeyCode, motion: [f32; 2]) -> KeyEvent {
        KeyEvent {
            key,
            pressed: true,
            motion,
        }
    }
}
