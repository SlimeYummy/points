use cirtical_point_csgen::CsOut;
use glam::{Quat, Vec3, Vec3A};
use ozz_animation_rs::TrackSamplingJobRef;
use std::rc::Rc;

use crate::animation::RootMotionTrack;
use crate::utils::{CsQuat, XResult};

#[repr(C)]
#[derive(
    Debug,
    Default,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Value)]
pub struct StateRootMotion {
    ratio: f32,
    position: Vec3,
    rotation: CsQuat,
    delta_position: Vec3,
    delta_rotation: CsQuat,
}

#[derive(Debug)]
pub(crate) struct LogicRootMotion {
    track: Rc<RootMotionTrack>,
    is_loop: bool,
    max_distance: f32,

    ratio: f32,
    position: Vec3,
    rotation: Quat,
    delta_position: Vec3,
    delta_rotation: Quat,
}

impl LogicRootMotion {
    pub fn new(track: Rc<RootMotionTrack>, is_loop: bool, max_distance: f32) -> XResult<LogicRootMotion> {
        Ok(LogicRootMotion {
            track,
            is_loop,
            max_distance,

            ratio: 0.0,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            delta_position: Vec3::ZERO,
            delta_rotation: Quat::IDENTITY,
        })
    }

    pub fn restore(&mut self, state: &StateRootMotion) {
        self.ratio = state.ratio;
        self.position = state.position;
        self.rotation = state.rotation.into();
        self.delta_position = state.delta_position;
        self.delta_rotation = state.delta_rotation.into();
    }

    pub fn save(&self) -> StateRootMotion {
        StateRootMotion {
            ratio: self.ratio,
            position: self.position,
            rotation: self.rotation.into(),
            delta_position: self.delta_position,
            delta_rotation: self.delta_rotation.into(),
        }
    }

    pub fn update(&mut self, ratio: f32) -> XResult<()> {
        let real_ratio = if self.is_loop {
            ratio.max(0.0) % 1.0
        } else {
            ratio.clamp(0.0, 1.0)
        };

        let mut position_job = TrackSamplingJobRef::default();
        position_job.set_track(&self.track.position);
        let mut rotation_job = TrackSamplingJobRef::default();
        rotation_job.set_track(&self.track.rotation);

        if real_ratio >= self.ratio {
            position_job.set_ratio(ratio);
            position_job.run().unwrap();
            self.delta_position = position_job.result() - self.position;

            rotation_job.set_ratio(ratio);
            rotation_job.run().unwrap();
            self.delta_rotation = rotation_job.result() * self.rotation.inverse();
        } else {
            position_job.set_ratio(1.0);
            position_job.run().unwrap();
            self.delta_position = position_job.result() - self.position;
            position_job.set_ratio(ratio);
            position_job.run().unwrap();
            self.delta_position += position_job.result();

            rotation_job.set_ratio(1.0);
            rotation_job.run().unwrap();
            self.delta_rotation = rotation_job.result() * self.rotation.inverse();
            rotation_job.set_ratio(ratio);
            rotation_job.run().unwrap();
            self.delta_rotation *= rotation_job.result();
        }

        self.ratio = real_ratio;
        self.position = position_job.result();
        self.rotation = rotation_job.result();
        Ok(())
    }
}

#[allow(dead_code)]
impl LogicRootMotion {
    #[inline]
    pub fn ratio(&self) -> f32 {
        self.ratio
    }

    #[inline]
    pub fn position(&self) -> Vec3A {
        (self.position - self.track.position.values()[0]).into()
    }

    #[inline]
    pub fn rotation(&self) -> Quat {
        self.rotation * self.track.rotation.values()[0].inverse()
    }

    #[inline]
    pub fn delta_position(&self) -> Vec3A {
        self.delta_position.into()
    }

    #[inline]
    pub fn delta_rotation(&self) -> Quat {
        self.delta_rotation
    }

    #[inline]
    pub fn velocity(&self, step: f32) -> Vec3A {
        self.delta_position() / step
    }
}
