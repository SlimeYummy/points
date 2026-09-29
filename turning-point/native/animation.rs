use critical_point_core::animation::{self as cp_anim, RootMotion, RootTrackName};
use napi::bindgen_prelude::*;
use napi::{Error, Status};
use napi_derive::napi;
use ozz_animation_rs::{Animation, Archive, Skeleton};
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
    let cp_meta = match cp_anim::load_animation_meta(&path) {
        Ok(meta) => meta,
        Err(err) => return Err(cp_err_msg(err, &path)),
    };

    Ok(AnimationMeta {
        version: Animation::version(),
        duration: cp_meta.duration as f64,
        num_tracks: cp_meta.num_tracks,
        name: cp_meta.name,
        translations_count: cp_meta.translations_count,
        rotations_count: cp_meta.rotations_count,
        scales_count: cp_meta.scales_count,
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
    let cp_meta = match cp_anim::load_root_motion_meta(&path) {
        Ok(meta) => meta,
        Err(err) => return Err(cp_err_msg(err, &path)),
    };

    Ok(RootMotionMeta {
        version: cp_meta.version,
            position_default: match cp_meta.position_default.enabled {
                true => Some(RootMotionPositionMeta {
                    whole_distance: cp_meta.position_default.whole_distance as f64,
                    whole_distance_xz: cp_meta.position_default.whole_distance_xz as f64,
                    whole_distance_y: cp_meta.position_default.whole_distance_y as f64,
                }),
                false => None,
            },
            position_move: match cp_meta.position_move.enabled {
                true => Some(RootMotionPositionMeta {
                    whole_distance: cp_meta.position_move.whole_distance as f64,
                    whole_distance_xz: cp_meta.position_move.whole_distance_xz as f64,
                    whole_distance_y: cp_meta.position_move.whole_distance_y as f64,
                }),
                false => None,
            },
            position_move_ex: match cp_meta.position_move_ex.enabled {
                true => Some(RootMotionPositionMeta {
                    whole_distance: cp_meta.position_move_ex.whole_distance as f64,
                    whole_distance_xz: cp_meta.position_move_ex.whole_distance_xz as f64,
                    whole_distance_y: cp_meta.position_move_ex.whole_distance_y as f64,
                }),
                false => None,
            },
        has_rotation: cp_meta.has_rotation,
    })
}

#[napi(object)]
pub struct RangePair {
    pub from: f64,
    pub to: f64,
}

#[napi]
pub fn calc_root_motion_distances(path: String, ranges: Vec<RangePair>) -> Result<Vec<f32>> {
    let root_motion = match RootMotion::from_path(&path) {
        Ok(root_motion) => root_motion,
        Err(err) => return Err(cp_err_msg(err, &path)),
    };

    let mut distances = Vec::<f32>::with_capacity(ranges.len());
    for (idx, range) in ranges.iter().enumerate() {
        if range.from.is_nan() || range.to.is_nan() {
            return Err(Error::new(Status::GenericFailure, format!("ranges[{}]", idx)));
        }
        let from = range.from as f32;
        let to = range.to as f32;
        let distance = match root_motion.calc_distance_between(RootTrackName::Default, from, to) {
            Ok(distance) => distance,
            Err(err) => return Err(cp_err_msg(err, &format!("ranges[{}]", idx))),
        };
        distances.push(distance);
    }
    Ok(distances)
}

#[napi(object)]
pub struct WeaponControlMeta {
    pub version: u32,
    pub has_left_weapon: bool,
    pub has_right_weapon: bool,
}

#[napi]
pub fn load_weapon_control_meta(path: String) -> Result<WeaponControlMeta> {
    let cp_meta = match cp_anim::load_weapon_control_meta(&path) {
        Ok(meta) => meta,
        Err(err) => return Err(cp_err_msg(err, &path)),
    };
    Ok(WeaponControlMeta {
        version: cp_meta.version,
        has_left_weapon: cp_meta.has_left_weapon,
        has_right_weapon: cp_meta.has_right_weapon,
    })
}

#[napi(object)]
pub struct HitMotionMeta {
    pub groups: Vec<HitMotionGroupMeta>,
}

#[napi(object)]
pub struct HitMotionGroupMeta {
    pub group: String,
    pub tracks: i32,
}

#[napi]
pub fn load_hit_motion_meta(path: String) -> Result<HitMotionMeta> {
    let cp_meta = match cp_anim::load_hit_motion_meta(&path) {
        Ok(meta) => meta,
        Err(err) => return Err(cp_err_msg(err, &path)),
    };

    Ok(HitMotionMeta {
        groups: cp_meta
        .track_groups
        .into_iter()
        .map(|g| HitMotionGroupMeta {
            group: g.group,
            tracks: g.count as i32,
        })
        .collect()
    })
}

#[napi(object)]
pub struct ShapeKeyMeta {
    pub version: u32,
    pub count: u32,
    pub names: Vec<String>,
}

#[napi]
pub fn load_shape_key_meta(path: String) -> Result<ShapeKeyMeta> {
    let cp_meta = match cp_anim::load_shape_key_meta(&path) {
        Ok(meta) => meta,
        Err(err) => return Err(cp_err_msg(err, &path)),
    };
    Ok(ShapeKeyMeta {
        version: cp_meta.version,
        count: cp_meta.count,
        names: cp_meta.names,
    })
}

#[napi(object)]
pub struct JointWeightsTableMeta {
    pub names: HashMap<String, u32>,
    pub count: u32,
}

#[napi]
pub fn load_joint_weights_table_meta(path: String) -> Result<JointWeightsTableMeta> {
    let cp_meta = match cp_anim::load_joint_weights_table_meta(&path) {
        Ok(meta) => meta,
        Err(err) => return Err(cp_err_msg(err, &path)),
    };
    Ok(JointWeightsTableMeta {
        names: cp_meta.names.into_iter().enumerate().map(|(idx, name)| (name, idx as u32)).collect(),
        count: cp_meta.count,
    })
}
