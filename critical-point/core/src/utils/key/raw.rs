use approx::abs_diff_eq;
use critical_point_csgen::{CsEnum, CsIn};
use glam::Vec2;
use std::fmt;

#[repr(u8)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsEnum,
)]
pub enum RawKey {
    Move,
    View,
    Dodge,
    Jump,
    Guard,
    Interact,
    Lock,

    Attack1,
    Attack2,
    Attack3,
    Attack4,
    Attack5,
    Attack6,
    Attack7,
    Spell,
    Shot1,
    Shot2,
    Aim,
    Switch,

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
}

impl RawKey {
    #[inline]
    pub fn is_motion(&self) -> bool {
        use RawKey::*;
        matches!(self, Move | View)
    }

    #[inline]
    pub fn is_button(&self) -> bool {
        !self.is_motion()
    }

    #[inline]
    pub fn is_general(&self) -> bool {
        use RawKey::*;
        matches!(self, Move | View | Dodge | Jump | Guard | Interact | Lock)
    }

    #[inline]
    pub fn is_action(&self) -> bool {
        use RawKey::*;
        matches!(
            self,
            Attack1
                | Attack2
                | Attack3
                | Attack4
                | Attack5
                | Attack6
                | Attack7
                | Spell
                | Shot1
                | Shot2
                | Aim
                | Switch
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

    #[inline]
    pub fn is_derive(&self) -> bool {
        use RawKey::*;
        matches!(self, Derive1 | Derive2 | Derive3)
    }
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsIn,
)]
pub struct RawInput {
    pub key: RawKey,
    pub pressed: bool,
    pub motion: Vec2,
}

impl RawInput {
    #[inline]
    pub fn new_button(key: RawKey, pressed: bool) -> RawInput {
        RawInput {
            key,
            pressed,
            motion: Vec2::ZERO,
        }
    }

    #[inline]
    pub fn new_move(move_dir: Vec2) -> RawInput {
        RawInput {
            key: RawKey::Move,
            pressed: !abs_diff_eq!(move_dir, Vec2::ZERO),
            motion: move_dir,
        }
    }

    #[inline]
    pub fn new_view(view_radius: Vec2) -> RawInput {
        RawInput {
            key: RawKey::View,
            pressed: true,
            motion: view_radius,
        }
    }

    #[inline]
    pub fn is_button(&self) -> bool {
        self.key.is_button()
    }

    #[inline]
    pub fn is_motion(&self) -> bool {
        self.key.is_motion()
    }
}

impl fmt::Debug for RawInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_button() {
            f.debug_struct("RawInput")
                .field("key", &self.key)
                .field("pressed", &self.pressed)
                .finish()
        }
        else {
            f.debug_struct("RawInput")
                .field("key", &self.key)
                .field("motion", &self.motion)
                .finish()
        }
    }
}
