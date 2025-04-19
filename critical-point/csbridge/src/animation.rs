#![allow(improper_ctypes_definitions)]

use glam_ext::{Mat4, Transform3A};
use libc::c_char;
use ozz_animation_rs::{ozz_rc_buf, Animation, LocalToModelJob, SamplingContext, SamplingJob, Skeleton, SoaTransform};
use std::collections::HashMap;
use std::ffi::CStr;
use std::path::Path;
use std::ptr;
use std::rc::Rc;

use cirtical_point_core::animation::{
    rest_poses_to_local_transforms, rest_poses_to_model_matrices, soa_transforms_to_transforms, SkeletalAnimator,
    SkeletonJointMeta, SkeletonMeta,
};
use cirtical_point_core::consts::MAX_ACTION_ANIMATION;
use cirtical_point_core::logic::StateAction;
use cirtical_point_core::utils::{xerror, ASymbol, XResult};

use crate::utils::Return;

//
// SkeletalAnimator
//

pub struct AnimatorWrapper {
    animator: SkeletalAnimator,
    resource: Box<SkeletalResource>,
}

#[cfg(feature = "debug-print")]
impl Drop for AnimatorWrapper {
    fn drop(&mut self) {
        println!("AnimatorWrapper.drop()");
    }
}

#[no_mangle]
pub extern "C" fn skeletal_animator_create(resource: *mut SkeletalResource, outs: u32) -> Return<*mut AnimatorWrapper> {
    let res: XResult<*mut AnimatorWrapper> = (|| {
        if resource.is_null() {
            return Err(xerror!(BadArgument, "resource=null"));
        }
        if unsafe { &*resource }.sealed {
            return Err(xerror!(BadOperation, "resource consumed"));
        }
        let mut resource = unsafe { Box::from_raw(resource) };
        resource.sealed = true;
        let animator = Box::new(AnimatorWrapper {
            animator: SkeletalAnimator::new(resource.skeleton.clone(), outs, 6, 4 * MAX_ACTION_ANIMATION),
            resource,
        });
        Ok(Box::into_raw(animator))
    })();
    Return::from_result_with(res, ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn skeletal_animator_destroy(animator: *mut AnimatorWrapper) {
    if !animator.is_null() {
        unsafe { drop(Box::from_raw(animator)) };
    }
}

#[no_mangle]
pub extern "C" fn skeletal_animator_skeleton_meta<'t>(animator: *mut AnimatorWrapper) -> Return<*const SkeletonMeta> {
    let res: XResult<*const SkeletonMeta> = (|| {
        let animator = as_animator(animator)?;
        let meta = Box::new(animator.animator.skeleton_meta());
        Ok(Box::into_raw(meta) as *const _)
    })();
    Return::from_result_with(res, ptr::null())
}

#[no_mangle]
pub extern "C" fn skeletal_animator_update(
    animator: *mut AnimatorWrapper,
    frame: u32,
    states: &[Box<dyn StateAction>],
) -> Return<()> {
    let res: XResult<()> = (|| {
        let animator = as_animator(animator)?;
        animator.animator.update(frame, states, |anime_path| {
            match animator.resource.animations.get(anime_path) {
                Some(animation) => Ok(animation.clone()),
                None => Err(xerror!(NotFound, format!("animation={:?}", anime_path))),
            }
        })
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_animator_restore(
    animator: *mut AnimatorWrapper,
    frame: u32,
    states: &[Box<dyn StateAction>],
) -> Return<()> {
    let res: XResult<()> = (|| {
        let animator = as_animator(animator)?;
        animator.animator.restore(frame, states)?;
        Ok(())
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_animator_discard(animator: *mut AnimatorWrapper, frame: u32) -> Return<()> {
    let res: XResult<()> = (|| {
        let animator = as_animator(animator)?;
        animator.animator.discard(frame);
        Ok(())
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_animator_animate(animator: *mut AnimatorWrapper) -> Return<()> {
    let res: XResult<()> = (|| {
        let animator = as_animator(animator)?;
        animator.animator.animate()
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_animator_joint_rest_poses<'t>(animator: *mut AnimatorWrapper) -> Return<&'t [SoaTransform]> {
    let res: XResult<&[SoaTransform]> = (|| {
        let animator = as_animator(animator)?;
        let rest_poses = animator.animator.skeleton_ref().joint_rest_poses();
        Ok(rest_poses)
    })();
    Return::from_result_with(res, &[])
}

#[no_mangle]
pub extern "C" fn skeletal_animator_local_out<'t>(animator: *mut AnimatorWrapper) -> Return<&'t [Transform3A]> {
    let res: XResult<&[Transform3A]> = (|| {
        let animator = as_animator(animator)?;
        let local_out = animator.animator.local_transforms();
        match local_out {
            Some(local_out) => {
                let ptr = local_out.as_ptr();
                let len = local_out.len();
                Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
            }
            None => Ok(&[] as &[Transform3A]),
        }
    })();
    Return::from_result_with(res, &[])
}

#[no_mangle]
pub extern "C" fn skeletal_animator_model_out<'t>(animator: *mut AnimatorWrapper) -> Return<&'t [Mat4]> {
    let res: XResult<&[Mat4]> = (|| {
        let animator = as_animator(animator)?;
        let model_out = animator.animator.model_matrices();
        match model_out {
            Some(model_out) => {
                let ptr = model_out.as_ptr();
                let len = model_out.len();
                Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
            }
            None => Ok(&[] as &[Mat4]),
        }
    })();
    Return::from_result_with(res, &[])
}

fn as_animator<'t>(animator: *mut AnimatorWrapper) -> XResult<&'t mut AnimatorWrapper> {
    if animator.is_null() {
        return Err(xerror!(BadArgument, "animator=null"));
    }
    Ok(unsafe { &mut *animator })
}

//
// SkeletalResource
//

pub struct SkeletalResource {
    skeleton: Rc<Skeleton>,
    animations: HashMap<ASymbol, Rc<Animation>>,
    sealed: bool,
}

impl SkeletalResource {
    fn new<P: AsRef<Path>>(skeleton_path: P) -> XResult<SkeletalResource> {
        let skeleton = Rc::new(Skeleton::from_path(skeleton_path)?);
        Ok(SkeletalResource {
            skeleton,
            animations: HashMap::new(),
            sealed: false,
        })
    }

    fn add_animation<P: AsRef<Path>>(&mut self, logic_path: ASymbol, view_path: P) -> XResult<()> {
        let animation = Rc::new(Animation::from_path(view_path)?);
        self.animations.insert(logic_path, animation);
        Ok(())
    }

    fn remove_animation(&mut self, logic_path: ASymbol) {
        self.animations.remove(&logic_path);
    }

    fn has_animation(&self, logic_path: ASymbol) -> bool {
        self.animations.contains_key(&logic_path)
    }
}

#[cfg(feature = "debug-print")]
impl Drop for SkeletalResource {
    fn drop(&mut self) {
        println!("SkeletalResource.drop()");
    }
}

#[no_mangle]
pub extern "C" fn skeletal_resource_create(skeleton_path: *const c_char) -> Return<*mut SkeletalResource> {
    let res: XResult<*mut SkeletalResource> = (|| {
        let path = unsafe { CStr::from_ptr(skeleton_path) }.to_str()?;
        let resource = Box::new(SkeletalResource::new(path)?);
        Ok(Box::into_raw(resource))
    })();
    Return::from_result_with(res, ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn skeletal_resource_destroy(resource: *mut SkeletalResource) {
    if !resource.is_null() {
        unsafe { drop(Box::from_raw(resource)) };
    }
}

#[no_mangle]
pub extern "C" fn skeletal_resource_add_animation(
    resource: *mut SkeletalResource,
    logic_path: ASymbol,
    view_path: *const c_char,
) -> Return<()> {
    let res: XResult<()> = (|| {
        let resource = as_resource(resource)?;
        if resource.sealed {
            return Err(xerror!(BadOperation, "resource consumed"));
        }
        let path = unsafe { CStr::from_ptr(view_path) }.to_str()?;
        resource.add_animation(logic_path, path)?;
        Ok(())
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_resource_remove_animation(
    resource: *mut SkeletalResource,
    logic_path: ASymbol,
) -> Return<()> {
    let res: XResult<()> = (|| {
        let resource = as_resource(resource)?;
        if resource.sealed {
            return Err(xerror!(BadOperation, "resource consumed"));
        }
        resource.remove_animation(logic_path);
        Ok(())
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_resource_has_animation(
    resource: *mut SkeletalResource,
    logic_path: ASymbol,
) -> Return<bool> {
    let res: XResult<bool> = (|| {
        let resource = as_resource(resource)?;
        Ok(resource.has_animation(logic_path))
    })();
    Return::from_result_with(res, false)
}

fn as_resource<'t>(resource: *mut SkeletalResource) -> XResult<&'t mut SkeletalResource> {
    if resource.is_null() {
        return Err(xerror!(BadArgument, "resource=null"));
    }
    Ok(unsafe { &mut *resource })
}

//
// SkeletalPlayer
//

pub struct SkeletalPlayer {
    skeleton: Rc<Skeleton>,
    sampling_job: SamplingJob,
    l2m_job: LocalToModelJob,
    is_loop: bool,
    progress: f32,
    local_rest_poses: Vec<Transform3A>,
    model_rest_poses: Vec<Mat4>,
    local_out: Vec<Transform3A>,
}

#[cfg(feature = "debug-print")]
impl Drop for SkeletalPlayer {
    fn drop(&mut self) {
        println!("SkeletalPlayer.drop()");
    }
}

impl SkeletalPlayer {
    fn new(skeleton_path: &str) -> XResult<SkeletalPlayer> {
        let skeleton = Rc::new(Skeleton::from_path(skeleton_path)?);

        let mut sampling_job = SamplingJob::default();
        sampling_job.set_output(ozz_rc_buf(vec![SoaTransform::default(); skeleton.num_soa_joints()]));

        let mut l2m_job = LocalToModelJob::default();
        l2m_job.set_skeleton(skeleton.clone());
        l2m_job.set_input(sampling_job.output().unwrap().clone());
        l2m_job.set_output(ozz_rc_buf(vec![Mat4::IDENTITY; skeleton.num_joints()]));

        Ok(SkeletalPlayer {
            skeleton: skeleton.clone(),
            sampling_job,
            l2m_job,
            is_loop: false,
            progress: 0.0,
            local_rest_poses: rest_poses_to_local_transforms(&skeleton)?,
            model_rest_poses: rest_poses_to_model_matrices(&skeleton)?,
            local_out: vec![Transform3A::IDENTITY; skeleton.num_joints()],
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

    fn set_animation(&mut self, animation_path: &str, is_loop: bool) -> XResult<()> {
        if animation_path.is_empty() {
            self.sampling_job.clear_animation();
            self.sampling_job.clear_context();
            // self.sampling_job
            //     .output()
            //     .unwrap()
            //     .borrow_mut()
            //     .copy_from_slice(self.skeleton.joint_rest_poses());
            // self.l2m_job.run()?;
            // self.local_out.copy_from_slice(&self.local_rest_poses);
        } else {
            let animation = Rc::new(Animation::from_path(animation_path)?);
            self.sampling_job.set_animation(animation.clone());
            self.sampling_job
                .set_context(SamplingContext::from_animation(&animation));
        }
        self.is_loop = is_loop;
        self.progress = 0.0;
        Ok(())
    }

    fn duration(&self) -> f32 {
        match self.sampling_job.animation() {
            Some(a) => a.duration(),
            None => 0.0,
        }
    }

    fn set_progress(&mut self, progress: f32) {
        if self.is_loop {
            self.progress = progress.rem_euclid(self.duration());
        } else {
            self.progress = progress.clamp(0.0, self.duration());
        }
    }

    fn add_progress(&mut self, delta: f32) {
        self.progress += delta;
        // if self.is_loop {
        //     self.progress = self.progress.rem_euclid(self.duration());
        // } else {
        //     self.progress = self.progress.clamp(0.0, self.duration());
        // }
    }

    fn update(&mut self) -> XResult<()> {
        let animation = match self.sampling_job.animation() {
            Some(a) => a,
            None => return Ok(()),
        };

        let ratio = self.progress / animation.duration();
        self.sampling_job.set_ratio(ratio);
        self.sampling_job.run()?;
        self.l2m_job.run()?;
        soa_transforms_to_transforms(self.l2m_job.input().unwrap().borrow().as_slice(), &mut self.local_out);
        Ok(())
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
pub extern "C" fn skeletal_player_set_animation(
    playback: *mut SkeletalPlayer,
    animation_path: *const c_char,
    is_loop: bool,
) -> Return<()> {
    let res: XResult<()> = (|| {
        let playback = as_playback(playback)?;
        let mut animation = "";
        if !animation_path.is_null() {
            animation = unsafe { CStr::from_ptr(animation_path) }.to_str()?
        };
        playback.set_animation(animation, is_loop)?;
        Ok(())
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_player_duration(playback: *mut SkeletalPlayer) -> Return<f32> {
    let res: XResult<f32> = (|| as_playback(playback).map(|p| p.duration()))();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_player_set_progress(playback: *mut SkeletalPlayer, progress: f32) -> Return<()> {
    let res: XResult<()> = (|| as_playback(playback).map(|p| p.set_progress(progress)))();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_player_add_progress(playback: *mut SkeletalPlayer, delta: f32) -> Return<()> {
    let res: XResult<()> = (|| as_playback(playback).map(|p| p.add_progress(delta)))();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_player_update(playback: *mut SkeletalPlayer) -> Return<()> {
    let res: XResult<()> = (|| as_playback(playback).and_then(|p| p.update()))();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_player_local_rest_poses<'t>(playback: *mut SkeletalPlayer) -> Return<&'t [Transform3A]> {
    let res: XResult<&[Transform3A]> = (|| {
        let playback = as_playback(playback)?;
        let ptr = playback.local_rest_poses.as_ptr();
        let len = playback.local_rest_poses.len();
        Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
    })();
    Return::from_result_with(res, &[])
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
pub extern "C" fn skeletal_player_local_out<'t>(playback: *mut SkeletalPlayer) -> Return<&'t [Transform3A]> {
    let res: XResult<&[Transform3A]> = (|| {
        let playback = as_playback(playback)?;
        if playback.sampling_job.animation().is_some() {
            let ptr = playback.local_out.as_ptr();
            let len = playback.local_out.len();
            Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
        } else {
            let ptr = playback.local_rest_poses.as_ptr();
            let len = playback.local_rest_poses.len();
            Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
        }
    })();
    Return::from_result_with(res, &[])
}

#[no_mangle]
pub extern "C" fn skeletal_player_model_out<'t>(playback: *mut SkeletalPlayer) -> Return<&'t [Mat4]> {
    let res: XResult<&[Mat4]> = (|| {
        let playback = as_playback(playback)?;
        if playback.sampling_job.animation().is_some() {
            let model_out = playback.l2m_job.output().unwrap();
            let model_out = model_out.borrow();
            let ptr = model_out.as_ptr();
            let len = model_out.len();
            Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
        } else {
            let ptr = playback.model_rest_poses.as_ptr();
            let len = playback.model_rest_poses.len();
            Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
        }
    })();
    Return::from_result_with(res, &[])
}

fn as_playback<'t>(playback: *mut SkeletalPlayer) -> XResult<&'t mut SkeletalPlayer> {
    if playback.is_null() {
        return Err(xerror!(BadArgument, "playback=null"));
    }
    Ok(unsafe { &mut *playback })
}
