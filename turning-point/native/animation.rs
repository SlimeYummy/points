use glam::{Quat, Vec3, Vec3Swizzles};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use ozz_animation_rs::{Animation, Archive, Skeleton, Track};
use std::collections::HashMap;
use cirtical_point_core::animation::RootMotionTrack;

use crate::error::{cp_err, ozz_err};

#[napi(object)]
pub struct SkeletonMeta {
    pub version: u32,
    #[napi(js_name = "num_joints")]
    pub num_joints: u32,
    #[napi(js_name = "joint_names")]
    pub joint_names: HashMap<String, i16>,
    #[napi(js_name = "joint_parents")]
    pub joint_parents: Vec<i16>,
}

#[napi]
pub fn load_skeleton_meta(path: String, with_joints: bool) -> Result<SkeletonMeta> {
    let mut archive = Archive::from_path(path).map_err(ozz_err)?;
    let ozz_meta = Skeleton::read_meta(&mut archive, with_joints).map_err(ozz_err)?;

    Ok(SkeletonMeta {
        version: Skeleton::version(),
        num_joints: ozz_meta.num_joints,
        joint_names: HashMap::from_iter(ozz_meta.joint_names.into_iter()),
        joint_parents: ozz_meta.joint_parents,
    })
}

#[napi(object)]
pub struct AnimationMeta {
    pub version: u32,
    pub duration: f64,
    #[napi(js_name = "num_tracks")]
    pub num_tracks: u32,
    pub name: String,
    #[napi(js_name = "translations_count")]
    pub translations_count: u32,
    #[napi(js_name = "rotations_count")]
    pub rotations_count: u32,
    #[napi(js_name = "scales_count")]
    pub scales_count: u32,
}

#[napi]
pub fn load_animation_meta(path: String) -> Result<AnimationMeta> {
    let mut archive = Archive::from_path(path).map_err(ozz_err)?;
    let ozz_meta = Animation::read_meta(&mut archive).map_err(ozz_err)?;

    Ok(AnimationMeta {
        version: Animation::version(),
        duration: ozz_meta.duration as f64,
        num_tracks: ozz_meta.num_tracks,
        name: ozz_meta.name,
        translations_count: ozz_meta.translations_count,
        rotations_count: ozz_meta.rotations_count,
        scales_count: ozz_meta.scales_count,
    })
}

#[napi(object)]
pub struct RootMotionMeta {
    pub version: u32,
    #[napi(js_name = "has_position")]
    pub has_position: bool,
    #[napi(js_name = "has_rotation")]
    pub has_rotation: bool,
    #[napi(js_name = "max_distance")]
    pub max_distance: f64,
}

#[napi]
pub fn load_root_motion_meta(path: String) -> Result<RootMotionMeta> {
    let root_motion = RootMotionTrack::from_path(path).map_err(cp_err)?;
    Ok(RootMotionMeta {
        version: Track::<Vec3>::version(),
        has_position: root_motion.has_position(),
        has_rotation: root_motion.has_rotation(),
        max_distance: root_motion.max_xz_distance() as f64,
    })
}
