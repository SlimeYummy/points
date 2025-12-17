use critical_point_core::animation::{RootTrackName, RootMotion, WeaponMotion};
use glam::{Vec3, Vec3Swizzles};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use ozz_animation_rs::{Animation, Archive, Skeleton, Track};
use std::collections::HashMap;

use crate::error::{cp_err_msg, ozz_err_msg};

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
    let mut archive = match Archive::from_path(&path) {
        Ok(archive) => archive,
        Err(err) => return Err(ozz_err_msg(err, &path)),
    };
    let ozz_meta = match Skeleton::read_meta(&mut archive, with_joints) {
        Ok(meta) => meta,
        Err(err) => return Err(ozz_err_msg(err, &path)),
    };

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
    let mut archive = match Archive::from_path(&path) {
        Ok(archive) => archive,
        Err(err) => return Err(ozz_err_msg(err, &path)),
    };
    let ozz_meta = match Animation::read_meta(&mut archive) {
        Ok(meta) => meta,
        Err(err) => return Err(ozz_err_msg(err, &path)),
    };

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
    #[napi(js_name = "position_default")]
    pub position_default: Option<RootMotionPositionMeta>,
    #[napi(js_name = "position_move")]
    pub position_move: Option<RootMotionPositionMeta>,
    #[napi(js_name = "position_move_ex")]
    pub position_move_ex: Option<RootMotionPositionMeta>,
    #[napi(js_name = "has_rotation")]
    pub has_rotation: bool,
}

#[napi(object)]
pub struct RootMotionPositionMeta {
    #[napi(js_name = "whole_distance")]
    pub whole_distance: f64,
    #[napi(js_name = "whole_distance_xz")]
    pub whole_distance_xz: f64,
    #[napi(js_name = "whole_distance_y")]
    pub whole_distance_y: f64,
}

#[napi]
pub fn load_root_motion_meta(path: String) -> Result<RootMotionMeta> {
    let root_motion = match RootMotion::from_path(&path) {
        Ok(root_motion) => root_motion,
        Err(err) => return Err(cp_err_msg(err, &path)),
    };

    let mut position_default = None;
    if root_motion.has_position(RootTrackName::Default) {
        let whole = root_motion.whole_position(RootTrackName::Default);
        position_default = Some(RootMotionPositionMeta {
            whole_distance: whole.length() as f64,
            whole_distance_xz: whole.xz().length() as f64,
            whole_distance_y: whole.y as f64,
        });
    }

    let mut position_move = None;
    if root_motion.has_position(RootTrackName::Move) {
        let whole = root_motion.whole_position(RootTrackName::Move);
        position_move = Some(RootMotionPositionMeta {
            whole_distance: whole.length() as f64,
            whole_distance_xz: whole.xz().length() as f64,
            whole_distance_y: whole.y as f64,
        });
    }

    let mut position_move_ex = None;
    if root_motion.has_position(RootTrackName::MoveEx) {
        let whole = root_motion.whole_position(RootTrackName::MoveEx);
        position_move_ex = Some(RootMotionPositionMeta {
            whole_distance: whole.length() as f64,
            whole_distance_xz: whole.xz().length() as f64,
            whole_distance_y: whole.y as f64,
        });
    }

    Ok(RootMotionMeta {
        version: Track::<Vec3>::version(),
        position_default,
        position_move,
        position_move_ex,
        has_rotation: root_motion.has_rotation(),
    })
}

#[napi(object)]
pub struct WeaponMotionTracksMeta {
    pub version: u32,
    pub count: u32,
    pub names: Vec<String>,
}

#[napi]
pub fn load_weapon_trajectory_meta(path: String) -> Result<WeaponMotionTracksMeta> {
    let weapon_motion = match WeaponMotion::from_path(&path) {
        Ok(weapon_motion) => weapon_motion,
        Err(err) => return Err(cp_err_msg(err, &path)),
    };
    Ok(WeaponMotionTracksMeta {
        version: Track::<Vec3>::version(),
        count: weapon_motion.len() as u32,
        names: weapon_motion.iter().map(|w| w.name().to_string()).collect(),
    })
}
