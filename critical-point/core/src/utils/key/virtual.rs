use cirtical_point_csgen::CsEnum;
use glam::{Vec2, Vec3A};

use super::raw::RawKey;

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
pub enum VirtualKey {
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

    Idle,
    Walk,
    Run,
    Dash,
    Break1,
    Break2,
    Break3,
}

impl From<RawKey> for VirtualKey {
    fn from(key: RawKey) -> VirtualKey {
        use VirtualKey::*;
        match key {
            RawKey::Move => Move,
            RawKey::View => View,
            RawKey::Dodge => Dodge,
            RawKey::Jump => Jump,
            RawKey::Guard => Guard,
            RawKey::Interact => Interact,
            RawKey::Lock => Lock,
            RawKey::Attack1 => Attack1,
            RawKey::Attack2 => Attack2,
            RawKey::Attack3 => Attack3,
            RawKey::Attack4 => Attack4,
            RawKey::Attack5 => Attack5,
            RawKey::Attack6 => Attack6,
            RawKey::Attack7 => Attack7,
            RawKey::Spell => Spell,
            RawKey::Shot1 => Shot1,
            RawKey::Shot2 => Shot2,
            RawKey::Aim => Aim,
            RawKey::Switch => Switch,
            RawKey::Skill1 => Skill1,
            RawKey::Skill2 => Skill2,
            RawKey::Skill3 => Skill3,
            RawKey::Skill4 => Skill4,
            RawKey::Skill5 => Skill5,
            RawKey::Skill6 => Skill6,
            RawKey::Skill7 => Skill7,
            RawKey::Skill8 => Skill8,
            RawKey::Derive1 => Derive1,
            RawKey::Derive2 => Derive2,
            RawKey::Derive3 => Derive3,
            RawKey::Item1 => Item1,
            RawKey::Item2 => Item2,
            RawKey::Item3 => Item3,
            RawKey::Item4 => Item4,
            RawKey::Item5 => Item5,
            RawKey::Item6 => Item6,
            RawKey::Item7 => Item7,
            RawKey::Item8 => Item8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Deserialize)]
pub struct VirtualEvent {
    pub id: u64,
    pub frame: u32,
    pub key: VirtualKey,
    pub pressed: bool,
    pub view_dir_2d: Vec2,
    pub view_dir_3d: Vec3A,
    pub world_move_dir: Vec2,
}

impl VirtualEvent {
    #[inline]
    pub fn new(id: u64, frame: u32, key: VirtualKey, pressed: bool) -> VirtualEvent {
        VirtualEvent::new_ex(id, frame, key, pressed, Vec2::ZERO, Vec3A::ZERO, Vec2::ZERO)
    }

    #[inline]
    pub fn new_ex(
        id: u64,
        frame: u32,
        key: VirtualKey,
        pressed: bool,
        view_dir_2d: Vec2,
        view_dir_3d: Vec3A,
        world_move_dir: Vec2,
    ) -> VirtualEvent {
        VirtualEvent {
            id,
            frame,
            key,
            pressed,
            view_dir_2d,
            view_dir_3d,
            world_move_dir,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum VirtualDirection {
    // All variants are stored the cos values in range [0, 180].
    Forward(f32),
    Backward(f32),
    Left(f32),
    Right(f32),
}

impl VirtualDirection {
    #[inline]
    pub fn cos(&self) -> f32 {
        match self {
            VirtualDirection::Forward(cos) => *cos,
            VirtualDirection::Backward(cos) => *cos,
            VirtualDirection::Left(cos) => *cos,
            VirtualDirection::Right(cos) => *cos,
        }
    }

    #[inline]
    pub fn radius(&self) -> f32 {
        libm::acosf(self.cos())
    }
}
