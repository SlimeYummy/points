use cirtical_point_csgen::{CsEnum, CsIn};
use glam::Vec2;
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
    CsEnum,
)]
pub enum KeyCode {
    Run,
    Dash,
    Walk,
    View,
    Dodge,
    Jump,
    Guard,
    Interact,
    Lock,
    LockSwitch,

    Attack1,
    Attack2,
    Attack3,
    Attack4,
    Attack5,
    Attack6,

    Shot1,
    Shot2,
    Aim,
    Reload,

    Skill1,
    Skill2,
    Skill3,
    Skill4,
    Skill5,
    Skill6,
    Skill7,
    Skill8,

    Derive1,
    Derive2,
    Derive3,

    Item1,
    Item2,
    Item3,
    Item4,
    Item5,
    Item6,
    Item7,
    Item8,

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
            Attack1
                | Attack2
                | Attack3
                | Attack4
                | Attack5
                | Attack6
                | Shot1
                | Shot2
                | Aim
                | Reload
                | Skill1
                | Skill2
                | Skill3
                | Skill4
                | Skill5
                | Skill6
                | Skill7
                | Skill8
        )
    }

    pub fn is_derive(&self) -> bool {
        use KeyCode::*;
        matches!(self, Derive1 | Derive2 | Derive3)
    }

    pub fn is_virtual(&self) -> bool {
        use KeyCode::*;
        matches!(self, Idle | Break1 | Break2 | Break3)
    }

    pub fn is_non_virtual(&self) -> bool {
        !self.is_virtual()
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsIn,
)]
pub struct KeyEvent {
    pub key: KeyCode,
    pub pressed: bool,
    pub motion: Vec2,
}

impl KeyEvent {
    #[inline]
    pub fn new_button(key: KeyCode, pressed: bool) -> KeyEvent {
        KeyEvent {
            key,
            pressed,
            motion: Vec2::ZERO,
        }
    }

    #[inline]
    pub fn new_motion(key: KeyCode, motion: Vec2) -> KeyEvent {
        KeyEvent {
            key,
            pressed: true,
            motion,
        }
    }
}
