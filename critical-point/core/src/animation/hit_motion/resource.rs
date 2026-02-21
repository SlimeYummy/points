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

#[derive(Debug)]
pub struct HitTrackBase {
    pub hit_id: u16,
    pub shape: JRef<Shape>,
    pub typ: HitType,
    pub group: Symbol,
    pub group_index: u16,
    pub start_time: f32,
    pub finish_time: f32,
    pos_keys_ptr: *const HitKeyPosition,
    pos_keys_len: u32,
    rot_keys_ptr: *const HitKeyRotation,
    rot_keys_len: u32,
}

impl HitTrackBase {
    fn new(
        hit_id: u16,
        shape: JRef<Shape>,
        typ: HitType,
        group: Symbol,
        positions_keys: &[HitKeyPosition],
        rotations_keys: &[HitKeyRotation],
    ) -> XResult<HitTrackBase> {
        if positions_keys.len() < 2 {
            return xresf!(BadAsset; "Positions keys size must >= 2");
        }
        if rotations_keys.len() < 2 {
            return xresf!(BadAsset; "Rotations keys size must >= 2");
        }

        let pos_start = positions_keys.first().unwrap().time;
        let rot_start = rotations_keys.first().unwrap().time;
        if pos_start != rot_start {
            return xresf!(BadAsset; "Start time mismatch");
        }

        let pos_finish = positions_keys.last().unwrap().time;
        let rot_finish = rotations_keys.last().unwrap().time;
        if pos_finish != rot_finish {
            return xresf!(BadAsset; "Finish time mismatch");
        }

        Ok(HitTrackBase {
            hit_id,
            shape,
            typ,
            group,
            group_index: 0,
            start_time: pos_start,
            finish_time: pos_finish,
            pos_keys_ptr: positions_keys.as_ptr(),
            pos_keys_len: positions_keys.len() as u32,
            rot_keys_ptr: rotations_keys.as_ptr(),
            rot_keys_len: rotations_keys.len() as u32,
        })
    }

    #[inline]
    pub fn positions_keys(&self) -> &[HitKeyPosition] {
        unsafe { slice::from_raw_parts(self.pos_keys_ptr, self.pos_keys_len as usize) }
    }

    #[inline]
    pub fn rotations_keys(&self) -> &[HitKeyRotation] {
        unsafe { slice::from_raw_parts(self.rot_keys_ptr, self.rot_keys_len as usize) }
    }
}

#[derive(Debug)]
pub struct HitTrackJoint {
    pub _base: HitTrackBase,
    pub joint: Symbol,
    pub ratio: f32,
    pub joint2: Symbol,
}

impl Deref for HitTrackJoint {
    type Target = HitTrackBase;
    fn deref(&self) -> &Self::Target {
        &self._base
    }
}

impl DerefMut for HitTrackJoint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._base
    }
}

impl HitTrackJoint {
    #[inline]
    fn from_raw(
        raw: RawHitTrackJoint,
        hit_id: u16,
        shape: JRef<Shape>,
        pos_keys: &[HitKeyPosition],
        rot_keys: &[HitKeyRotation],
    ) -> XResult<HitTrackJoint> {
        Ok(HitTrackJoint {
            _base: HitTrackBase::new(
                hit_id,
                shape,
                raw.typ,
                raw.group,
                pos_keys,
                rot_keys,
            )?,
            joint: raw.joint,
            ratio: raw.ratio,
            joint2: raw.joint2,
        })
    }

    #[inline]
    fn from_archived(
        raw: &ArchivedRawHitTrackJoint,
        hit_id: u16,
        shape: JRef<Shape>,
        pos_keys: &[HitKeyPosition],
        rot_keys: &[HitKeyRotation],
    ) -> XResult<HitTrackJoint> {
        Ok(HitTrackJoint{
            _base: HitTrackBase::new(
                hit_id,
                shape,
                raw.typ,
                Symbol::new(raw.group.as_str())?,
                pos_keys,
                rot_keys
            )?,
            joint: Symbol::new(raw.joint.as_str())?,
            ratio: raw.ratio.into(),
            joint2: Symbol::new(raw.joint2.as_str())?,
        })
    }
}

#[derive(Debug)]
pub struct HitTrackWeapon {
    pub _base: HitTrackBase,
    pub weapon: Symbol,
}

impl Deref for HitTrackWeapon {
    type Target = HitTrackBase;
    fn deref(&self) -> &Self::Target {
        &self._base
    }
}

impl DerefMut for HitTrackWeapon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._base
    }
}

impl HitTrackWeapon {
    #[inline]
    fn from_raw(
        raw: RawHitTrackWeapon,
        hit_id: u16,
        shape: JRef<Shape>,
        pos_keys: &[HitKeyPosition],
        rot_keys: &[HitKeyRotation],
    ) -> XResult<HitTrackWeapon> {
        Ok(HitTrackWeapon {
            _base: 
            HitTrackBase::new(
                hit_id,
                shape,
                raw.typ,
                raw.group,
                pos_keys,
                rot_keys,
            )?,
            weapon: raw.weapon
        })
    }

    #[inline]
    fn from_archived(
        raw: &ArchivedRawHitTrackWeapon,
        hit_id: u16,
        shape: JRef<Shape>,
        pos_keys: &[HitKeyPosition],
        rot_keys: &[HitKeyRotation],
    ) -> XResult<HitTrackWeapon> {
        Ok(HitTrackWeapon {
            _base: HitTrackBase::new(
                hit_id,
                shape,
                raw.typ,
                Symbol::new(raw.group.as_str())?,
                pos_keys,
                rot_keys,
            )?,
            weapon: Symbol::new(raw.weapon.as_str())?,
        })
    }
}

#[repr(C)]
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct HitKeyPosition {
    pub time: f32,
    value: [f32; 3],
    #[serde(skip)]
    _padding: f32,
}

impl HitKeyPosition {
    #[allow(dead_code)]
    #[inline(always)]
    pub fn new(time: f32, value: Vec3A) -> HitKeyPosition {
        HitKeyPosition {
            time,
            value: value.to_array(),
            _padding: 0.0,
        }
    }

    #[inline(always)]
    pub fn value(&self) -> Vec3A {
        Vec3A::from_array(self.value)
    }
}

#[repr(C)]
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct HitKeyRotation {
    pub time: f32,
    value: [f32; 4],
}

impl HitKeyRotation {
    #[allow(dead_code)]
    #[inline(always)]
    pub fn new(time: f32, value: Quat) -> HitKeyRotation {
        HitKeyRotation {
            time,
            value: value.to_array(),
        }
    }

    #[inline(always)]
    pub fn value(&self) -> Quat {
        Quat::from_array(self.value)
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct HitKeyInner {
    time: f32,
    value: [f32; 4],
}

const_assert_eq!(mem::size_of::<HitKeyPosition>(), mem::size_of::<HitKeyInner>());
const_assert_eq!(mem::size_of::<HitKeyRotation>(), mem::size_of::<HitKeyInner>());

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_ASSET_PATH;
    use crate::utils::sb;
    use std::fs::File;
    use std::io::Read;

    fn check_hit_motion(hit_motion: &HitMotion) {
        assert_eq!(hit_motion.keyframes_buf.len(), 16);

        assert_eq!(hit_motion.joint_tracks.len(), 2);

        assert_eq!(hit_motion.joint_tracks[0].hit_id, 0);
        assert!(hit_motion.joint_tracks[0].shape.count_ref() > 0);
        assert_eq!(hit_motion.joint_tracks[0].typ, HitType::Health);
        assert_eq!(hit_motion.joint_tracks[0].group, sb!("Health"));
        assert_eq!(hit_motion.joint_tracks[0].joint, sb!("Spine"));
        assert_eq!(hit_motion.joint_tracks[0].ratio, 0.5);
        assert_eq!(hit_motion.joint_tracks[0].joint2, sb!(""));
        assert_eq!(hit_motion.joint_tracks[0].positions_keys(), &[
            HitKeyPosition::new(0.5, Vec3A::new(0.0, 0.0, 0.0)),
            HitKeyPosition::new(0.8333333, Vec3A::new(0.0, 0.0, 0.0)),
        ]);
        assert_eq!(hit_motion.joint_tracks[0].rotations_keys(), &[
            HitKeyRotation::new(0.5, Quat::from_xyzw(0.0, 0.0, 0.0, 1.0)),
            HitKeyRotation::new(0.8333333, Quat::from_xyzw(0.0, 0.0, 0.0, 1.0)),
        ]);

        assert_eq!(hit_motion.joint_tracks[1].hit_id, 1);
        assert!(hit_motion.joint_tracks[1].shape.count_ref() > 0);
        assert_eq!(hit_motion.joint_tracks[1].typ, HitType::Counter);
        assert_eq!(hit_motion.joint_tracks[1].group, sb!("Counter"));
        assert_eq!(hit_motion.joint_tracks[1].joint, sb!("LeftHand"));
        assert_eq!(hit_motion.joint_tracks[1].ratio, 0.2);
        assert_eq!(hit_motion.joint_tracks[1].joint2, sb!("LeftLowerArm"));
        assert_eq!(hit_motion.joint_tracks[1].positions_keys(), &[
            HitKeyPosition::new(1.0, Vec3A::new(0.1, 0.1, 0.0)),
            HitKeyPosition::new(1.13333333, Vec3A::new(0.1, 0.1, 0.0)),
        ]);
        assert_eq!(hit_motion.joint_tracks[1].rotations_keys(), &[
            HitKeyRotation::new(1.0, Quat::from_xyzw(1.0, 0.0, 0.0, 0.0)),
            HitKeyRotation::new(1.13333333, Quat::from_xyzw(1.0, 0.0, 0.0, 0.0)),
        ]);

        assert_eq!(hit_motion.weapon_tracks.len(), 2);

        assert_eq!(hit_motion.weapon_tracks[0].hit_id, 2);
        assert!(hit_motion.weapon_tracks[0].shape.count_ref() > 0);
        assert_eq!(hit_motion.weapon_tracks[0].typ, HitType::Attack);
        assert_eq!(hit_motion.weapon_tracks[0].group, sb!("Axe"));
        assert_eq!(hit_motion.weapon_tracks[0].weapon, sb!("Axe"));
        assert_eq!(hit_motion.weapon_tracks[0].positions_keys(), &[
            HitKeyPosition::new(1.05, Vec3A::new(0.0, -0.15, -0.7)),
            HitKeyPosition::new(1.2, Vec3A::new(0.0, -0.15, -0.7)),
        ]);
        assert_eq!(hit_motion.weapon_tracks[0].rotations_keys(), &[
            HitKeyRotation::new(1.05, Quat::from_xyzw(0.7071068, 0.0, 0.0, 0.7071068)),
            HitKeyRotation::new(1.2, Quat::from_xyzw(0.7071068, 0.0, 0.0, 0.7071068)),
        ]);

        assert_eq!(hit_motion.weapon_tracks[1].hit_id, 3);
        assert!(hit_motion.weapon_tracks[1].shape.count_ref() > 0);
        assert_eq!(hit_motion.weapon_tracks[1].typ, HitType::Attack);
        assert_eq!(hit_motion.weapon_tracks[1].group, sb!("Axe"));
        assert_eq!(hit_motion.weapon_tracks[1].weapon, sb!("Axe"));
        assert_eq!(hit_motion.weapon_tracks[1].positions_keys(), &[
            HitKeyPosition::new(1.2166667, Vec3A::new(0.0, -0.15, -0.7)),
            HitKeyPosition::new(1.36666667, Vec3A::new(0.0, -0.15, -0.7)),
        ]);
        assert_eq!(hit_motion.weapon_tracks[1].rotations_keys(), &[
            HitKeyRotation::new(1.2166667, Quat::from_xyzw(0.7071068, 0.0, 0.0, 0.7071068)),
            HitKeyRotation::new(1.36666667, Quat::from_xyzw(0.7071068, 0.0, 0.0, 0.7071068)),
        ]);
    }

    #[test]
    fn test_hit_motion_from_json_reader() {
        let json_path = format!("{}/TestDemo.hm-json", TEST_ASSET_PATH);
        let mut json_file = File::open(&json_path).unwrap();
        let mut json_buf = Vec::new();
        json_file.read_to_end(&mut json_buf).unwrap();
        let hit_motion = HitMotion::from_json_bytes(&json_buf, Some(&json_path)).unwrap();
        check_hit_motion(&hit_motion);
    }
}
