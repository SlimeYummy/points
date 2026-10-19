use critical_point_macros::csharp_out;
use glam::{Vec3, Vec3Swizzles};
use ozz_animation_rs::{Animation, Archive, Skeleton, Track};

use crate::animation::{HitMotion, JointWeightsTableRc, RootMotion, RootTrackName, ShapeKey, WeaponControl};
use crate::utils::XResult;

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, Default, Clone)]
pub struct SkeletonMeta {
    pub version: u32,
    pub num_joints: u32,
    pub num_soa_joints: u32,
    pub joint_metas: Vec<SkeletonJointMeta>,
}

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SkeletonJointMeta {
    pub index: i16,
    pub parent: i16,
    pub name: String,
}

#[cfg(feature = "debug-print")]
impl Drop for SkeletonMeta {
    fn drop(&mut self) {
        log::debug!("SkeletonMeta::drop()");
    }
}

pub fn load_skeleton_meta(path: &str, with_joints: bool) -> XResult<SkeletonMeta> {
    let mut archive = Archive::from_path(path)?;
    let ozz_meta = Skeleton::read_meta(&mut archive, with_joints)?;

    let mut meta = SkeletonMeta {
        version: ozz_meta.version,
        num_joints: ozz_meta.num_joints,
        num_soa_joints: ozz_meta.num_joints.div_ceil(4),
        joint_metas: Vec::new(),
    };

    if with_joints {
        meta.joint_metas = vec![SkeletonJointMeta::default(); ozz_meta.num_joints as usize];
        for (name, index) in ozz_meta.joint_names {
            meta.joint_metas[index as usize] = SkeletonJointMeta {
                index: index as i16,
                parent: ozz_meta.joint_parents[index as usize],
                name: name.clone(),
            }
        }
    }
    Ok(meta)
}

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, Default, Clone)]
pub struct AnimationMeta {
    pub version: u32,
    pub duration: f32,
    pub num_tracks: u32,
    pub name: String,
    pub translations_count: u32,
    pub rotations_count: u32,
    pub scales_count: u32,
}

#[cfg(feature = "debug-print")]
impl Drop for AnimationMeta {
    fn drop(&mut self) {
        log::debug!("AnimationMeta::drop()");
    }
}

pub fn load_animation_meta(path: &str) -> XResult<AnimationMeta> {
    let mut archive = Archive::from_path(path)?;
    let ozz_meta = Animation::read_meta(&mut archive)?;

    Ok(AnimationMeta {
        version: ozz_meta.version,
        duration: ozz_meta.duration,
        num_tracks: ozz_meta.num_tracks,
        name: ozz_meta.name,
        translations_count: ozz_meta.translations_count,
        rotations_count: ozz_meta.rotations_count,
        scales_count: ozz_meta.scales_count,
    })
}

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, Default, Clone)]
pub struct RootMotionMeta {
    pub version: u32,
    pub position_default: RootMotionPositionMeta,
    pub position_move: RootMotionPositionMeta,
    pub position_move_ex: RootMotionPositionMeta,
    pub has_rotation: bool,
}

#[cfg(feature = "debug-print")]
impl Drop for RootMotionMeta {
    fn drop(&mut self) {
        log::debug!("RootMotionMeta::drop()");
    }
}

#[repr(C)]
#[csharp_out(Value)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct RootMotionPositionMeta {
    pub enabled: bool,
    pub whole_distance: f32,
    pub whole_distance_xz: f32,
    pub whole_distance_y: f32,
}

pub fn load_root_motion_meta(path: &str) -> XResult<RootMotionMeta> {
    let root_motion = RootMotion::from_path(path)?;

    let mut position_default = RootMotionPositionMeta::default();
    if root_motion.has_position(RootTrackName::Default) {
        let whole = root_motion.whole_position(RootTrackName::Default);
        position_default = RootMotionPositionMeta {
            enabled: true,
            whole_distance: whole.length(),
            whole_distance_xz: whole.xz().length(),
            whole_distance_y: whole.y,
        };
    }

    let mut position_move = RootMotionPositionMeta::default();
    if root_motion.has_position(RootTrackName::Move) {
        let whole = root_motion.whole_position(RootTrackName::Move);
        position_move = RootMotionPositionMeta {
            enabled: true,
            whole_distance: whole.length(),
            whole_distance_xz: whole.xz().length(),
            whole_distance_y: whole.y,
        };
    }

    let mut position_move_ex = RootMotionPositionMeta::default();
    if root_motion.has_position(RootTrackName::MoveEx) {
        let whole = root_motion.whole_position(RootTrackName::MoveEx);
        position_move_ex = RootMotionPositionMeta {
            enabled: true,
            whole_distance: whole.length(),
            whole_distance_xz: whole.xz().length(),
            whole_distance_y: whole.y,
        };
    }

    Ok(RootMotionMeta {
        version: Track::<Vec3>::version(),
        position_default,
        position_move,
        position_move_ex,
        has_rotation: root_motion.has_rotation(),
    })
}

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, Default, Clone)]
pub struct WeaponControlMeta {
    pub version: u32,
    pub has_left_weapon: bool,
    pub has_right_weapon: bool,
}

#[cfg(feature = "debug-print")]
impl Drop for WeaponControlMeta {
    fn drop(&mut self) {
        log::debug!("WeaponControlMeta::drop()");
    }
}

pub fn load_weapon_control_meta(path: &str) -> XResult<WeaponControlMeta> {
    let weapon_control = WeaponControl::from_path(path)?;
    Ok(WeaponControlMeta {
        version: Track::<Vec3>::version(),
        has_left_weapon: weapon_control.has_left_weapon(),
        has_right_weapon: weapon_control.has_right_weapon(),
    })
}

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, Default, Clone)]
pub struct HitMotionMeta {
    pub track_groups: Vec<HitTrackGroupMeta>,
}

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct HitTrackGroupMeta {
    pub group: String,
    pub count: u32,
}

pub fn load_hit_motion_meta(path: &str) -> XResult<HitMotionMeta> {
    let hit_motion = HitMotion::from_path(path)?;
    let mut meta = HitMotionMeta::default();

    for bx in hit_motion.joint_boxes() {
        match meta.track_groups.iter_mut().find(|g| g.group == bx.group) {
            Some(group) => group.count += 1,
            None => {
                meta.track_groups.push(HitTrackGroupMeta {
                    group: bx.group.to_string(),
                    count: 1,
                });
            }
        }
    }

    for bx in hit_motion.weapon_boxes() {
        match meta.track_groups.iter_mut().find(|g| g.group == bx.group) {
            Some(group) => group.count += 1,
            None => {
                meta.track_groups.push(HitTrackGroupMeta {
                    group: bx.group.to_string(),
                    count: 1,
                });
            }
        }
    }

    Ok(meta)
}

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, Default, Clone)]
pub struct ShapeKeyMeta {
    pub version: u32,
    pub count: u32,
    pub names: Vec<String>,
}

#[cfg(feature = "debug-print")]
impl Drop for ShapeKeyMeta {
    fn drop(&mut self) {
        log::debug!("ShapeKeyMeta::drop()");
    }
}

pub fn load_shape_key_meta(path: &str) -> XResult<ShapeKeyMeta> {
    let shape_keys = ShapeKey::from_path(path)?;
    Ok(ShapeKeyMeta {
        version: Track::<f32>::version(),
        count: shape_keys.len() as u32,
        names: shape_keys.iter().map(|w| w.name().to_string()).collect(),
    })
}

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct JointWeightsTableMeta {
    pub names: Vec<String>,
    pub count: u32,
}

pub fn load_joint_weights_table_meta(path: &str) -> XResult<JointWeightsTableMeta> {
    let names = JointWeightsTableRc::names_from_path(&path)?;
    Ok(JointWeightsTableMeta {
        count: names.len() as u32,
        names: names,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_ASSET_PATH;

    #[test]
    fn test_skeleton_meta() {
        let meta = load_skeleton_meta(&format!("{}/Girl/Girl.ls-ozz", TEST_ASSET_PATH), true).unwrap();
        assert_eq!(meta.version, Skeleton::version());
        assert_eq!(meta.num_joints, 22);
        assert_eq!(meta.num_soa_joints, 6);
        assert_eq!(meta.joint_metas.len(), 22);
        assert_eq!(meta.joint_metas[0], SkeletonJointMeta {
            index: 0,
            parent: -1,
            name: "Hips".to_string()
        });
    }

    #[test]
    fn test_animation_meta() {
        let meta = load_animation_meta(&format!("{}/Girl/Attack_01A.la-ozz", TEST_ASSET_PATH)).unwrap();
        assert_eq!(meta.version, Animation::version());
        assert_eq!(meta.duration, 2.6666667);
        assert_eq!(meta.num_tracks, 20);
        assert_eq!(meta.name, "Attack_01A");
    }

    #[test]
    fn test_root_motion_meta() {
        let meta = load_root_motion_meta(&format!("{}/Girl/Attack_01A.rm-ozz", TEST_ASSET_PATH)).unwrap();
        assert_eq!(meta.version, Track::<Vec3>::version());
        assert_eq!(meta.position_default.enabled, true);
        assert_eq!(
            meta.position_default.whole_distance,
            meta.position_default.whole_distance_xz
        );
        assert_eq!(meta.position_default.whole_distance_y, 0.0);
        assert_eq!(meta.position_move.enabled, true);
        assert_eq!(meta.position_move.whole_distance, meta.position_move.whole_distance_xz);
        assert_eq!(meta.position_move.whole_distance_y, 0.0);
        assert_eq!(meta.position_move_ex.enabled, true);
        assert_eq!(
            meta.position_move_ex.whole_distance,
            meta.position_move_ex.whole_distance_xz
        );
        assert_eq!(meta.position_move_ex.whole_distance_y, 0.0);
        assert_eq!(meta.has_rotation, false);
    }

    #[test]
    fn test_weapon_control_meta() {
        let meta = load_weapon_control_meta(&format!("{}/Girl/Idle_Axe.wc-ozz", TEST_ASSET_PATH)).unwrap();
        assert_eq!(meta.version, Track::<Vec3>::version());
        assert_eq!(meta.has_left_weapon, false);
        assert_eq!(meta.has_right_weapon, true);
    }

    #[test]
    fn test_hit_motion_meta() {
        let meta = load_hit_motion_meta(&format!("{}/Girl/Attack_Test.hm-json", TEST_ASSET_PATH)).unwrap();
        assert_eq!(meta.track_groups, vec![
            HitTrackGroupMeta {
                group: "Health".to_string(),
                count: 1
            },
            HitTrackGroupMeta {
                group: "Counter".to_string(),
                count: 1
            },
            HitTrackGroupMeta {
                group: "Axe".to_string(),
                count: 2
            },
        ]);
    }

    #[test]
    fn test_shape_key_meta() {
        let meta = load_shape_key_meta(&format!("{}/Slime/RunLoop.sk-ozz", TEST_ASSET_PATH)).unwrap();
        assert_eq!(meta.version, Track::<f32>::version());
        assert_eq!(meta.count, 2);
        assert_eq!(meta.names, vec!["Air".to_string(), "Land".to_string()]);
    }

    #[test]
    fn test_joint_weights_table_meta() {
        let meta = load_joint_weights_table_meta(&format!("{}/Girl/Girl.lw-json", TEST_ASSET_PATH)).unwrap();
        assert_eq!(meta.count, 8);
        assert_eq!(meta.names, vec![
            "All".to_string(),
            "NoArm".to_string(),
            "Arm".to_string(),
            "LeftArm".to_string(),
            "RightArm".to_string(),
            "All".to_string(),
            "NoLeg".to_string(),
            "Leg".to_string()
        ]);
    }
}
