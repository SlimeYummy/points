use glam::{Quat, Vec3A};
use jolt_physics_rs::{self as jolt, JRef, Shape, StaticCompoundShapeSettings, SubShapeSettings};
use rkyv::vec::ArchivedVec;
use smallvec::SmallVec;
use static_assertions::const_assert_eq;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::{fs, mem, slice};

use crate::asset::{AssetIndxedCompoundShape, AssetShape};
use crate::utils::{xerrf, xresf, HitType, Symbol, ThinVec, XResult, xfrom};

//
// Raw
//

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
struct RawHitMotion {
    name: String,
    shapes: Vec<AssetShape>,
    compound_shapes: Vec<AssetIndxedCompoundShape>,
    tracks: Vec<RawHitTrack>,
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
#[serde(tag = "T")]
enum RawHitTrack {
    Joint(RawHitTrackJoint),
    Weapon(RawHitTrackWeapon),
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
struct RawHitTrackJoint {
    shape_index: u32,
    typ: HitType,
    group: Symbol,
    joint: Symbol,
    ratio: f32,
    #[serde(default)]
    joint2: Symbol,
    position_keys: Vec<HitKeyPosition>,
    rotation_keys: Vec<HitKeyRotation>,
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
struct RawHitTrackWeapon {
    shape_index: u32,
    typ: HitType,
    group: Symbol,
    weapon: Symbol,
    position_keys: Vec<HitKeyPosition>,
    rotation_keys: Vec<HitKeyRotation>,
}

//
// HitMotion
//

#[derive(Debug)]
pub struct HitMotion {
    keyframes_buf: Vec<HitKeyInner>,
    pub name: String,
    pub groups: SmallVec<[Symbol; 4]>,
    pub joint_tracks: ThinVec<HitTrackJoint>,
    pub weapon_tracks: ThinVec<HitTrackWeapon>,
}

const TRACK_TYPE_MASK: u16 = 0x8000;
const TRACK_TYPE_JOINT: u16 = 0 << 15;
const TRACK_TYPE_WEAPON: u16 = 1 << 15;

impl HitMotion {
    pub fn from_path<P: AsRef<Path>>(path: P) -> XResult<HitMotion> {
        let path_str = path.as_ref().to_str();
        let is_rkyv = path.as_ref().extension().map_or(false, |ext| ext == "hm-rkyv");
        if is_rkyv {
            let bytes = fs::read(path.as_ref())?;
            HitMotion::from_rkyv_bytes(&bytes, path_str)
        }
        else {
            let bytes = fs::read(path.as_ref())?;
            HitMotion::from_json_bytes(&bytes, path_str)
        }
    }

    pub fn from_json_bytes(bytes: &[u8], path: Option<&str>) -> XResult<HitMotion> {
        let raw: RawHitMotion = serde_json::from_slice(bytes)?;

        let mut jolt_shapes = Vec::with_capacity(raw.shapes.len() + raw.compound_shapes.len());
        for shape in &raw.shapes {
            jolt_shapes.push(shape.create_physics()?);
        }

        let mut buf: Vec<SubShapeSettings> = Vec::with_capacity(8);
        for compound_shape in &raw.compound_shapes {
            for sub_shape in &compound_shape.sub_shapes {
                let jolt_shape = jolt_shapes
                    .get(sub_shape.shape_index as usize)
                    .ok_or_else(|| xerrf!(BadAsset; "file={}, shape_index={}", path.unwrap_or(""), sub_shape.shape_index))?;
                buf.push(SubShapeSettings::new(jolt_shape.clone(), sub_shape.position, sub_shape.rotation));
            }
            if !buf.is_empty() {
                let settings = StaticCompoundShapeSettings::new(&buf);
                let jolt_shape = jolt::create_static_compound_shape(&settings).map_err(xfrom!())?;
                jolt_shapes.push(jolt_shape.into());
                buf.clear();
            }
        }

        let mut buf_count = 0;
        let mut joint_count = 0;
        let mut weapon_count = 0;
        for track in &raw.tracks {
            match track {
                RawHitTrack::Joint(joint) => {
                    joint_count += 1;
                    buf_count += joint.position_keys.len() + joint.rotation_keys.len()
                }
                RawHitTrack::Weapon(weapon) => {
                    weapon_count += 1;
                    buf_count += weapon.position_keys.len() + weapon.rotation_keys.len()
                }
            }
        }

        let mut hit_motion = HitMotion {
            name: raw.name,
            keyframes_buf: Vec::with_capacity(buf_count),
            groups: SmallVec::new(),
            joint_tracks: ThinVec::with_capacity(joint_count),
            weapon_tracks: ThinVec::with_capacity(weapon_count),
        };

        if raw.tracks.len() >= u16::MAX as usize {
            return xresf!(BadAsset; "path={}, too many tracks", path.unwrap_or(""));
        }

        for track in raw.tracks.into_iter() {
            match track {
                RawHitTrack::Joint(joint) => {
                    let shape = jolt_shapes.get(joint.shape_index as usize).ok_or_else(
                        || xerrf!(BadAsset; "path={}, shape_index={}", path.unwrap_or(""), joint.shape_index),
                    )?;
                    let (pos_keys, rot_keys) = Self::copy_key_buffers(
                        &mut hit_motion.keyframes_buf,
                        &joint.position_keys,
                        &joint.rotation_keys,
                    );
                    let hit_id = hit_motion.joint_tracks.len() as u16 | TRACK_TYPE_JOINT;
                    let track = HitTrackJoint::from_raw(joint, hit_id, shape.clone(), pos_keys, rot_keys)?;
                    hit_motion.joint_tracks.push(track);
                }
                RawHitTrack::Weapon(weapon) => {
                    let shape = jolt_shapes.get(weapon.shape_index as usize).ok_or_else(
                        || xerrf!(BadAsset; "path={}, shape_index={}", path.unwrap_or(""), weapon.shape_index),
                    )?;
                    let (pos_keys, rot_keys) = Self::copy_key_buffers(
                        &mut hit_motion.keyframes_buf,
                        &weapon.position_keys,
                        &weapon.rotation_keys,
                    );
                    let hit_id = hit_motion.weapon_tracks.len() as u16 | TRACK_TYPE_WEAPON;
                    let track = HitTrackWeapon::from_raw(weapon, hit_id, shape.clone(), pos_keys, rot_keys)?;
                    hit_motion.weapon_tracks.push(track);
                }
            }
        }

        assert_eq!(hit_motion.keyframes_buf.len(), buf_count);
        assert_eq!(hit_motion.joint_tracks.len(), joint_count);
        assert_eq!(hit_motion.weapon_tracks.len(), weapon_count);
        
        hit_motion.init_groups();
        Ok(hit_motion)
    }

    fn copy_key_buffers<'t>(
        keyframes_buf: &'t mut Vec<HitKeyInner>,
        pos_keys: &[HitKeyPosition],
        rot_keys: &[HitKeyRotation],
    ) -> (&'t [HitKeyPosition], &'t [HitKeyRotation]) {
        let pos_start = keyframes_buf.len();
        let pos_len = pos_keys.len();
        let pos_keys_vec4: &[HitKeyInner] = unsafe { mem::transmute_copy(&pos_keys) };
        keyframes_buf.extend_from_slice(pos_keys_vec4);

        let rot_start = keyframes_buf.len();
        let rot_len = rot_keys.len();
        let rot_keys_vec4: &[HitKeyInner] = unsafe { mem::transmute_copy(&rot_keys) };
        keyframes_buf.extend_from_slice(rot_keys_vec4);

        let pos_keys_buf = unsafe { mem::transmute_copy(&&keyframes_buf[pos_start..pos_start + pos_len]) };
        let rot_keys_buf = unsafe { mem::transmute_copy(&&keyframes_buf[rot_start..rot_start + rot_len]) };
        (pos_keys_buf, rot_keys_buf)
    }

    pub(crate) fn from_rkyv_bytes(bytes: &[u8], path: Option<&str>) -> XResult<HitMotion> {
        let raw = unsafe { rkyv::access_unchecked::<ArchivedRawHitMotion>(bytes) };

        let mut jolt_shapes = Vec::with_capacity(raw.shapes.len());
        for archived_shape in raw.shapes.iter() {
            let shape = match rkyv::deserialize::<AssetShape, rkyv::rancor::Error>(archived_shape) {
                Ok(shape) => shape,
                Err(err) => return Err(xerrf!(BadAsset; "path={}, err={}", path.unwrap_or(""), err)),
            };
            jolt_shapes.push(shape.create_physics()?);
        }
        
        let mut buf: Vec<SubShapeSettings> = Vec::with_capacity(8);
        for compound_shape in raw.compound_shapes.iter() {
            for sub_shape in compound_shape.sub_shapes.iter() {
                let shape_index: u32 = sub_shape.shape_index.into();
                let jolt_shape = jolt_shapes
                    .get(shape_index as usize)
                    .ok_or_else(|| xerrf!(BadAsset; "file={}, shape_index={}", path.unwrap_or(""), sub_shape.shape_index))?;
                buf.push(SubShapeSettings::new(jolt_shape.clone(), sub_shape.position, sub_shape.rotation));
            }
            if !buf.is_empty() {
                let settings = StaticCompoundShapeSettings::new(&buf);
                let jolt_shape = jolt::create_static_compound_shape(&settings).map_err(xfrom!())?;
                jolt_shapes.push(jolt_shape.into());
                buf.clear();
            }
        }

        let mut buf_count = 0;
        let mut joint_count = 0;
        let mut weapon_count = 0;
        for track in raw.tracks.iter() {
            match track {
                ArchivedRawHitTrack::Joint(joint) => {
                    joint_count += 1;
                    buf_count += joint.position_keys.len() + joint.rotation_keys.len()
                }
                ArchivedRawHitTrack::Weapon(weapon) => {
                    weapon_count += 1;
                    buf_count += weapon.position_keys.len() + weapon.rotation_keys.len()
                }
            }
        }

        let mut hit_motion = HitMotion {
            name: raw.name.to_string(),
            keyframes_buf: Vec::with_capacity(buf_count),
            groups: SmallVec::new(),
            joint_tracks: ThinVec::with_capacity(joint_count),
            weapon_tracks: ThinVec::with_capacity(weapon_count),
        };

        if raw.tracks.len() >= u16::MAX as usize {
            return xresf!(BadAsset; "path={}, too many tracks", path.unwrap_or(""));
        }

        for track in raw.tracks.into_iter() {
            match track {
                ArchivedRawHitTrack::Joint(joint) => {
                    let shape_index: u32 = joint.shape_index.into();
                    let shape = jolt_shapes.get(shape_index as usize).ok_or_else(
                        || xerrf!(BadAsset; "path={}, shape_index={}", path.unwrap_or(""), joint.shape_index),
                    )?;
                    let (pos_keys, rot_keys) = Self::copy_archived_key_buffers(
                        &mut hit_motion.keyframes_buf,
                        &joint.position_keys,
                        &joint.rotation_keys,
                    );
                    let hit_id = hit_motion.joint_tracks.len() as u16 | TRACK_TYPE_JOINT;
                    let track = HitTrackJoint::from_archived(joint, hit_id, shape.clone(), pos_keys, rot_keys)?;
                    hit_motion.joint_tracks.push(track);
                }
                ArchivedRawHitTrack::Weapon(weapon) => {
                    let shape_index: u32 = weapon.shape_index.into();
                    let shape = jolt_shapes.get(shape_index as usize).ok_or_else(
                        || xerrf!(BadAsset; "path={}, shape_index={}", path.unwrap_or(""), weapon.shape_index),
                    )?;
                    let (pos_keys, rot_keys) = Self::copy_archived_key_buffers(
                        &mut hit_motion.keyframes_buf,
                        &weapon.position_keys,
                        &weapon.rotation_keys,
                    );
                    let hit_id = hit_motion.weapon_tracks.len() as u16 | TRACK_TYPE_WEAPON;
                    let track = HitTrackWeapon::from_archived(weapon, hit_id, shape.clone(), pos_keys, rot_keys)?;
                    hit_motion.weapon_tracks.push(track);
                }
            }
        }

        assert_eq!(hit_motion.keyframes_buf.len(), buf_count);
        assert_eq!(hit_motion.joint_tracks.len(), joint_count);
        assert_eq!(hit_motion.weapon_tracks.len(), weapon_count);

        hit_motion.init_groups();
        Ok(hit_motion)
    }

    fn copy_archived_key_buffers<'t>(
        keyframes_buf: &'t mut Vec<HitKeyInner>,
        pos_keys: &ArchivedVec<ArchivedHitKeyPosition>,
        rot_keys: &ArchivedVec<ArchivedHitKeyRotation>,
    ) -> (&'t [HitKeyPosition], &'t [HitKeyRotation]) {
        let pos_start = keyframes_buf.len();
        let pos_len = pos_keys.len();
        for key in pos_keys.iter() {
            keyframes_buf.push(HitKeyInner {
                time: key.time.into(),
                value: [key.value[0].into(), key.value[1].into(), key.value[2].into(), 0.0],
            });
        }

        let rot_start = keyframes_buf.len();
        let rot_len = rot_keys.len();
        for key in rot_keys.iter() {
            keyframes_buf.push(HitKeyInner {
                time: key.time.into(),
                value: [
                    key.value[0].into(),
                    key.value[1].into(),
                    key.value[2].into(),
                    key.value[3].into(),
                ],
            });
        }

        let pos_keys_buf = unsafe { mem::transmute_copy(&&keyframes_buf[pos_start..pos_start + pos_len]) };
        let rot_keys_buf = unsafe { mem::transmute_copy(&&keyframes_buf[rot_start..rot_start + rot_len]) };
        (pos_keys_buf, rot_keys_buf)
    }

    fn init_groups(&mut self) {
        for track in self.joint_tracks.iter_mut() {
            let idx = self.groups.iter().position(|&g| g == track.group);
            if let Some(idx) = idx {
                track.group_index = idx as u16;
            } else {
                track.group_index = self.groups.len() as u16;
                self.groups.push(track.group.clone())
            }
        }

        for track in self.weapon_tracks.iter_mut() {
            let idx = self.groups.iter().position(|&g| g == track.group);
            if let Some(idx) = idx {
                track.group_index = idx as u16;
            } else {
                track.group_index = self.groups.len() as u16;
                self.groups.push(track.group.clone())
            }
        }
    }

    pub fn get_track(&self, hit_id: u16) -> Option<&HitTrackBase> {
        let idx = (hit_id & !TRACK_TYPE_MASK) as usize;
        if hit_id & TRACK_TYPE_MASK == TRACK_TYPE_JOINT {
            self.joint_tracks.get(idx).map(|t| &t._base)
        } else {
            self.weapon_tracks.get(idx).map(|t| &t._base)
        }
    }

    pub fn get_joint_track(&self, hit_id: u16) -> Option<&HitTrackJoint> {
        if hit_id & TRACK_TYPE_MASK != TRACK_TYPE_JOINT {
            return None;
        }
        self.joint_tracks.get((hit_id & !TRACK_TYPE_MASK) as usize)
    }

    pub fn get_weapon_track(&self, hit_id: u16) -> Option<&HitTrackWeapon> {
        if hit_id & TRACK_TYPE_MASK != TRACK_TYPE_WEAPON {
            return None;
        }
        self.weapon_tracks.get((hit_id & !TRACK_TYPE_MASK) as usize)
    }
}
