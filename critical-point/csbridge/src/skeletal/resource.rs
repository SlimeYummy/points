// #![allow(improper_ctypes_definitions)]
#![allow(static_mut_refs)]

use ozz_animation_rs::{Animation, Skeleton};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, LazyLock, RwLock};

use critical_point_core::engine::ENV_PATH;
use critical_point_core::utils::{Symbol, XResult};

use crate::utils::Return;

pub struct SkeletalResource {
    skeletons: HashMap<Symbol, Arc<Skeleton>>,
    animations: HashMap<Symbol, Arc<Animation>>,
}

impl SkeletalResource {
    #[inline]
    fn load_skeleton<P: AsRef<Path>>(path: P, skel: Symbol) -> XResult<Arc<Skeleton>> {
        let mut path_buf = path.as_ref().to_path_buf();
        path_buf.push(&skel[0..skel.len() - 2]);
        path_buf.set_extension("vs-ozz");
        let skeleton = Arc::new(Skeleton::from_path(path_buf)?);
        Ok(skeleton)
    }

    #[inline]
    fn load_animation<P: AsRef<Path>>(path: P, anim: Symbol) -> XResult<Arc<Animation>> {
        let mut path_buf = path.as_ref().to_path_buf();
        path_buf.push(&anim[0..anim.len() - 2]);
        path_buf.set_extension("va-ozz");
        let animation = Arc::new(Animation::from_path(path_buf)?);
        Ok(animation)
    }

    #[inline]
    fn set_skeleton(&mut self, skel: Symbol, skeleton: Arc<Skeleton>) {
        self.skeletons.insert(skel, skeleton);
    }

    #[inline]
    fn set_animation(&mut self, anim: Symbol, animation: Arc<Animation>) {
        self.animations.insert(anim, animation);
    }

    #[inline]
    fn has_skeleton(&self, skel: Symbol) -> bool {
        self.skeletons.contains_key(&skel)
    }

    #[inline]
    fn has_animation(&self, anim: Symbol) -> bool {
        self.animations.contains_key(&anim)
    }

    #[inline]
    fn skeleton_count(&self) -> usize {
        self.skeletons.len()
    }

    #[inline]
    fn animation_count(&self) -> usize {
        self.animations.len()
    }

    pub fn clear_unused(&mut self) {
        self.skeletons.retain(|_, skeleton| Arc::strong_count(skeleton) > 1);
        self.animations.retain(|_, animation| Arc::strong_count(animation) > 1);
    }

    pub fn clear_all(&mut self) {
        self.skeletons.clear();
        self.animations.clear();
    }
}

pub static SKELETAL_RESOURCE: LazyLock<RwLock<SkeletalResource>> = LazyLock::new(|| {
    RwLock::new(SkeletalResource {
        skeletons: HashMap::with_capacity(16),
        animations: HashMap::with_capacity(256),
    })
});

pub(super) fn load_skeleton(skel: Symbol) -> XResult<Arc<Skeleton>> {
    let cached = SKELETAL_RESOURCE.read().unwrap().skeletons.get(&skel).cloned();
    match cached {
        Some(skeleton) => Ok(skeleton),
        None => {
            let skeleton = SkeletalResource::load_skeleton(unsafe { &ENV_PATH.asset_path }, skel)?;
            SKELETAL_RESOURCE.write().unwrap().set_skeleton(skel, skeleton.clone());
            Ok(skeleton)
        }
    }
}

pub(super) fn load_animation(anim: Symbol) -> XResult<Arc<Animation>> {
    let cached = SKELETAL_RESOURCE.read().unwrap().animations.get(&anim).cloned();
    match cached {
        Some(animation) => Ok(animation),
        None => {
            let animation = SkeletalResource::load_animation(unsafe { &ENV_PATH.asset_path }, anim)?;
            SKELETAL_RESOURCE
                .write()
                .unwrap()
                .set_animation(anim, animation.clone());
            Ok(animation)
        }
    }
}

#[no_mangle]
pub extern "C" fn skeletal_resource_load_skeleton(skel: Symbol) -> Return<()> {
    let res: XResult<()> = (|| {
        if SKELETAL_RESOURCE.read().unwrap().has_skeleton(skel) {
            return Ok(());
        }
        let skeleton = SkeletalResource::load_skeleton(unsafe { &ENV_PATH.asset_path }, skel)?;
        SKELETAL_RESOURCE.write().unwrap().set_skeleton(skel, skeleton);
        Ok(())
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_resource_load_animation(anim: Symbol) -> Return<()> {
    let res: XResult<()> = (|| {
        if SKELETAL_RESOURCE.read().unwrap().has_animation(anim) {
            return Ok(());
        }
        let animation = SkeletalResource::load_animation(unsafe { &ENV_PATH.asset_path }, anim)?;
        SKELETAL_RESOURCE.write().unwrap().set_animation(anim, animation);
        Ok(())
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_resource_load(
    skels: *const Symbol,
    skel_len: u32,
    anims: *const Symbol,
    anim_len: u32,
) -> Return<()> {
    let res: XResult<()> = (|| {
        let mut skel_paths = Vec::with_capacity(skel_len as usize);
        for i in 0..skel_len {
            skel_paths.push(unsafe { *skels.offset(i as isize) });
        }
        let mut anim_paths = Vec::with_capacity(anim_len as usize);
        for i in 0..anim_len {
            anim_paths.push(unsafe { *anims.offset(i as isize) });
        }

        {
            let resource = SKELETAL_RESOURCE.read().unwrap();
            skel_paths.retain(|skel_path| !resource.has_skeleton(*skel_path));
            anim_paths.retain(|anim_path| !resource.has_animation(*anim_path));
        }

        let mut skeletons = Vec::<Arc<Skeleton>>::with_capacity(skel_paths.len());
        for skel_path in &skel_paths {
            let skeleton = SkeletalResource::load_skeleton(unsafe { &ENV_PATH.asset_path }, *skel_path)?;
            skeletons.push(skeleton);
        }
        let mut animations = Vec::<Arc<Animation>>::with_capacity(anim_paths.len());
        for anim_path in &anim_paths {
            let animation = SkeletalResource::load_animation(unsafe { &ENV_PATH.asset_path }, *anim_path)?;
            animations.push(animation);
        }

        {
            let mut resource = SKELETAL_RESOURCE.write().unwrap();
            for (skel_path, skeleton) in skel_paths.iter().zip(skeletons) {
                resource.set_skeleton(*skel_path, skeleton);
            }
            for (anim_path, animation) in anim_paths.iter().zip(animations) {
                resource.set_animation(*anim_path, animation);
            }
        }
        Ok(())
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn skeletal_resource_skeleton_count() -> u32 {
    SKELETAL_RESOURCE.read().unwrap().skeleton_count() as u32
}

#[no_mangle]
pub extern "C" fn skeletal_resource_animation_count() -> u32 {
    SKELETAL_RESOURCE.read().unwrap().animation_count() as u32
}

#[no_mangle]
pub extern "C" fn skeletal_resource_clear_unused() {
    SKELETAL_RESOURCE.write().unwrap().clear_unused();
}

#[no_mangle]
pub extern "C" fn skeletal_resource_clear_all() {
    SKELETAL_RESOURCE.write().unwrap().clear_all();
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn test_a() {
//         // SkeletalResource::load_skeleton("D:\\project\\points\\test-tmp\\test-asset", Symbol::new("girl.*").unwrap()).unwrap();
//         let a = skeletal_animator_create(Symbol::new("girl.*").unwrap());
//         assert!(a.value.is_null());
//     }
// }
