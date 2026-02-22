use glam::{Quat, Vec3A};
use glam_ext::{Isometry3A, Transform3A};
use ozz_animation_rs::Skeleton;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use thin_vec::ThinVec;

use crate::animation::hit_motion::{HitKeyPosition, HitKeyRotation, HitMotion};
use crate::animation::{HitTrackJoint, HitTrackWeapon, WeaponTransform};
use crate::utils::{strict_gt, strict_lt, xerrf, XResult};

#[derive(Debug)]
pub(crate) struct HitMotionSampler {
    pub(crate) hit_motion: Rc<HitMotion>,
    joints: ThinVec<HitSamplerJoint>,
    weapons: ThinVec<HitSamplerWeapon>,
}

impl HitMotionSampler {
    pub(crate) fn new(hit_motion: Rc<HitMotion>, skeleton: &Skeleton) -> XResult<HitMotionSampler> {
        let mut sampler = HitMotionSampler {
            hit_motion: hit_motion.clone(),
            joints: ThinVec::with_capacity(hit_motion.joint_tracks.len()),
            weapons: ThinVec::with_capacity(hit_motion.weapon_tracks.len()),
        };

        for track in hit_motion.joint_tracks.iter() {
            let data = HitSamplerJointData::new(&hit_motion, track, skeleton)?;
            sampler.joints.push(HitSampler::new(track.hit_id, data));
        }

        for track in hit_motion.weapon_tracks.iter() {
            sampler
                .weapons
                .push(HitSampler::new(track.hit_id, HitSamplerWeaponData));
        }

        Ok(sampler)
    }

    pub(crate) fn sample(&mut self, time: f32, model_transforms: &[Transform3A], weapon_transform: &[WeaponTransform]) {
        for (idx, track) in self.hit_motion.joint_tracks.iter().enumerate() {
            self.joints[idx].sample(track, time, model_transforms);
        }

        for (idx, track) in self.hit_motion.weapon_tracks.iter().enumerate() {
            self.weapons[idx].sample(track, time, weapon_transform);
        }
    }

    #[inline]
    pub(crate) fn joints(&self) -> &[HitSamplerJoint] {
        &self.joints
    }

    #[inline]
    pub(crate) fn weapons(&self) -> &[HitSamplerWeapon] {
        &self.weapons
    }

    #[inline]
    pub(crate) fn tracks_count(&self) -> usize {
        self.hit_motion.joint_tracks.len() + self.hit_motion.weapon_tracks.len()
    }
}

#[derive(Debug)]
pub(crate) struct HitSampler<T> {
    pub(crate) hit_id: u16,
    pub(crate) active: bool,
    pos_cursor: u32,
    rot_cursor: u32,
    isometry: Isometry3A,
    data: T,
}

pub(crate) type HitSamplerJoint = HitSampler<HitSamplerJointData>;
pub(crate) type HitSamplerWeapon = HitSampler<HitSamplerWeaponData>;

impl<T> Deref for HitSampler<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for HitSampler<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> HitSampler<T> {
    fn new(hit_id: u16, data: T) -> HitSampler<T> {
        HitSampler {
            hit_id,
            active: false,
            pos_cursor: 0,
            rot_cursor: 0,
            isometry: Isometry3A::IDENTITY,
            data,
        }
    }

    #[inline]
    pub(crate) fn isometry(&self) -> Option<&Isometry3A> {
        match self.active {
            true => Some(&self.isometry),
            false => None,
        }
    }

    fn sample_inner(
        &mut self,
        start_time: f32,
        finish_time: f32,
        positions_keys: &[HitKeyPosition],
        rotations_keys: &[HitKeyRotation],
        time: f32,
    ) {
        if strict_lt!(time, start_time, 1e-2) {
            self.active = false;
            self.pos_cursor = 0;
            self.rot_cursor = 0;
            return;
        }
        else if strict_gt!(time, finish_time, 1e-2) {
            self.active = false;
            self.pos_cursor = positions_keys.len() as u32 - 2;
            self.rot_cursor = rotations_keys.len() as u32 - 2;
            return;
        }

        // position

        let pos_len = positions_keys.len() as u32;
        self.pos_cursor %= pos_len - 1;

        if time < positions_keys[self.pos_cursor as usize].time {
            while self.pos_cursor > 0 {
                self.pos_cursor -= 1;
                if time >= positions_keys[self.pos_cursor as usize].time {
                    break;
                }
            }
        }
        else if time > positions_keys[self.pos_cursor as usize + 1].time {
            while self.pos_cursor < pos_len - 2 {
                self.pos_cursor += 1;
                if time <= positions_keys[self.pos_cursor as usize + 1].time {
                    break;
                }
            }
        }

        let pos_left = positions_keys[self.pos_cursor as usize];
        let pos_right = positions_keys[self.pos_cursor as usize + 1];
        let pos = Vec3A::lerp(
            pos_left.value(),
            pos_right.value(),
            (time - pos_left.time) / (pos_right.time - pos_left.time),
        );

        // rotation

        let rot_len = rotations_keys.len() as u32;
        self.rot_cursor %= rot_len - 1;

        if time < rotations_keys[self.rot_cursor as usize].time {
            while self.rot_cursor > 0 {
                self.rot_cursor -= 1;
                if time >= rotations_keys[self.rot_cursor as usize].time {
                    break;
                }
            }
        }
        else if time > rotations_keys[self.rot_cursor as usize + 1].time {
            while self.rot_cursor < rot_len - 2 {
                self.rot_cursor += 1;
                if time <= rotations_keys[self.rot_cursor as usize + 1].time {
                    break;
                }
            }
        }

        let rot_left = rotations_keys[self.rot_cursor as usize];
        let rot_right = rotations_keys[self.rot_cursor as usize + 1];
        let rot = Quat::lerp(
            rot_left.value(),
            rot_right.value(),
            (time - rot_left.time) / (rot_right.time - rot_left.time),
        );

        // results

        self.active = true;
        self.isometry = Isometry3A::new_3a(pos, rot);
    }
}

#[derive(Debug)]
pub(crate) struct HitSamplerJointData {
    pub(crate) joint: i16,
    pub(crate) joint2: i16,
}

impl HitSamplerJointData {
    #[inline]
    fn new(hit_motion: &HitMotion, track: &HitTrackJoint, skeleton: &Skeleton) -> XResult<HitSamplerJointData> {
        let joint = skeleton
            .joint_by_name(track.joint.as_str())
            .ok_or_else(|| xerrf!(BadAsset; "hit_motion={}, joint={}", &hit_motion.name, &track.joint))?;

        let joint2 = if track.joint2.is_empty() {
            -1
        }
        else {
            skeleton
                .joint_by_name(track.joint2.as_str())
                .ok_or_else(|| xerrf!(BadAsset; "hit_motion={}, joint2={}", &hit_motion.name, &track.joint2))?
        };

        Ok(HitSamplerJointData { joint, joint2 })
    }
}

impl HitSampler<HitSamplerJointData> {
    fn sample(&mut self, track: &HitTrackJoint, time: f32, model_transforms: &[Transform3A]) {
        self.sample_inner(
            track.start_time,
            track.finish_time,
            track.positions_keys(),
            track.rotations_keys(),
            time,
        );

        if self.active {
            let transform = model_transforms[self.joint as usize];

            let rotation = transform.rotation * self.isometry.rotation;

            let mut position = transform.translation;
            if self.joint2 >= 0 {
                position = Vec3A::lerp(
                    transform.translation,
                    model_transforms[self.joint2 as usize].translation,
                    track.ratio,
                )
            };
            position += transform.rotation * self.isometry.translation;

            self.isometry = Isometry3A::new_3a(position, rotation);
        }
    }
}

#[derive(Debug)]
pub(crate) struct HitSamplerWeaponData;

impl HitSampler<HitSamplerWeaponData> {
    fn sample(&mut self, track: &HitTrackWeapon, time: f32, weapon_transform: &[WeaponTransform]) {
        self.sample_inner(
            track.start_time,
            track.finish_time,
            track.positions_keys(),
            track.rotations_keys(),
            time,
        );

        if self.active {
            match weapon_transform.iter().find(|wt| wt.name == track.weapon) {
                Some(transform) => {
                    let rotation = transform.rotation * self.isometry.rotation;
                    let position = transform.position + transform.rotation * self.isometry.translation;
                    self.isometry = Isometry3A::new_3a(position, rotation);
                }
                None => self.active = false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_ulps_eq;

    #[test]
    fn test_hit_motion_sample_inner() {
        const P1: Vec3A = Vec3A::new(1.0, 1.0, 1.0);
        const P2: Vec3A = Vec3A::new(2.0, 1.0, -1.0);
        const P3: Vec3A = Vec3A::new(3.0, 0.0, 0.0);
        const P4: Vec3A = Vec3A::new(4.0, 4.0, 4.0);
        let positions_keys = vec![
            HitKeyPosition::new(1.0, P1),
            HitKeyPosition::new(2.0, P2),
            HitKeyPosition::new(3.0, P3),
            HitKeyPosition::new(4.0, P4),
        ];

        const R1: Quat = Quat::from_xyzw(1.0, 0.0, 0.0, 0.0);
        const R2: Quat = Quat::from_xyzw(0.0, 1.0, 0.0, 0.0);
        let rotation_keys = vec![HitKeyRotation::new(1.0, R1), HitKeyRotation::new(4.0, R2)];

        let mut sampler = HitSampler::<HitSamplerWeaponData> {
            hit_id: 0,
            active: false,
            pos_cursor: 0,
            rot_cursor: 0,
            isometry: Isometry3A::IDENTITY,
            data: HitSamplerWeaponData,
        };

        sampler.pos_cursor = 0;
        sampler.rot_cursor = 0;
        sampler.sample_inner(1.0, 4.0, &positions_keys, &rotation_keys, 0.0);
        check3(&sampler, false, 0, 0);

        sampler.pos_cursor = 0;
        sampler.rot_cursor = 0;
        sampler.sample_inner(1.0, 4.0, &positions_keys, &rotation_keys, 5.0);
        check3(&sampler, false, 2, 0);

        sampler.pos_cursor = 0;
        sampler.rot_cursor = 0;
        sampler.sample_inner(1.0, 4.0, &positions_keys, &rotation_keys, 0.9999);
        let pos = Vec3A::lerp(P1, P2, -0.0001);
        let rot = Quat::lerp(R1, R2, -0.0001 / 3.0);
        check5(&sampler, true, 0, 0, pos, rot);

        sampler.pos_cursor = 0;
        sampler.rot_cursor = 0;
        sampler.sample_inner(1.0, 4.0, &positions_keys, &rotation_keys, 4.0001);
        let pos = Vec3A::lerp(P3, P4, 1.0001);
        let rot = Quat::lerp(R1, R2, 3.0001 / 3.0);
        check5(&sampler, true, 2, 0, pos, rot);

        // forward

        sampler.active = false;
        sampler.pos_cursor = 0;
        sampler.rot_cursor = 0;

        sampler.sample_inner(1.0, 4.0, &positions_keys, &rotation_keys, 1.5);
        let pos = Vec3A::lerp(P1, P2, 0.5);
        let rot = Quat::lerp(R1, R2, 0.5 / 3.0);
        check5(&sampler, true, 0, 0, pos, rot);

        sampler.sample_inner(1.0, 4.0, &positions_keys, &rotation_keys, 2.0);
        let pos = Vec3A::lerp(P1, P2, 1.0);
        let rot = Quat::lerp(R1, R2, 1.0 / 3.0);
        check5(&sampler, true, 0, 0, pos, rot);

        sampler.sample_inner(1.0, 4.0, &positions_keys, &rotation_keys, 2.5);
        let pos = Vec3A::lerp(P2, P3, 0.5);
        let rot = Quat::lerp(R1, R2, 1.5 / 3.0);
        check5(&sampler, true, 1, 0, pos, rot);

        sampler.sample_inner(1.0, 4.0, &positions_keys, &rotation_keys, 3.7);
        let pos = Vec3A::lerp(P3, P4, 0.7);
        let rot = Quat::lerp(R1, R2, 2.7 / 3.0);
        check5(&sampler, true, 2, 0, pos, rot);

        sampler.sample_inner(1.0, 4.0, &positions_keys, &rotation_keys, 4.1);
        check3(&sampler, false, 2, 0);

        // backward

        sampler.active = false;
        sampler.pos_cursor = 2;
        sampler.rot_cursor = 0;

        sampler.sample_inner(1.0, 4.0, &positions_keys, &rotation_keys, 3.5);
        let pos = Vec3A::lerp(P3, P4, 0.5);
        let rot = Quat::lerp(R1, R2, 2.5 / 3.0);
        check5(&sampler, true, 2, 0, pos, rot);

        sampler.sample_inner(1.0, 4.0, &positions_keys, &rotation_keys, 1.2);
        let pos = Vec3A::lerp(P1, P2, 0.2);
        let rot = Quat::lerp(R1, R2, 0.2 / 3.0);
        check5(&sampler, true, 0, 0, pos, rot);

        sampler.sample_inner(1.0, 4.0, &positions_keys, &rotation_keys, 0.9);
        check3(&sampler, false, 0, 0);
    }

    fn check3<T>(sampler: &HitSampler<T>, active: bool, pos_cursor: u32, rot_cursor: u32) {
        assert_eq!(sampler.active, active);
        assert_eq!(sampler.pos_cursor, pos_cursor);
        assert_eq!(sampler.rot_cursor, rot_cursor);
    }

    fn check5<T>(sampler: &HitSampler<T>, active: bool, pos_cursor: u32, rot_cursor: u32, pos: Vec3A, rot: Quat) {
        assert_eq!(sampler.active, active);
        assert_eq!(sampler.pos_cursor, pos_cursor);
        assert_eq!(sampler.rot_cursor, rot_cursor);
        assert_ulps_eq!(sampler.isometry.translation, pos);
        assert_ulps_eq!(sampler.isometry.rotation, rot);
    }
}
