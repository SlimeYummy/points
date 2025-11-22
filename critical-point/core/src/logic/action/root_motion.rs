use critical_point_csgen::CsOut;
use glam::{Quat, Vec3, Vec3A};
use ozz_animation_rs::TrackSamplingJobRef;
use std::rc::Rc;

use crate::animation::RootMotionTrack;
use crate::instance::InstAnimation;
use crate::logic::ContextUpdate;
use crate::utils::{xres, CsQuat, XResult};

#[derive(Debug)]
pub(crate) struct RootMotionDynamic {
    ratio: f32,
    position: Vec3A,
    position_delta: Vec3A,
    rotation_cursor: Quat,
    rotation: Quat,
    rotation_delta: Quat,
}

impl Default for RootMotionDynamic {
    fn default() -> Self {
        Self {
            ratio: 0.0,
            position: Vec3A::ZERO,
            position_delta: Vec3A::ZERO,
            rotation_cursor: Quat::IDENTITY,
            rotation: Quat::IDENTITY,
            rotation_delta: Quat::IDENTITY,
        }
    }
}

//
// LogicRootMotion
//

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
    pub ratio: f32,
    pub position: Vec3,
    pub position_delta: Vec3,
    pub rotation_cursor: CsQuat,
    pub rotation: CsQuat,
    pub rotation_delta: CsQuat,
}

#[derive(Debug)]
pub(crate) struct LogicRootMotion {
    track: Rc<RootMotionTrack>,
    dynamic: RootMotionDynamic,
}

#[allow(dead_code)]
impl LogicRootMotion {
    pub fn new(ctx: &mut ContextUpdate<'_>, inst_anim: &InstAnimation, start_ratio: f32) -> XResult<LogicRootMotion> {
        let track = ctx.asset.load_root_motion(inst_anim.files)?;
        let mut zelf = LogicRootMotion {
            track,
            dynamic: RootMotionDynamic::default(),
        };

        if start_ratio != 0.0 {
            if zelf.track.has_position() {
                zelf.dynamic.position = run_root_position_job(&zelf.track, start_ratio)?;
            }

            if zelf.track.has_rotation() {
                zelf.dynamic.rotation_cursor = zelf.track.first_rotation();
                zelf.dynamic.rotation =
                    update_root_rotation_job(&zelf.track, &mut zelf.dynamic.rotation_cursor, 0.0, start_ratio)?;
            }

            zelf.dynamic.ratio = start_ratio;
        }

        Ok(zelf)
    }

    pub fn restore(&mut self, state: &StateRootMotion) {
        self.dynamic.ratio = state.ratio;
        self.dynamic.position = state.position.into();
        self.dynamic.position_delta = state.position_delta.into();
        self.dynamic.rotation_cursor = state.rotation_cursor.into();
        self.dynamic.rotation = state.rotation.into();
        self.dynamic.rotation_delta = state.rotation_delta.into();
    }

    pub fn save(&self) -> StateRootMotion {
        StateRootMotion {
            ratio: self.dynamic.ratio,
            position: self.dynamic.position.into(),
            position_delta: self.dynamic.position_delta.into(),
            rotation_cursor: self.dynamic.rotation_cursor.into(),
            rotation: self.dynamic.rotation.into(),
            rotation_delta: self.dynamic.rotation_delta.into(),
        }
    }

    pub fn update(&mut self, ratio: f32) -> XResult<()> {
        if self.track.has_position() {
            let old_pos = self.dynamic.position;
            self.dynamic.position = run_root_position_job(&self.track, ratio)?;
            self.dynamic.position_delta = self.dynamic.position - old_pos;
        }

        if self.track.has_rotation() {
            self.dynamic.rotation_delta = update_root_rotation_job(
                &self.track,
                &mut self.dynamic.rotation_cursor,
                self.dynamic.ratio,
                ratio,
            )?;
            self.dynamic.rotation *= self.dynamic.rotation_delta;
        }

        self.dynamic.ratio = ratio;
        Ok(())
    }

    #[inline]
    pub fn track(&self) -> &RootMotionTrack {
        &self.track
    }

    #[inline]
    pub fn ratio(&self) -> f32 {
        self.dynamic.ratio
    }

    #[inline]
    pub fn position(&self) -> Vec3A {
        self.dynamic.position
    }

    #[inline]
    pub fn rotation(&self) -> Quat {
        self.dynamic.rotation
    }

    #[inline]
    pub fn position_delta(&self) -> Vec3A {
        self.dynamic.position_delta
    }

    #[inline]
    pub fn rotation_delta(&self) -> Quat {
        self.dynamic.rotation_delta
    }

    #[inline]
    pub fn velocity(&self, step: f32) -> Vec3A {
        self.position_delta() / step
    }
}

//
// LogicMultiRootMotion
//

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
pub struct StateMultiRootMotion {
    pub local_id: u16,
    pub ratio: f32,
    pub position: Vec3,
    pub position_delta: Vec3,
    pub rotation_cursor: CsQuat,
    pub rotation: CsQuat,
    pub rotation_delta: CsQuat,
}

#[derive(Debug)]
pub(crate) struct LogicMultiRootMotion {
    tracks: Vec<Rc<RootMotionTrack>>,
    local_id: u16,
    dynamic: RootMotionDynamic,
}

#[allow(dead_code)]
impl LogicMultiRootMotion {
    #[inline]
    pub fn new<'t, I: Iterator<Item = &'t InstAnimation>>(
        ctx: &mut ContextUpdate<'_>,
        inst_anims: I,
    ) -> XResult<LogicMultiRootMotion> {
        let size_hint = inst_anims.size_hint().0;
        Self::new_with_capacity(ctx, inst_anims, size_hint)
    }

    pub fn new_with_capacity<'t, I: Iterator<Item = &'t InstAnimation>>(
        ctx: &mut ContextUpdate<'_>,
        inst_anims: I,
        capacity: usize,
    ) -> XResult<LogicMultiRootMotion> {
        let mut tracks = Vec::with_capacity(capacity);
        for (idx, inst_anim) in inst_anims.enumerate() {
            tracks.push(ctx.asset.load_root_motion(inst_anim.files)?);
            debug_assert_eq!(idx, inst_anim.local_id as usize);
        }

        Ok(LogicMultiRootMotion {
            tracks,
            local_id: u16::MAX,
            dynamic: RootMotionDynamic::default(),
        })
    }

    pub fn restore(&mut self, state: &StateMultiRootMotion) {
        self.local_id = state.local_id;
        self.dynamic.ratio = state.ratio;
        self.dynamic.position = state.position.into();
        self.dynamic.position_delta = state.position_delta.into();
        self.dynamic.rotation_cursor = state.rotation_cursor.into();
        self.dynamic.rotation = state.rotation.into();
        self.dynamic.rotation_delta = state.rotation_delta.into();
    }

    pub fn save(&self) -> StateMultiRootMotion {
        StateMultiRootMotion {
            local_id: self.local_id,
            ratio: self.dynamic.ratio,
            position: self.dynamic.position.into(),
            position_delta: self.dynamic.position_delta.into(),
            rotation_cursor: self.dynamic.rotation_cursor.into(),
            rotation: self.dynamic.rotation.into(),
            rotation_delta: self.dynamic.rotation_delta.into(),
        }
    }

    pub fn set_track(&mut self, local_id: u16, start_ratio: f32) -> XResult<()> {
        self.local_id = local_id;
        self.dynamic = RootMotionDynamic::default();

        if start_ratio != 0.0 {
            if let Some(track) = self.tracks.get(self.local_id as usize) {
                if track.has_position() {
                    self.dynamic.position = run_root_position_job(track, start_ratio)?;
                }

                if track.has_rotation() {
                    self.dynamic.rotation_cursor = track.first_rotation();
                    self.dynamic.rotation =
                        update_root_rotation_job(track, &mut self.dynamic.rotation_cursor, 0.0, start_ratio)?;
                }

                self.dynamic.ratio = start_ratio;
            }
        }
        Ok(())
    }

    pub fn clear_track(&mut self) {
        self.local_id = u16::MAX;
        self.dynamic = RootMotionDynamic::default();
    }

    pub fn update(&mut self, ratio: f32) -> XResult<()> {
        if let Some(track) = self.tracks.get(self.local_id as usize) {
            if track.has_position() {
                let old_pos = self.dynamic.position;
                self.dynamic.position = run_root_position_job(track, ratio)?;
                self.dynamic.position_delta = self.dynamic.position - old_pos;
            }

            if track.has_rotation() {
                self.dynamic.rotation_delta =
                    update_root_rotation_job(track, &mut self.dynamic.rotation_cursor, self.dynamic.ratio, ratio)?;
                self.dynamic.rotation *= self.dynamic.rotation_delta;
            }

            self.dynamic.ratio = ratio;
        }
        Ok(())
    }

    #[inline]
    pub fn track(&self, local_id: u16) -> &RootMotionTrack {
        &self.tracks[local_id as usize]
    }

    #[inline]
    pub fn ratio(&self) -> f32 {
        self.dynamic.ratio
    }

    #[inline]
    pub fn position(&self) -> Vec3A {
        self.dynamic.position
    }

    #[inline]
    pub fn rotation(&self) -> Quat {
        self.dynamic.rotation
    }

    #[inline]
    pub fn position_delta(&self) -> Vec3A {
        self.dynamic.position_delta
    }

    #[inline]
    pub fn rotation_delta(&self) -> Quat {
        self.dynamic.rotation_delta
    }

    #[inline]
    pub fn velocity(&self, step: f32) -> Vec3A {
        self.position_delta() / step
    }
}

//
// Utils
//

fn run_root_position_job(track: &RootMotionTrack, ratio: f32) -> XResult<Vec3A> {
    let trunc = ratio.floor();
    let frac = ratio - trunc;

    let mut pos_job = TrackSamplingJobRef::default();
    pos_job.set_track(&track.position);
    pos_job.set_ratio(frac);
    pos_job.run()?;
    let frac_pos: Vec3A = pos_job.result().into();

    let last_pos: Vec3A = track.last_position().into();
    let trunc_pos = last_pos * trunc;

    Ok(trunc_pos + frac_pos)
}

fn update_root_rotation_job(
    track: &RootMotionTrack,
    cursor: &mut Quat,
    from_ratio: f32,
    to_ratio: f32,
) -> XResult<Quat> {
    if (to_ratio - from_ratio).abs() > 10.0 {
        return xres!(BadArgument; "from_ratio - to_ratio > 10.0");
    }

    let mut diff;
    let mut rot_job = TrackSamplingJobRef::default();
    rot_job.set_track(&track.rotation);

    if from_ratio <= to_ratio {
        if from_ratio.ceil() >= to_ratio {
            rot_job.set_ratio(to_ratio % 1.0);
            rot_job.run()?;
            diff = rot_job.result() * cursor.inverse();
            *cursor = rot_job.result();
        }
        else {
            let last_rot = track.last_rotation();
            diff = last_rot * cursor.inverse();

            for _ in (from_ratio.ceil() as i64)..=(to_ratio.floor() as i64) {
                diff *= last_rot;
            }

            rot_job.set_ratio(to_ratio % 1.0);
            rot_job.run()?;
            diff *= rot_job.result();
            *cursor = rot_job.result();
        }
    }
    else {
        if from_ratio.floor() <= to_ratio {
            rot_job.set_ratio(to_ratio % 1.0 + 1.0);
            rot_job.run()?;
            diff = cursor.inverse() * rot_job.result();
            *cursor = rot_job.result();
        }
        else {
            let last_rot = track.last_rotation();
            diff = cursor.inverse() * last_rot;

            for _ in (to_ratio.ceil() as i64)..=(from_ratio.floor() as i64) {
                diff *= last_rot.inverse();
            }

            rot_job.set_ratio(to_ratio % 1.0 + 1.0);
            rot_job.run()?;
            diff *= rot_job.result().inverse();
            *cursor = rot_job.result();
        }
    }

    Ok(diff)
}
