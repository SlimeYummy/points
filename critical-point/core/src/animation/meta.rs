use critical_point_csgen::CsOut;
use glam::{Vec3, Vec3Swizzles};
use ozz_animation_rs::{Animation, Archive, Skeleton, Track};

use crate::animation::{HitMotion, RootMotion, RootTrackName, WeaponMotion};
use crate::utils::XResult;

#[repr(C)]
#[derive(Debug, Default, Clone, CsOut)]
#[cs_attr(Ref)]
pub struct SkeletonMeta {
    pub version: u32,
    pub num_joints: u32,
    pub num_soa_joints: u32,
    pub joint_metas: Vec<SkeletonJointMeta>,
}

#[repr(C)]
#[derive(Debug, Default, Clone, PartialEq, CsOut)]
#[cs_attr(Ref)]
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

pub fn load_skeleton_meta(path: String, with_joints: bool) -> XResult<SkeletonMeta> {
    let mut archive = Archive::from_path(&path)?;
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
#[derive(Debug, Default, Clone, CsOut)]
#[cs_attr(Ref)]
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

pub fn load_animation_meta(path: String) -> XResult<AnimationMeta> {
    let mut archive = Archive::from_path(&path)?;
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
#[derive(Debug, Default, Clone, CsOut)]
#[cs_attr(Ref)]
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
#[derive(Debug, Default, PartialEq, Clone, CsOut)]
#[cs_attr(Value)]
pub struct RootMotionPositionMeta {
    pub enabled: bool,
    pub whole_distance: f32,
    pub whole_distance_xz: f32,
    pub whole_distance_y: f32,
}

pub fn load_root_motion_meta(path: String) -> XResult<RootMotionMeta> {
    let root_motion = RootMotion::from_path(&path)?;

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
#[derive(Debug, Default, Clone, CsOut)]
#[cs_attr(Ref)]
pub struct WeaponMotionMeta {
    pub version: u32,
    pub count: u32,
    pub names: Vec<String>,
}

#[cfg(feature = "debug-print")]
impl Drop for WeaponMotionMeta {
    fn drop(&mut self) {
        log::debug!("WeaponMotionMeta::drop()");
    }
}

pub fn load_weapon_motion_meta(path: String, with_names: bool) -> XResult<WeaponMotionMeta> {
    let weapon_motion = WeaponMotion::from_path(&path)?;

    let mut meta = WeaponMotionMeta {
        version: Track::<Vec3>::version(),
        count: weapon_motion.len() as u32,
        names: Vec::new(),
    };

    if with_names {
        meta.names = weapon_motion.iter().map(|w| w.name().to_string()).collect();
    }
    Ok(meta)
}

#[repr(C)]
#[derive(Debug, Default, Clone, CsOut)]
#[cs_attr(Ref)]
pub struct HitMotionMeta {
    pub track_groups: Vec<HitTrackGroupMeta>,
}

#[repr(C)]
#[derive(Debug, Default, Clone, PartialEq, CsOut)]
#[cs_attr(Ref)]
pub struct HitTrackGroupMeta {
    pub group: String,
    pub count: u32,
}

pub fn load_hit_motion_meta(path: String) -> XResult<HitMotionMeta> {
    let hit_motion = HitMotion::from_path(&path)?;
    let mut meta = HitMotionMeta::default();

    for track in hit_motion.joint_tracks {
        match meta.track_groups.iter_mut().find(|g| g.group == track.group) {
            Some(group) => group.count += 1,
            None => {
                meta.track_groups.push(HitTrackGroupMeta {
                    group: track.group.to_string(),
                    count: 1,
                });
            }
        }
    }

    for track in hit_motion.weapon_tracks {
        match meta.track_groups.iter_mut().find(|g| g.group == track.group) {
            Some(group) => group.count += 1,
            None => {
                meta.track_groups.push(HitTrackGroupMeta {
                    group: track.group.to_string(),
                    count: 1,
                });
            }
        }
    }

    Ok(meta)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_ASSET_PATH;

    #[test]
    fn test_skeleton_meta() {
        let meta = load_skeleton_meta(format!("{}/Girl.ls-ozz", TEST_ASSET_PATH), true).unwrap();
        assert_eq!(meta.version, Skeleton::version());
        assert_eq!(meta.num_joints, 20);
        assert_eq!(meta.num_soa_joints, 5);
        assert_eq!(meta.joint_metas.len(), 20);
        assert_eq!(meta.joint_metas[0], SkeletonJointMeta {
            index: 0,
            parent: -1,
            name: "Hips".to_string()
        });
    }

    #[test]
    fn test_animation_meta() {
        let meta = load_animation_meta(format!("{}/Girl_Attack_01A.la-ozz", TEST_ASSET_PATH)).unwrap();
        assert_eq!(meta.version, Animation::version());
        assert_eq!(meta.duration, 2.8);
        assert_eq!(meta.num_tracks, 20);
        assert_eq!(meta.name, "Attack_01A");
    }

    #[test]
    fn test_root_motion_meta() {
        let meta = load_root_motion_meta(format!("{}/Girl_Attack_01A.rm-ozz", TEST_ASSET_PATH)).unwrap();
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
    fn test_weapon_motion_meta() {
        let meta = load_weapon_motion_meta(format!("{}/Girl_Attack_01A.wm-ozz", TEST_ASSET_PATH), true).unwrap();
        assert_eq!(meta.version, Track::<Vec3>::version());
        assert_eq!(meta.count, 1);
        assert_eq!(meta.names, vec!["Axe".to_string()]);
    }

    #[test]
    fn test_hit_motion_meta() {
        let meta = load_hit_motion_meta(format!("{}/TestDemo.hm-json", TEST_ASSET_PATH)).unwrap();
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
}
