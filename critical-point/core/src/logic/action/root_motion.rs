use critical_point_csgen::CsOut;
use glam::Vec3A;
use ozz_animation_rs::TrackSamplingJobRef;
use std::rc::Rc;

use crate::animation::{RootMotion, RootTrackName};
use crate::instance::InstAnimation;
use crate::logic::ContextUpdate;
use crate::utils::{xresf, XResult};

//
// LogicRootMotion
//

#[repr(C)]
#[derive(
    Debug,
    Clone,
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
    pub pos_track: RootTrackName,
    pub ratio: f32,
    pub current_pos: Vec3A,
    pub previous_pos: Vec3A,
    pub pos_delta: Vec3A,
    // pub rotation_cursor: Quat,
    // pub rotation: Quat,
    // pub rotation_delta: Quat,
}

impl Default for StateRootMotion {
    fn default() -> StateRootMotion {
        StateRootMotion {
            pos_track: RootTrackName::Default,
            ratio: 0.0,
            current_pos: Vec3A::ZERO,
            previous_pos: Vec3A::ZERO,
            pos_delta: Vec3A::ZERO,
        }
    }
}

#[derive(Debug)]
pub(crate) struct LogicRootMotion {
    root_motion: Rc<RootMotion>,
    state: StateRootMotion,
}

#[allow(dead_code)]
impl LogicRootMotion {
    pub fn new(ctx: &mut ContextUpdate, inst_anim: &InstAnimation, start_ratio: f32) -> XResult<LogicRootMotion> {
        let root_motion = ctx.asset.load_root_motion(inst_anim.files)?;
        let mut zelf = LogicRootMotion {
            root_motion,
            state: StateRootMotion::default(),
        };

        if start_ratio != 0.0 {
            if zelf.root_motion.has_position(zelf.state.pos_track) {
                zelf.state.current_pos = run_position_job(&zelf.root_motion, zelf.state.pos_track, start_ratio)?;
                zelf.state.previous_pos = zelf.state.current_pos;
            }

            // if zelf.root_motion.has_rotation() {
            //     zelf.state.rotation_cursor = zelf.root_motion.first_rotation();
            //     zelf.state.rotation =
            //         update_rotation_job(&zelf.root_motion, &mut zelf.state.rotation_cursor, 0.0, start_ratio)?;
            // }

            zelf.state.ratio = start_ratio;
        }

        Ok(zelf)
    }

    pub fn restore(&mut self, state: &StateRootMotion) {
        self.state = state.clone();
    }

    pub fn save(&self) -> StateRootMotion {
        self.state.clone()
    }

    pub fn set_position_track(&mut self, pos_track: RootTrackName) -> XResult<()> {
        if !self.root_motion.has_position(pos_track) {
            return xresf!(NotFound; "pos_track={:?}", pos_track);
        }

        if pos_track != self.state.pos_track {
            self.state.pos_track = pos_track;
            self.state.previous_pos = run_position_job(&self.root_motion, self.state.pos_track, self.state.ratio)?;
        }
        Ok(())
    }

    pub fn update(&mut self, ratio: f32) -> XResult<()> {
        if self.root_motion.has_position(self.state.pos_track) {
            self.state.previous_pos = self.state.current_pos;
            self.state.current_pos = run_position_job(&self.root_motion, self.state.pos_track, ratio)?;
            self.state.pos_delta = self.state.current_pos - self.state.previous_pos;
        }

        // if self.root_motion.has_rotation() {
        //     self.state.rotation_delta = update_rotation_job(
        //         &self.root_motion,
        //         &mut self.state.rotation_cursor,
        //         self.state.ratio,
        //         ratio,
        //     )?;
        //     self.state.rotation *= self.state.rotation_delta;
        // }

        self.state.ratio = ratio;
        Ok(())
    }

    #[inline]
    pub fn root_motion(&self) -> &RootMotion {
        &self.root_motion
    }

    #[inline]
    pub fn ratio(&self) -> f32 {
        self.state.ratio
    }

    #[inline]
    pub fn position_delta(&self) -> Vec3A {
        self.state.pos_delta
    }

    #[inline]
    pub fn velocity(&self, step: f32) -> Vec3A {
        self.state.pos_delta / step
    }
}

//
// LogicMultiRootMotion
//

#[repr(C)]
#[derive(
    Debug,
    Clone,
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
    pub pos_track: RootTrackName,
    pub ratio: f32,
    pub current_pos: Vec3A,
    pub previous_pos: Vec3A,
    pub pos_delta: Vec3A,
    // pub rotation_cursor: Quat,
    // pub rotation: Quat,
    // pub rotation_delta: Quat,
}

impl Default for StateMultiRootMotion {
    fn default() -> StateMultiRootMotion {
        StateMultiRootMotion {
            local_id: u16::MAX,
            pos_track: RootTrackName::Default,
            ratio: 0.0,
            current_pos: Vec3A::ZERO,
            previous_pos: Vec3A::ZERO,
            pos_delta: Vec3A::ZERO,
        }
    }
}

#[derive(Debug)]
pub(crate) struct LogicMultiRootMotion {
    root_motions: Vec<Rc<RootMotion>>,
    state: StateMultiRootMotion,
}

#[allow(dead_code)]
impl LogicMultiRootMotion {
    #[inline]
    pub fn new<'t, I: Iterator<Item = &'t InstAnimation>>(
        ctx: &mut ContextUpdate,
        inst_anims: I,
    ) -> XResult<LogicMultiRootMotion> {
        let size_hint = inst_anims.size_hint().0;
        Self::new_with_capacity(ctx, inst_anims, size_hint)
    }

    pub fn new_with_capacity<'t, I: Iterator<Item = &'t InstAnimation>>(
        ctx: &mut ContextUpdate,
        inst_anims: I,
        capacity: usize,
    ) -> XResult<LogicMultiRootMotion> {
        let mut root_motions = Vec::with_capacity(capacity);
        for (idx, inst_anim) in inst_anims.enumerate() {
            root_motions.push(ctx.asset.load_root_motion(inst_anim.files)?);
            debug_assert_eq!(idx, inst_anim.local_id as usize);
        }

        Ok(LogicMultiRootMotion {
            root_motions,
            state: StateMultiRootMotion::default(),
        })
    }

    pub fn restore(&mut self, state: &StateMultiRootMotion) {
        self.state = state.clone();
    }

    pub fn save(&self) -> StateMultiRootMotion {
        self.state.clone()
    }

    pub fn set_local_id(&mut self, local_id: u16, start_ratio: f32) -> XResult<()> {
        self.state = StateMultiRootMotion::default();
        self.state.local_id = local_id;

        if start_ratio != 0.0 {
            if let Some(root_motion) = self.root_motions.get(self.state.local_id as usize) {
                if root_motion.has_position(self.state.pos_track) {
                    self.state.current_pos = run_position_job(root_motion, self.state.pos_track, start_ratio)?;
                    self.state.previous_pos = self.state.current_pos;
                }

                // if root_motion.has_rotation() {
                //     self.state.rotation_cursor = root_motion.first_rotation();
                //     self.state.rotation =
                //         update_rotation_job(root_motion, &mut self.state.rotation_cursor, 0.0, start_ratio)?;
                // }

                self.state.ratio = start_ratio;
            }
        }
        Ok(())
    }

    #[inline]
    pub fn clear_local_id(&mut self) {
        self.state = StateMultiRootMotion::default();
    }

    pub fn set_position_track(&mut self, pos_track: RootTrackName) -> XResult<()> {
        if let Some(root_motion) = self.root_motions.get(self.state.local_id as usize) {
            if !root_motion.has_position(pos_track) {
                return xresf!(NotFound; "pos_track={:?}", pos_track);
            }

            if pos_track != self.state.pos_track {
                self.state.pos_track = pos_track;
                self.state.previous_pos = run_position_job(&root_motion, self.state.pos_track, self.state.ratio)?;
            }
        }
        Ok(())
    }

    pub fn update(&mut self, ratio: f32) -> XResult<()> {
        if let Some(track) = self.root_motions.get(self.state.local_id as usize) {
            if track.has_position(self.state.pos_track) {
                let old_pos = self.state.current_pos;
                self.state.current_pos = run_position_job(track, self.state.pos_track, ratio)?;
                self.state.pos_delta = self.state.current_pos - old_pos;
            }

            // if track.has_rotation() {
            //     self.state.rotation_delta =
            //         update_rotation_job(track, &mut self.state.rotation_cursor, self.state.ratio, ratio)?;
            //     self.state.rotation *= self.state.rotation_delta;
            // }

            self.state.ratio = ratio;
        }
        Ok(())
    }

    #[inline]
    pub fn track(&self, local_id: u16) -> &RootMotion {
        &self.root_motions[local_id as usize]
    }

    #[inline]
    pub fn ratio(&self) -> f32 {
        self.state.ratio
    }

    #[inline]
    pub fn position_delta(&self) -> Vec3A {
        self.state.pos_delta
    }

    #[inline]
    pub fn velocity(&self, step: f32) -> Vec3A {
        self.state.pos_delta / step
    }
}

//
// Utils
//

fn run_position_job(root_motion: &RootMotion, pos_track: RootTrackName, ratio: f32) -> XResult<Vec3A> {
    let trunc = ratio.floor();
    let frac = ratio - trunc;

    let mut job = TrackSamplingJobRef::default();
    job.set_track(&root_motion.position(pos_track));
    job.set_ratio(frac);
    job.run()?;
    let frac_pos: Vec3A = job.result().into();

    let last_pos: Vec3A = root_motion.last_position(pos_track).into();
    let trunc_pos = last_pos * trunc;

    Ok(trunc_pos + frac_pos)
}

// fn update_rotation_job(
//     root_motion: &RootMotion,
//     cursor: &mut Quat,
//     from_ratio: f32,
//     to_ratio: f32,
// ) -> XResult<Quat> {
//     if (to_ratio - from_ratio).abs() > 10.0 {
//         return xres!(BadArgument; "from_ratio - to_ratio > 10.0");
//     }

//     let mut diff;
//     let mut job = TrackSamplingJobRef::default();
//     job.set_track(&root_motion.rotation);

//     if from_ratio <= to_ratio {
//         if from_ratio.ceil() >= to_ratio {
//             job.set_ratio(to_ratio % 1.0);
//             job.run()?;
//             diff = job.result() * cursor.inverse();
//             *cursor = job.result();
//         }
//         else {
//             let last_rot = root_motion.last_rotation();
//             diff = last_rot * cursor.inverse();

//             for _ in (from_ratio.ceil() as i64)..=(to_ratio.floor() as i64) {
//                 diff *= last_rot;
//             }

//             job.set_ratio(to_ratio % 1.0);
//             job.run()?;
//             diff *= job.result();
//             *cursor = job.result();
//         }
//     }
//     else {
//         if from_ratio.floor() <= to_ratio {
//             job.set_ratio(to_ratio % 1.0 + 1.0);
//             job.run()?;
//             diff = cursor.inverse() * job.result();
//             *cursor = job.result();
//         }
//         else {
//             let last_rot = root_motion.last_rotation();
//             diff = cursor.inverse() * last_rot;

//             for _ in (to_ratio.ceil() as i64)..=(from_ratio.floor() as i64) {
//                 diff *= last_rot.inverse();
//             }

//             job.set_ratio(to_ratio % 1.0 + 1.0);
//             job.run()?;
//             diff *= job.result().inverse();
//             *cursor = job.result();
//         }
//     }

//     Ok(diff)
// }
