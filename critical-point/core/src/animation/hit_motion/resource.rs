use glam::{Quat, Vec3A};
use jolt_physics_rs::{self as jolt, JRef, Shape, StaticCompoundShapeSettings, SubShapeSettings};
use rkyv::vec::ArchivedVec;
use smallvec::SmallVec;
use static_assertions::const_assert_eq;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::{fs, mem, slice};

use crate::asset::{AssetIndxedCompoundShape, AssetShape};
use crate::utils::{loose_ge, loose_le, xerrf, xfrom, xresf, HitType, Symbol, XResult};

//
// Raw
//

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
struct RawHitMotion {
    name: String,
    shapes: Vec<AssetShape>,
    #[serde(default)]
    compound_shapes: Vec<AssetIndxedCompoundShape>,
    boxes: Vec<RawHitBox>,
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
#[serde(tag = "T")]
enum RawHitBox {
    Joint(RawHitBoxJoint),
    Weapon(RawHitBoxWeapon),
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
struct RawHitBoxJoint {
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
struct RawHitBoxWeapon {
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
    name: String,
    groups: SmallVec<[HitGroup; 3]>,
    joint_boxes: Vec<HitBoxJoint>,
    joint_offset: u16,
    weapon_boxes: Vec<HitBoxWeapon>,
    weapon_offset: u16,
    boxes_ptrs: Vec<*const HitBoxBase>,
}

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
                let jolt_shape = jolt_shapes.get(sub_shape.shape_index as usize).ok_or_else(
                    || xerrf!(BadAsset; "file={}, shape_index={}", path.unwrap_or(""), sub_shape.shape_index),
                )?;
                buf.push(SubShapeSettings::new(
                    jolt_shape.clone(),
                    sub_shape.position,
                    sub_shape.rotation,
                ));
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
        for bx in &raw.boxes {
            match bx {
                RawHitBox::Joint(joint) => {
                    joint_count += 1;
                    buf_count += joint.position_keys.len() + joint.rotation_keys.len()
                }
                RawHitBox::Weapon(weapon) => {
                    weapon_count += 1;
                    buf_count += weapon.position_keys.len() + weapon.rotation_keys.len()
                }
            }
        }

        let mut hit_motion = HitMotion {
            name: raw.name,
            keyframes_buf: Vec::with_capacity(buf_count),
            groups: SmallVec::new(),
            joint_boxes: Vec::with_capacity(joint_count),
            joint_offset: 0,
            weapon_boxes: Vec::with_capacity(weapon_count),
            weapon_offset: joint_count as u16,
            boxes_ptrs: Vec::with_capacity(joint_count + weapon_count),
        };

        if raw.boxes.len() >= u16::MAX as usize {
            return xresf!(BadAsset; "path={}, too many boxes", path.unwrap_or(""));
        }

        for bx in raw.boxes.into_iter() {
            match bx {
                RawHitBox::Joint(joint) => {
                    let shape = jolt_shapes.get(joint.shape_index as usize).ok_or_else(
                        || xerrf!(BadAsset; "path={}, shape_index={}", path.unwrap_or(""), joint.shape_index),
                    )?;
                    let (pos_keys, rot_keys) = Self::copy_key_buffers(
                        &mut hit_motion.keyframes_buf,
                        &joint.position_keys,
                        &joint.rotation_keys,
                    );
                    let box_index = hit_motion.joint_offset + hit_motion.joint_boxes.len() as u16;
                    let bx = HitBoxJoint::from_raw(joint, box_index, shape.clone(), pos_keys, rot_keys)?;
                    hit_motion.joint_boxes.push(bx);
                }
                RawHitBox::Weapon(weapon) => {
                    let shape = jolt_shapes.get(weapon.shape_index as usize).ok_or_else(
                        || xerrf!(BadAsset; "path={}, shape_index={}", path.unwrap_or(""), weapon.shape_index),
                    )?;
                    let (pos_keys, rot_keys) = Self::copy_key_buffers(
                        &mut hit_motion.keyframes_buf,
                        &weapon.position_keys,
                        &weapon.rotation_keys,
                    );
                    let box_index = hit_motion.weapon_offset + hit_motion.weapon_boxes.len() as u16;
                    let bx = HitBoxWeapon::from_raw(weapon, box_index, shape.clone(), pos_keys, rot_keys)?;
                    hit_motion.weapon_boxes.push(bx);
                }
            }
        }

        assert_eq!(hit_motion.keyframes_buf.len(), buf_count);
        assert_eq!(hit_motion.joint_boxes.len(), joint_count);
        assert_eq!(hit_motion.weapon_boxes.len(), weapon_count);

        hit_motion.init_groups();
        hit_motion.init_box_ptrs();
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
                let jolt_shape = jolt_shapes.get(shape_index as usize).ok_or_else(
                    || xerrf!(BadAsset; "file={}, shape_index={}", path.unwrap_or(""), sub_shape.shape_index),
                )?;
                buf.push(SubShapeSettings::new(
                    jolt_shape.clone(),
                    sub_shape.position,
                    sub_shape.rotation,
                ));
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
        for bx in raw.boxes.iter() {
            match bx {
                ArchivedRawHitBox::Joint(joint) => {
                    joint_count += 1;
                    buf_count += joint.position_keys.len() + joint.rotation_keys.len()
                }
                ArchivedRawHitBox::Weapon(weapon) => {
                    weapon_count += 1;
                    buf_count += weapon.position_keys.len() + weapon.rotation_keys.len()
                }
            }
        }

        let mut hit_motion = HitMotion {
            name: raw.name.to_string(),
            keyframes_buf: Vec::with_capacity(buf_count),
            groups: SmallVec::new(),
            joint_boxes: Vec::with_capacity(joint_count),
            joint_offset: 0,
            weapon_boxes: Vec::with_capacity(weapon_count),
            weapon_offset: joint_count as u16,
            boxes_ptrs: Vec::with_capacity(joint_count + weapon_count),
        };

        if raw.boxes.len() >= u16::MAX as usize {
            return xresf!(BadAsset; "path={}, too many boxes", path.unwrap_or(""));
        }

        for bx in raw.boxes.into_iter() {
            match bx {
                ArchivedRawHitBox::Joint(joint) => {
                    let shape_index: u32 = joint.shape_index.into();
                    let shape = jolt_shapes.get(shape_index as usize).ok_or_else(
                        || xerrf!(BadAsset; "path={}, shape_index={}", path.unwrap_or(""), joint.shape_index),
                    )?;
                    let (pos_keys, rot_keys) = Self::copy_archived_key_buffers(
                        &mut hit_motion.keyframes_buf,
                        &joint.position_keys,
                        &joint.rotation_keys,
                    );
                    let box_index = hit_motion.joint_offset + hit_motion.joint_boxes.len() as u16;
                    let bx = HitBoxJoint::from_archived(joint, box_index, shape.clone(), pos_keys, rot_keys)?;
                    hit_motion.joint_boxes.push(bx);
                }
                ArchivedRawHitBox::Weapon(weapon) => {
                    let shape_index: u32 = weapon.shape_index.into();
                    let shape = jolt_shapes.get(shape_index as usize).ok_or_else(
                        || xerrf!(BadAsset; "path={}, shape_index={}", path.unwrap_or(""), weapon.shape_index),
                    )?;
                    let (pos_keys, rot_keys) = Self::copy_archived_key_buffers(
                        &mut hit_motion.keyframes_buf,
                        &weapon.position_keys,
                        &weapon.rotation_keys,
                    );
                    let box_index = hit_motion.weapon_offset + hit_motion.weapon_boxes.len() as u16;
                    let bx = HitBoxWeapon::from_archived(weapon, box_index, shape.clone(), pos_keys, rot_keys)?;
                    hit_motion.weapon_boxes.push(bx);
                }
            }
        }

        assert_eq!(hit_motion.keyframes_buf.len(), buf_count);
        assert_eq!(hit_motion.joint_boxes.len(), joint_count);
        assert_eq!(hit_motion.weapon_boxes.len(), weapon_count);

        hit_motion.init_groups();
        hit_motion.init_box_ptrs();
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
        for bx in self.joint_boxes.iter_mut() {
            let idx = self.groups.iter().position(|g| g.name == bx.group);
            if let Some(idx) = idx {
                bx.group_index = idx as u16;
                self.groups[idx].start_time = f32::min(self.groups[idx].start_time, bx.start_time);
                self.groups[idx].finish_time = f32::max(self.groups[idx].finish_time, bx.finish_time);
            }
            else {
                bx.group_index = self.groups.len() as u16;
                self.groups.push(HitGroup::new(bx.group, bx.start_time, bx.finish_time));
            }
        }

        for bx in self.weapon_boxes.iter_mut() {
            let idx = self.groups.iter().position(|g| g.name == bx.group);
            if let Some(idx) = idx {
                bx.group_index = idx as u16;
                self.groups[idx].start_time = f32::min(self.groups[idx].start_time, bx.start_time);
                self.groups[idx].finish_time = f32::max(self.groups[idx].finish_time, bx.finish_time);
            }
            else {
                bx.group_index = self.groups.len() as u16;
                self.groups
                    .push(HitGroup::new(bx.group.clone(), bx.start_time, bx.finish_time));
            }
        }
    }

    fn init_box_ptrs(&mut self) {
        for bx in &self.joint_boxes {
            debug_assert_eq!(bx.box_index as usize, self.boxes_ptrs.len());
            self.boxes_ptrs.push(&bx._base as *const _);
        }
        for bx in &self.weapon_boxes {
            debug_assert_eq!(bx.box_index as usize, self.boxes_ptrs.len());
            self.boxes_ptrs.push(&bx._base as *const _);
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn groups(&self) -> &[HitGroup] {
        &self.groups
    }

    #[inline]
    pub fn joint_boxes(&self) -> &[HitBoxJoint] {
        &self.joint_boxes
    }

    #[inline]
    pub fn weapon_boxes(&self) -> &[HitBoxWeapon] {
        &self.weapon_boxes
    }

    #[inline]
    pub fn to_joint_box_index(&self, box_index: u16) -> Option<u16> {
        if box_index < self.joint_offset {
            return None;
        }
        let idx = box_index - self.joint_offset;
        match (idx as usize) < self.joint_boxes.len() {
            true => Some(idx),
            false => None,
        }
    }

    #[inline]
    pub fn to_weapon_box_index(&self, box_index: u16) -> Option<u16> {
        if box_index < self.weapon_offset {
            return None;
        }
        let idx = box_index - self.weapon_offset;
        match (idx as usize) < self.weapon_boxes.len() {
            true => Some(idx),
            false => None,
        }
    }

    #[inline]
    pub fn find_box_joint(&self, box_index: u16) -> Option<&HitBoxJoint> {
        if box_index > self.joint_offset {
            return None;
        }
        self.joint_boxes.get((box_index - self.joint_offset) as usize)
    }

    #[inline]
    pub fn find_box_weapon(&self, box_index: u16) -> Option<&HitBoxWeapon> {
        if box_index > self.weapon_offset {
            return None;
        }
        self.weapon_boxes.get((box_index - self.weapon_offset) as usize)
    }

    #[inline]
    pub fn find_box(&self, box_index: u16) -> Option<&HitBoxBase> {
        // Safety: HitMotion is immutable, so it is safe to cache pointers.
        match self.boxes_ptrs.get(box_index as usize).cloned() {
            Some(ptr) => unsafe { Some(&*ptr) },
            None => None,
        }
    }

    #[inline]
    pub(crate) fn count_boxes(&self) -> usize {
        self.boxes_ptrs.len()
    }

    #[inline]
    pub(crate) fn iter_boxes(&self) -> impl Iterator<Item = &HitBoxBase> {
        // Safety: HitMotion is immutable, so it is safe to cache pointers.
        self.boxes_ptrs.iter().map(|&ptr| unsafe { &*ptr })
    }
}

#[derive(Debug, PartialEq)]
pub struct HitGroup {
    pub name: Symbol,
    pub start_time: f32,
    pub finish_time: f32,
}

impl HitGroup {
    #[inline]
    fn new(name: Symbol, start_time: f32, finish_time: f32) -> HitGroup {
        HitGroup {
            name,
            start_time,
            finish_time,
        }
    }

    #[inline]
    pub fn in_time_loose(&self, time: f32) -> bool {
        loose_ge!(time, self.start_time) && loose_le!(time, self.finish_time)
    }
}

#[derive(Debug)]
pub struct HitBoxBase {
    pub box_index: u16,
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

impl HitBoxBase {
    fn new(
        box_index: u16,
        shape: JRef<Shape>,
        typ: HitType,
        group: Symbol,
        positions_keys: &[HitKeyPosition],
        rotations_keys: &[HitKeyRotation],
    ) -> XResult<HitBoxBase> {
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

        Ok(HitBoxBase {
            box_index,
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
pub struct HitBoxJoint {
    pub _base: HitBoxBase,
    pub joint: Symbol,
    pub ratio: f32,
    pub joint2: Symbol,
}

impl Deref for HitBoxJoint {
    type Target = HitBoxBase;
    fn deref(&self) -> &Self::Target {
        &self._base
    }
}

impl DerefMut for HitBoxJoint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._base
    }
}

impl HitBoxJoint {
    #[inline]
    fn from_raw(
        raw: RawHitBoxJoint,
        box_index: u16,
        shape: JRef<Shape>,
        pos_keys: &[HitKeyPosition],
        rot_keys: &[HitKeyRotation],
    ) -> XResult<HitBoxJoint> {
        Ok(HitBoxJoint {
            _base: HitBoxBase::new(box_index, shape, raw.typ, raw.group, pos_keys, rot_keys)?,
            joint: raw.joint,
            ratio: raw.ratio,
            joint2: raw.joint2,
        })
    }

    #[inline]
    fn from_archived(
        raw: &ArchivedRawHitBoxJoint,
        box_index: u16,
        shape: JRef<Shape>,
        pos_keys: &[HitKeyPosition],
        rot_keys: &[HitKeyRotation],
    ) -> XResult<HitBoxJoint> {
        Ok(HitBoxJoint {
            _base: HitBoxBase::new(box_index, shape, raw.typ, Symbol::from(&raw.group), pos_keys, rot_keys)?,
            joint: Symbol::from(&raw.joint),
            ratio: raw.ratio.into(),
            joint2: Symbol::from(&raw.joint2),
        })
    }
}

#[derive(Debug)]
pub struct HitBoxWeapon {
    pub _base: HitBoxBase,
    pub weapon: Symbol,
}

impl Deref for HitBoxWeapon {
    type Target = HitBoxBase;
    fn deref(&self) -> &Self::Target {
        &self._base
    }
}

impl DerefMut for HitBoxWeapon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._base
    }
}

impl HitBoxWeapon {
    #[inline]
    fn from_raw(
        raw: RawHitBoxWeapon,
        hit_index: u16,
        shape: JRef<Shape>,
        pos_keys: &[HitKeyPosition],
        rot_keys: &[HitKeyRotation],
    ) -> XResult<HitBoxWeapon> {
        Ok(HitBoxWeapon {
            _base: HitBoxBase::new(hit_index, shape, raw.typ, raw.group, pos_keys, rot_keys)?,
            weapon: raw.weapon,
        })
    }

    #[inline]
    fn from_archived(
        raw: &ArchivedRawHitBoxWeapon,
        hit_index: u16,
        shape: JRef<Shape>,
        pos_keys: &[HitKeyPosition],
        rot_keys: &[HitKeyRotation],
    ) -> XResult<HitBoxWeapon> {
        Ok(HitBoxWeapon {
            _base: HitBoxBase::new(hit_index, shape, raw.typ, Symbol::from(&raw.group), pos_keys, rot_keys)?,
            weapon: Symbol::from(&raw.weapon),
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

        assert_eq!(hit_motion.groups.as_slice(), &[
            HitGroup::new(sb!("Health"), 0.5, 0.8333333),
            HitGroup::new(sb!("Counter"), 1.0, 1.13333333),
            HitGroup::new(sb!("Axe"), 1.05, 1.36666667),
        ]);

        assert_eq!(hit_motion.joint_boxes.len(), 2);

        assert_eq!(hit_motion.joint_boxes[0].box_index, 0);
        assert!(hit_motion.joint_boxes[0].shape.count_ref() > 0);
        assert_eq!(hit_motion.joint_boxes[0].typ, HitType::Health);
        assert_eq!(hit_motion.joint_boxes[0].group, sb!("Health"));
        assert_eq!(hit_motion.joint_boxes[0].group_index, 0);
        assert_eq!(hit_motion.joint_boxes[0].joint, sb!("Spine"));
        assert_eq!(hit_motion.joint_boxes[0].ratio, 0.5);
        assert_eq!(hit_motion.joint_boxes[0].joint2, sb!(""));
        assert_eq!(hit_motion.joint_boxes[0].positions_keys(), &[
            HitKeyPosition::new(0.5, Vec3A::new(0.0, 0.0, 0.0)),
            HitKeyPosition::new(0.8333333, Vec3A::new(0.0, 0.0, 0.0)),
        ]);
        assert_eq!(hit_motion.joint_boxes[0].rotations_keys(), &[
            HitKeyRotation::new(0.5, Quat::from_xyzw(0.0, 0.0, 0.0, 1.0)),
            HitKeyRotation::new(0.8333333, Quat::from_xyzw(0.0, 0.0, 0.0, 1.0)),
        ]);

        assert_eq!(hit_motion.joint_boxes[1].box_index, 1);
        assert!(hit_motion.joint_boxes[1].shape.count_ref() > 0);
        assert_eq!(hit_motion.joint_boxes[1].typ, HitType::Counter);
        assert_eq!(hit_motion.joint_boxes[1].group, sb!("Counter"));
        assert_eq!(hit_motion.joint_boxes[1].group_index, 1);
        assert_eq!(hit_motion.joint_boxes[1].joint, sb!("LeftHand"));
        assert_eq!(hit_motion.joint_boxes[1].ratio, 0.2);
        assert_eq!(hit_motion.joint_boxes[1].joint2, sb!("LeftLowerArm"));
        assert_eq!(hit_motion.joint_boxes[1].positions_keys(), &[
            HitKeyPosition::new(1.0, Vec3A::new(0.1, 0.1, 0.0)),
            HitKeyPosition::new(1.13333333, Vec3A::new(0.1, 0.1, 0.0)),
        ]);
        assert_eq!(hit_motion.joint_boxes[1].rotations_keys(), &[
            HitKeyRotation::new(1.0, Quat::from_xyzw(1.0, 0.0, 0.0, 0.0)),
            HitKeyRotation::new(1.13333333, Quat::from_xyzw(1.0, 0.0, 0.0, 0.0)),
        ]);

        assert_eq!(hit_motion.weapon_boxes.len(), 2);

        assert_eq!(hit_motion.weapon_boxes[0].box_index, 2);
        assert!(hit_motion.weapon_boxes[0].shape.count_ref() > 0);
        assert_eq!(hit_motion.weapon_boxes[0].typ, HitType::Attack);
        assert_eq!(hit_motion.weapon_boxes[0].group, sb!("Axe"));
        assert_eq!(hit_motion.weapon_boxes[0].group_index, 2);
        assert_eq!(hit_motion.weapon_boxes[0].weapon, sb!("Axe"));
        assert_eq!(hit_motion.weapon_boxes[0].positions_keys(), &[
            HitKeyPosition::new(1.05, Vec3A::new(0.0, -0.15, -0.7)),
            HitKeyPosition::new(1.2, Vec3A::new(0.0, -0.15, -0.7)),
        ]);
        assert_eq!(hit_motion.weapon_boxes[0].rotations_keys(), &[
            HitKeyRotation::new(1.05, Quat::from_xyzw(0.7071068, 0.0, 0.0, 0.7071068)),
            HitKeyRotation::new(1.2, Quat::from_xyzw(0.7071068, 0.0, 0.0, 0.7071068)),
        ]);

        assert_eq!(hit_motion.weapon_boxes[1].box_index, 3);
        assert!(hit_motion.weapon_boxes[1].shape.count_ref() > 0);
        assert_eq!(hit_motion.weapon_boxes[1].typ, HitType::Attack);
        assert_eq!(hit_motion.weapon_boxes[1].group, sb!("Axe"));
        assert_eq!(hit_motion.weapon_boxes[1].group_index, 2);
        assert_eq!(hit_motion.weapon_boxes[1].weapon, sb!("Axe"));
        assert_eq!(hit_motion.weapon_boxes[1].positions_keys(), &[
            HitKeyPosition::new(1.2166667, Vec3A::new(0.0, -0.15, -0.7)),
            HitKeyPosition::new(1.36666667, Vec3A::new(0.0, -0.15, -0.7)),
        ]);
        assert_eq!(hit_motion.weapon_boxes[1].rotations_keys(), &[
            HitKeyRotation::new(1.2166667, Quat::from_xyzw(0.7071068, 0.0, 0.0, 0.7071068)),
            HitKeyRotation::new(1.36666667, Quat::from_xyzw(0.7071068, 0.0, 0.0, 0.7071068)),
        ]);
    }

    #[test]
    fn test_hit_motion_from_json_reader() {
        let json_path = format!("{}/Girl_Attack_Test.hm-json", TEST_ASSET_PATH);
        let mut json_file = File::open(&json_path).unwrap();
        let mut json_buf = Vec::new();
        json_file.read_to_end(&mut json_buf).unwrap();
        let hit_motion = HitMotion::from_json_bytes(&json_buf, Some(&json_path)).unwrap();
        check_hit_motion(&hit_motion);
    }
}
