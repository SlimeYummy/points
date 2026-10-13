use critical_point_macros::{csharp_enum, csharp_out};
use glam::{Quat, Vec3A};
use glam_ext::{Mat4, Transform3A};
use ozz_animation_rs::{LocalToModelJob, LocalToModelJobRef, Skeleton, SoaTransform};

use crate::animation::weapon_control::WeaponState;
use crate::utils::{Symbol, XResult, rkyv_self, xfrom, xres};

#[repr(C)]
#[csharp_out(Value)]
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct AnimationFileMeta {
    pub files: Symbol,
    pub root_motion: bool,
    pub weapon_control: bool,
    pub shape_key: bool,
}

impl AnimationFileMeta {
    #[inline]
    pub fn new(files: Symbol, root_motion: bool, weapon_control: bool, shape_key: bool) -> Self {
        Self {
            files,
            root_motion,
            weapon_control,
            shape_key,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WeaponJoints {
    pub left_root: i16,
    pub right_root: i16,
}

impl WeaponJoints {
    pub fn new(skeleton: &Skeleton) -> XResult<WeaponJoints> {
        let left_root = match skeleton.joint_by_name("LeftWeaponRoot") {
            Some(j) => j,
            None => return xres!(BadAsset; "LeftWeaponRoot not found"),
        };
        let right_root = match skeleton.joint_by_name("RightWeaponRoot") {
            Some(j) => j,
            None => return xres!(BadAsset; "RightWeaponRoot not found"),
        };
        Ok(WeaponJoints { left_root, right_root })
    }

    pub fn try_new(skeleton: &Skeleton) -> Option<WeaponJoints> {
        let left_root = skeleton.joint_by_name("LeftWeaponRoot")?;
        let right_root = skeleton.joint_by_name("RightWeaponRoot")?;
        Some(WeaponJoints { left_root, right_root })
    }
}

#[repr(u8)]
#[csharp_enum]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WeaponSlot {
    Left,
    Right,
}

rkyv_self!(WeaponSlot);

#[repr(C)]
#[csharp_out(Value)]
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
pub struct WeaponValue {
    pub slot: WeaponSlot,
    pub left_hand: bool,
    pub right_hand: bool,
    pub position: Vec3A,
    pub rotation: Quat,
    pub left_hand_position: Vec3A,
    pub right_hand_position: Vec3A,
}

impl WeaponValue {
    pub fn new(slot: WeaponSlot, state: WeaponState, transform: &Mat4) -> WeaponValue {
        let (_, rot, pos) = transform.to_scale_rotation_translation();
        WeaponValue {
            slot,
            left_hand: state.left_hand,
            right_hand: state.right_hand,
            position: pos.into(),
            rotation: rot,
            left_hand_position: Vec3A::ZERO,
            right_hand_position: Vec3A::ZERO,
        }
    }
}

#[repr(C)]
#[csharp_out(Value)]
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct ShapeKeyValue {
    pub name: Symbol,
    pub value: f32,
}

pub fn soa_transforms_to_transforms(soa_transforms: &[SoaTransform], transforms: &mut [Transform3A]) -> XResult<()> {
    if transforms.len() < soa_transforms.len() * 4 {
        return xres!(BadArgument; "buffer too short");
    }
    for (idx, out) in transforms.iter_mut().take(soa_transforms.len() * 4).enumerate() {
        *out = soa_transforms[idx / 4].transform(idx % 4);
    }
    Ok(())
}

pub fn soa_transforms_to_matrices(soa_transforms: &[SoaTransform], matrices: &mut [Mat4]) -> XResult<()> {
    if matrices.len() < soa_transforms.len() * 4 {
        return xres!(BadArgument; "buffer too short");
    }
    for (idx, out) in matrices.iter_mut().take(soa_transforms.len() * 4).enumerate() {
        *out = Mat4::from(soa_transforms[idx / 4].transform(idx % 4));
    }
    Ok(())
}

pub fn matrices_to_transforms(matrices: &[Mat4], transforms: &mut [Transform3A]) -> XResult<()> {
    if transforms.len() < matrices.len() {
        return xres!(BadArgument; "buffer too short");
    }
    for (idx, out) in transforms.iter_mut().take(matrices.len()).enumerate() {
        *out = Transform3A::from_mat4(matrices[idx]);
    }
    Ok(())
}

pub fn rest_poses_to_model_matrices(skeleton: &Skeleton, matrices: &mut [Mat4]) -> XResult<()> {
    let mut l2m: LocalToModelJobRef<'_> = LocalToModelJob::default();
    l2m.set_skeleton(&skeleton);
    l2m.set_input(skeleton.joint_rest_poses());
    l2m.set_output(matrices);
    l2m.run().map_err(xfrom!())?;
    Ok(())
}

pub fn rest_poses_to_model_transforms(skeleton: &Skeleton, transfroms: &mut [Transform3A]) -> XResult<()> {
    let mut matrices = vec![Mat4::ZERO; skeleton.num_joints()];
    rest_poses_to_model_matrices(skeleton, &mut matrices)?;
    matrices_to_transforms(&matrices, transfroms)?;
    Ok(())
}
