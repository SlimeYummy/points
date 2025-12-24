use critical_point_csgen::CsEnum;
use glam::Vec3A;
use glam_ext::Vec2xz;

use super::raw::RawKey;
use crate::consts::{DEFAULT_VIEW_DIR_2D, DEFAULT_VIEW_DIR_3D};
use crate::utils::macros::rkyv_self;
use crate::utils::serde_by;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, CsEnum)]
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

rkyv_self!(VirtualKey);

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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct VirtualInput {
    pub id: u64,
    pub frame: u32,
    pub key: VirtualKey,
    pub pressed: bool,
    pub view_dir_2d: Vec2xz,
    pub view_dir_3d: Vec3A,
    pub world_move_dir: Vec2xz,
}

impl VirtualInput {
    #[inline]
    pub fn new(id: u64, frame: u32, key: VirtualKey, pressed: bool) -> VirtualInput {
        VirtualInput::new_ex(
            id,
            frame,
            key,
            pressed,
            DEFAULT_VIEW_DIR_2D,
            DEFAULT_VIEW_DIR_3D,
            Vec2xz::ZERO,
        )
    }

    #[inline]
    pub fn new_ex(
        id: u64,
        frame: u32,
        key: VirtualKey,
        pressed: bool,
        view_dir_2d: Vec2xz,
        view_dir_3d: Vec3A,
        world_move_dir: Vec2xz,
    ) -> VirtualInput {
        VirtualInput {
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

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "dir", content = "cos")]
pub enum InputDir {
    // All variants are stored the cos values in range [0, 180].
    Forward(f32),
    Backward(f32),
    Left(f32),
    Right(f32),
}

rkyv_self!(InputDir);

impl InputDir {
    #[inline]
    pub fn cos(&self) -> f32 {
        match self {
            InputDir::Forward(cos) => *cos,
            InputDir::Backward(cos) => *cos,
            InputDir::Left(cos) => *cos,
            InputDir::Right(cos) => *cos,
        }
    }

    #[inline]
    pub fn radius(&self) -> f32 {
        libm::acosf(self.cos())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualKeyDir {
    pub key: VirtualKey,
    pub dir: Option<InputDir>,
}

rkyv_self!(VirtualKeyDir);
serde_by!(
    VirtualKeyDir,
    (VirtualKey, Option<InputDir>),
    VirtualKeyDir::from,
    VirtualKeyDir::to_tuple
);

impl VirtualKeyDir {
    #[inline]
    pub fn new(key: VirtualKey, dir: Option<InputDir>) -> VirtualKeyDir {
        VirtualKeyDir { key, dir }
    }

    #[inline]
    pub fn to_tuple(&self) -> (VirtualKey, Option<InputDir>) {
        (self.key, self.dir)
    }
}

impl From<VirtualKey> for VirtualKeyDir {
    fn from(key: VirtualKey) -> VirtualKeyDir {
        VirtualKeyDir::new(key, None)
    }
}

impl From<(VirtualKey, Option<InputDir>)> for VirtualKeyDir {
    fn from((key, dir): (VirtualKey, Option<InputDir>)) -> VirtualKeyDir {
        VirtualKeyDir::new(key, dir)
    }
}

impl From<VirtualKeyDir> for (VirtualKey, Option<InputDir>) {
    fn from(virtual_key_dir: VirtualKeyDir) -> (VirtualKey, Option<InputDir>) {
        virtual_key_dir.to_tuple()
    }
}
