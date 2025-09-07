use glam_ext::{Mat4, Quat, Transform3A, Vec3};
use libc::c_char;
use ozz_animation_rs::{
    ozz_rc_buf, Animation, Archive, BlendingJob, BlendingLayer, LocalToModelJob, SamplingContext, SamplingJob,
    Skeleton, SoaTransform, Track, TrackSamplingJob,
};
use std::ffi::CStr;
use std::ptr;
use std::rc::Rc;

use cirtical_point_core::animation::{rest_poses_to_model_matrices, SkeletonJointMeta, SkeletonMeta};
use cirtical_point_core::utils::{lerp, xerrf, xres, XResult};

use crate::utils::{as_slice, Return};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct ClipAnimation {
    animation: String,
    motion: String,
    start_ratio: f32,
    finish_ratio: f32,
    fade_in_secs: f32,
}

#[allow(dead_code)]
struct ClipInstance {
    animation: Rc<Animation>,
    start_ratio: f32,
    finish_ratio: f32,
    start_secs: f32,
    finish_secs: f32,
    fade_in_secs: f32,
    duration_secs: f32,

    sampling_job: SamplingJob,
    pos_motion_job: Option<TrackSamplingJob<Vec3>>,
    rot_motion_job: Option<TrackSamplingJob<Quat>>,
    root_motion: Transform3A,
}

pub struct SkeletalPlayer {
    clips: Vec<ClipInstance>,
    is_loop: bool,
    duration_secs: f32,
    progress_secs: f32,

    skeleton: Rc<Skeleton>,
    blending_job: BlendingJob,
    l2m_job: LocalToModelJob,

    // local_rest_poses: Vec<Transform3A>,
    model_rest_poses: Vec<Mat4>,
    // local_out: Vec<Transform3A>,
    root_motion: Mat4,
}

#[cfg(feature = "debug-print")]
impl Drop for SkeletalPlayer {
    fn drop(&mut self) {
        log::debug!("SkeletalPlayer::drop()");
    }
}

impl SkeletalPlayer {
    fn new(skeleton_path: &str) -> XResult<SkeletalPlayer> {
        let skeleton = Rc::new(Skeleton::from_path(skeleton_path)?);

        let mut blending_job = BlendingJob::default();
        blending_job.set_skeleton(skeleton.clone());
        blending_job.set_output(ozz_rc_buf(vec![SoaTransform::default(); skeleton.num_soa_joints()]));

        let mut l2m_job = LocalToModelJob::default();
        l2m_job.set_skeleton(skeleton.clone());
        l2m_job.set_output(ozz_rc_buf(vec![Mat4::IDENTITY; skeleton.num_joints()]));

        // let mut local_rest_poses = vec![Transform3A::IDENTITY; skeleton.num_joints()];
        // soa_transforms_to_transforms(skeleton.joint_rest_poses(), &mut local_rest_poses)?;

        let mut model_rest_poses = vec![Mat4::IDENTITY; skeleton.num_joints()];
        rest_poses_to_model_matrices(&skeleton, &mut model_rest_poses)?;

        Ok(SkeletalPlayer {
            is_loop: false,
            clips: Vec::new(),
            duration_secs: 0.0,
            progress_secs: 0.0,

            skeleton: skeleton.clone(),
            blending_job,
            l2m_job,

            // local_rest_poses,
            model_rest_poses,
            // local_out: vec![Transform3A::IDENTITY; skeleton.num_joints()],
            root_motion: Mat4::IDENTITY,
        })
    }

    fn skeleton_meta(&self) -> SkeletonMeta {
        let mut joint_metas = vec![SkeletonJointMeta::default(); self.skeleton.num_joints() as usize];
        for (name, index) in self.skeleton.joint_names() {
            joint_metas[*index as usize] = SkeletonJointMeta {
                index: *index as i16,
                parent: self.skeleton.joint_parent(*index),
                name: name.clone(),
            };
        }
        SkeletonMeta {
            num_joints: self.skeleton.num_joints() as u32,
            num_soa_joints: self.skeleton.num_soa_joints() as u32,
            joint_metas,
        }
    }

    fn set_animations(&mut self, animations: Vec<ClipAnimation>, is_loop: bool) -> XResult<()> {
        let mut instances = Vec::with_capacity(animations.len());
        let mut duration_secs = 0.0;

        for cfg in animations {
            let animation = Rc::new(Animation::from_path(&cfg.animation)?);

            if cfg.finish_ratio <= cfg.start_ratio {
                return xres!(BadArgument; "finish_ratio <= start_ratio");
            }
            let start_secs = duration_secs;
            let finish_secs = duration_secs + (cfg.finish_ratio - cfg.start_ratio) * animation.duration();
            if cfg.fade_in_secs >= finish_secs - start_secs {
                return xres!(BadArgument; "invalid fade_in_secs");
            }

            let mut sampling_job = SamplingJob::default();
            sampling_job.set_output(ozz_rc_buf(vec![
                SoaTransform::default();
                self.skeleton.num_soa_joints()
            ]));
            sampling_job.set_animation(animation.clone());
            sampling_job.set_context(SamplingContext::from_animation(&animation));

            let mut pos_motion_job = None;
            let mut rot_motion_job = None;
            let mut root_motion = Transform3A::IDENTITY;
            if !cfg.motion.is_empty() {
                let mut archive = Archive::from_path(&cfg.motion)?;

                let mut pos_job = TrackSamplingJob::default();
                pos_job.set_track(Rc::new(Track::<Vec3>::from_archive(&mut archive)?));

                let mut rot_job = TrackSamplingJob::default();
                rot_job.set_track(Rc::new(Track::<Quat>::from_archive(&mut archive)?));

                root_motion = Self::compute_root_motion(&mut pos_job, &mut rot_job, cfg.start_ratio, cfg.finish_ratio)?;
                pos_motion_job = Some(pos_job);
                rot_motion_job = Some(rot_job);
            }

            instances.push(ClipInstance {
                animation,
                start_ratio: cfg.start_ratio,
                finish_ratio: cfg.finish_ratio,
                start_secs,
                finish_secs,
                fade_in_secs: cfg.fade_in_secs,
                duration_secs: finish_secs - start_secs,

                sampling_job,
                pos_motion_job,
                rot_motion_job,
                root_motion,
            });
            duration_secs = finish_secs;
        }

        self.clips = instances;
        self.is_loop = is_loop;
        self.duration_secs = duration_secs;
        self.progress_secs = 0.0;
        Ok(())
    }

    fn duration(&self) -> f32 {
        match self.clips.is_empty() {
            true => 0.0,
            false => self.duration_secs,
        }
    }

    fn progress(&self) -> f32 {
        self.progress_secs
    }

    fn set_progress(&mut self, progress: f32) {
        if self.is_loop {
            self.progress_secs = progress.rem_euclid(self.duration_secs);
        }
        else {
            self.progress_secs = progress.clamp(0.0, self.duration_secs);
        }
    }

    fn add_progress(&mut self, delta: f32) {
        self.progress_secs += delta;
        if self.is_loop {
            self.progress_secs = self.progress_secs.rem_euclid(self.duration_secs);
        }
        else {
            self.progress_secs = self.progress_secs.clamp(0.0, self.duration_secs);
        }
    }

    fn update(&mut self) -> XResult<()> {
        if self.clips.is_empty() {
            return Ok(());
        }

        let mut clip_idx: usize = 0;
        let mut fade_in = false;
        for (idx, clip) in self.clips.iter().enumerate() {
            if self.progress_secs >= clip.start_secs && self.progress_secs <= clip.finish_secs {
                clip_idx = idx;
                fade_in = idx > 0 && (self.progress_secs - clip.start_secs) < clip.fade_in_secs;
                break;
            }
        }

        self.root_motion = Mat4::IDENTITY;
        for (idx, clip) in self.clips.iter_mut().take(clip_idx + 1).enumerate() {
            if idx < clip_idx {
                self.root_motion = self.root_motion * clip.root_motion;
            }
            else {
                if clip.pos_motion_job.is_some() && clip.rot_motion_job.is_some() {
                    let ratio = Self::ratio_from_secs(clip, self.progress_secs, false);
                    let pos_job = clip.pos_motion_job.as_mut().unwrap();
                    let rot_job = clip.rot_motion_job.as_mut().unwrap();
                    let transform = Self::compute_root_motion(pos_job, rot_job, clip.start_ratio, ratio)?;
                    self.root_motion = self.root_motion * transform;
                }
            }
        }

        if fade_in {
            let (left, right) = self.clips.split_at_mut(clip_idx);
            let clip = &mut right[0];
            let prev_clip = &mut left[left.len() - 1];

            let ratio = Self::ratio_from_secs(clip, self.progress_secs, true);
            clip.sampling_job.set_ratio(ratio);
            clip.sampling_job.run()?;

            prev_clip.sampling_job.set_ratio(prev_clip.finish_ratio % 1.0);
            prev_clip.sampling_job.run()?;

            let fade_secs = self.progress_secs - clip.start_secs;
            self.blending_job.layers_mut().clear();
            self.blending_job.layers_mut().push(BlendingLayer::with_weight(
                clip.sampling_job.output().unwrap().clone(),
                fade_secs / clip.fade_in_secs,
            ));
            self.blending_job.layers_mut().push(BlendingLayer::with_weight(
                prev_clip.sampling_job.output().unwrap().clone(),
                1.0 - fade_secs / clip.fade_in_secs,
            ));
            self.blending_job.run()?;

            self.l2m_job.set_input(self.blending_job.output().unwrap().clone());
            self.l2m_job.run()?;
        }
        else {
            let clip = &mut self.clips[clip_idx];
            let ratio = Self::ratio_from_secs(clip, self.progress_secs, true);
            clip.sampling_job.set_ratio(ratio);
            clip.sampling_job.run()?;

            self.l2m_job.set_input(clip.sampling_job.output().unwrap().clone());
            self.l2m_job.run()?;
        }

        // soa_transforms_to_transforms(self.l2m_job.input().unwrap().borrow().as_slice(), &mut self.local_out);
        Ok(())
    }

    fn ratio_from_secs(clip: &ClipInstance, progress_secs: f32, wrapping: bool) -> f32 {
        let delta = (progress_secs - clip.start_secs) / (clip.finish_secs - clip.start_secs);
        let ratio = lerp!(clip.start_ratio, clip.finish_ratio, delta);
        if wrapping {
            ratio.rem_euclid(1.0)
        }
        else {
            ratio
        }
    }

    fn compute_root_motion(
        pos_job: &mut TrackSamplingJob<Vec3>,
        rot_job: &mut TrackSamplingJob<Quat>,
        from: f32,
        to: f32,
    ) -> XResult<Transform3A> {
        let from_trunc = from.trunc();
        let from_frac = from - from_trunc;
        let to_trunc = to.trunc();
        let to_frac = to - to_trunc;

        let first_pos = *pos_job.track().unwrap().values().first().unwrap_or(&Vec3::ZERO);
        let last_pos = *pos_job.track().unwrap().values().last().unwrap_or(&Vec3::ZERO);
        let trunc_pos = (last_pos - first_pos) * (to_trunc - from_trunc);

        pos_job.set_ratio(from_frac);
        pos_job.run()?;
        let pos1 = pos_job.result();
        pos_job.set_ratio(to_frac);
        pos_job.run()?;
        let pos2 = pos_job.result();
        let pos_diff = pos2 - pos1 + trunc_pos;

        let mut trunc_rot = Quat::IDENTITY;
        if to_trunc != from_trunc {
            let first_rot = *rot_job.track().unwrap().values().first().unwrap_or(&Quat::IDENTITY);
            let last_rot = *rot_job.track().unwrap().values().last().unwrap_or(&Quat::IDENTITY);
            trunc_rot = (last_rot * first_rot.inverse()) * (to_trunc - from_trunc);
        }

        rot_job.set_ratio(from_frac);
        rot_job.run()?;
        let rot1 = rot_job.result();
        rot_job.set_ratio(to_frac);
        rot_job.run()?;
        let rot2 = rot_job.result();
        let rot_diff = rot2 * rot1.inverse() * trunc_rot;

        Ok(Transform3A::from_rotation_translation(rot_diff, pos_diff))
    }
}

#[no_mangle]
pub extern "C" fn skeletal_player_create(skeleton_path: *const c_char) -> Return<*mut SkeletalPlayer> {
    let res: XResult<*mut SkeletalPlayer> = (|| {
        let mut skeleton = "";
        if !skeleton_path.is_null() {
            skeleton = unsafe { CStr::from_ptr(skeleton_path) }.to_str()?
        };
        let playback = Box::new(SkeletalPlayer::new(skeleton)?);
        Ok(Box::into_raw(playback))
    })();
    Return::from_result_with(res, ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn skeletal_player_destroy(playback: *mut SkeletalPlayer) {
    if !playback.is_null() {
        unsafe { drop(Box::from_raw(playback)) };
    }
}

#[no_mangle]
pub extern "C" fn skeletal_player_skeleton_meta<'t>(playback: *mut SkeletalPlayer) -> Return<*const SkeletonMeta> {
    let res: XResult<*const SkeletonMeta> = (|| {
        let playback = as_playback(playback)?;
        let meta = Box::new(playback.skeleton_meta());
        Ok(Box::into_raw(meta) as *const _)
    })();
    Return::from_result_with(res, ptr::null())
}

#[no_mangle]
pub extern "C" fn skeletal_player_set_animations(
    playback: *mut SkeletalPlayer,
    animations_data: *const u8,
    animations_len: u32,
    is_loop: bool,
) -> Return<()> {
    let res: XResult<()> = (|| {
        let playback = as_playback(playback)?;
        let animations_buf = as_slice(
            animations_data,
            animations_len,
            "skeletal_player_set_animations() animations data is null",
        )?;
        let animations: Vec<ClipAnimation> =
            rmp_serde::from_slice(animations_buf).map_err(|e| xerrf!(BadArgument; "{}", e.to_string()))?;
        playback.set_animations(animations, is_loop)
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_player_duration(playback: *mut SkeletalPlayer) -> Return<f32> {
    let res: XResult<f32> = as_playback(playback).map(|p| p.duration());
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_player_progress(playback: *mut SkeletalPlayer) -> Return<f32> {
    let res: XResult<f32> = as_playback(playback).map(|p| p.progress());
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_player_set_progress(playback: *mut SkeletalPlayer, progress: f32) -> Return<()> {
    let res: XResult<()> = as_playback(playback).map(|p| p.set_progress(progress));
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_player_add_progress(playback: *mut SkeletalPlayer, delta: f32) -> Return<()> {
    let res: XResult<()> = as_playback(playback).map(|p| p.add_progress(delta));
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_player_update(playback: *mut SkeletalPlayer) -> Return<()> {
    let res: XResult<()> = as_playback(playback).and_then(|p| p.update());
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_player_model_rest_poses<'t>(playback: *mut SkeletalPlayer) -> Return<&'t [Mat4]> {
    let res: XResult<&[Mat4]> = (|| {
        let playback = as_playback(playback)?;
        let ptr = playback.model_rest_poses.as_ptr();
        let len = playback.model_rest_poses.len();
        Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
    })();
    Return::from_result_with(res, &[])
}

#[no_mangle]
pub extern "C" fn skeletal_player_model_out<'t>(playback: *mut SkeletalPlayer) -> Return<&'t [Mat4]> {
    let res: XResult<&[Mat4]> = (|| {
        let playback = as_playback(playback)?;
        if !playback.clips.is_empty() {
            let model_out = playback.l2m_job.output().unwrap();
            let model_out = model_out.borrow();
            let ptr = model_out.as_ptr();
            let len = model_out.len();
            Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
        }
        else {
            let ptr = playback.model_rest_poses.as_ptr();
            let len = playback.model_rest_poses.len();
            Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
        }
    })();
    Return::from_result_with(res, &[])
}

#[no_mangle]
pub extern "C" fn skeletal_player_root_motion_out<'t>(playback: *mut SkeletalPlayer) -> Return<Mat4> {
    let res: XResult<Mat4> = (|| {
        let playback = as_playback(playback)?;
        match playback.clips.is_empty() {
            true => Ok(Mat4::IDENTITY),
            false => Ok(playback.root_motion),
        }
    })();
    Return::from_result_with(res, Mat4::IDENTITY)
}

fn as_playback<'t>(playback: *mut SkeletalPlayer) -> XResult<&'t mut SkeletalPlayer> {
    if playback.is_null() {
        return xres!(BadArgument; "playback=null");
    }
    Ok(unsafe { &mut *playback })
}
