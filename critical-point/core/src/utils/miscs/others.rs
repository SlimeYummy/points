use critical_point_csgen::CsOut;
use crate::utils::{CsQuat, CsVec3A, Symbol};

#[repr(C)]
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsOut,
)]
#[rkyv(derive(Debug))]
pub struct AnimationFileMeta {
    pub files: Symbol,
    pub root_motion: bool,
    pub weapon_motion: bool,
}

impl AnimationFileMeta {
    #[inline]
    pub fn new(files: Symbol, root_motion: bool, weapon_motion: bool) -> Self {
        Self {
            files,
            root_motion,
            weapon_motion,
        }
    }
}

#[repr(C)]
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsOut,
)]
pub struct WeaponMotionIsometry {
    pub name: Symbol,
    pub position: CsVec3A,
    pub rotation: CsQuat,
}
