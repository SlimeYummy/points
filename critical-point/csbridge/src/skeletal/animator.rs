#![allow(improper_ctypes_definitions)]

use glam_ext::{Mat4, Transform3A};
use libc::c_char;
use ozz_animation_rs::{ozz_rc_buf, Animation, LocalToModelJob, SamplingContext, SamplingJob, Skeleton, SoaTransform};
use std::collections::HashMap;
use std::ffi::CStr;
use std::path::Path;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;

use cirtical_point_core::animation::{
    rest_poses_to_local_transforms, rest_poses_to_model_matrices, soa_transforms_to_transforms, SkeletalAnimator,
    SkeletonJointMeta, SkeletonMeta,
};
use cirtical_point_core::consts::MAX_ACTION_ANIMATION;
use cirtical_point_core::logic::StateActionAny;
use cirtical_point_core::utils::{xerror, Symbol, XResult};

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
        println!("AnimatorWrapper::drop()");
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
    states: &[Box<dyn StateActionAny>],
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
    states: &[Box<dyn StateActionAny>],
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
    skeletons: HashMap<Symbol, Arc<Skeleton>>,
    animations: HashMap<Symbol, Arc<Animation>>,
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

    fn add_animation<P: AsRef<Path>>(&mut self, logic_path: Symbol, view_path: P) -> XResult<()> {
        let animation = Rc::new(Animation::from_path(view_path)?);
        self.animations.insert(logic_path, animation);
        Ok(())
    }

    fn remove_animation(&mut self, logic_path: Symbol) {
        self.animations.remove(&logic_path);
    }

    fn has_animation(&self, logic_path: Symbol) -> bool {
        self.animations.contains_key(&logic_path)
    }
}

#[cfg(feature = "debug-print")]
impl Drop for SkeletalResource {
    fn drop(&mut self) {
        println!("SkeletalResource::drop()");
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
    logic_path: Symbol,
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
    logic_path: Symbol,
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
    logic_path: Symbol,
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
