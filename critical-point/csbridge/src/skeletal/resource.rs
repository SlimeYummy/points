// #![allow(improper_ctypes_definitions)]
#![allow(static_mut_refs)]

use critical_point_core::animation::WeaponMotion;
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
    weapon_motions: HashMap<Symbol, Arc<WeaponMotion>>,
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
    fn load_weapon_motion<P: AsRef<Path>>(path: P, anim: Symbol) -> XResult<Arc<WeaponMotion>> {
        let mut path_buf = path.as_ref().to_path_buf();
        path_buf.push(&anim[0..anim.len() - 2]);
        path_buf.set_extension("wm-ozz");
        let weapon_motions = Arc::new(WeaponMotion::from_path(path_buf)?);
        Ok(weapon_motions)
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
    fn set_weapon_tracks(&mut self, anim: Symbol, weapon_motions: Arc<WeaponMotion>) {
        self.weapon_motions.insert(anim, weapon_motions);
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
    fn has_weapon_tracks(&self, anim: Symbol) -> bool {
        self.weapon_motions.contains_key(&anim)
    }

    #[inline]
    fn skeleton_count(&self) -> usize {
        self.skeletons.len()
    }

    #[inline]
    fn animation_count(&self) -> usize {
        self.animations.len()
    }

    #[inline]
    fn weapon_tracks_count(&self) -> usize {
        self.weapon_motions.len()
    }

    pub fn clear_unused(&mut self) {
        self.skeletons.retain(|_, skeleton| Arc::strong_count(skeleton) > 1);
        self.animations.retain(|_, animation| Arc::strong_count(animation) > 1);
        self.weapon_motions
            .retain(|_, weapon_motions| Arc::strong_count(weapon_motions) > 1);
    }

    pub fn clear_all(&mut self) {
        self.skeletons.clear();
        self.animations.clear();
        self.weapon_motions.clear();
    }
}

pub static SKELETAL_RESOURCE: LazyLock<RwLock<SkeletalResource>> = LazyLock::new(|| {
    RwLock::new(SkeletalResource {
        skeletons: HashMap::with_capacity(16),
        animations: HashMap::with_capacity(256),
        weapon_motions: HashMap::with_capacity(192),
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

pub(super) fn load_weapon_motion(anim: Symbol) -> XResult<Arc<WeaponMotion>> {
    let cached = SKELETAL_RESOURCE.read().unwrap().weapon_motions.get(&anim).cloned();
    match cached {
        Some(weapon_motions) => Ok(weapon_motions),
        None => {
            let weapon_motions = SkeletalResource::load_weapon_motion(unsafe { &ENV_PATH.asset_path }, anim)?;
            SKELETAL_RESOURCE
                .write()
                .unwrap()
                .set_weapon_tracks(anim, weapon_motions.clone());
            Ok(weapon_motions)
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
pub extern "C" fn skeletal_resource_load_weapon_tracks(anim: Symbol) -> Return<()> {
    let res: XResult<()> = (|| {
        if SKELETAL_RESOURCE.read().unwrap().has_weapon_tracks(anim) {
            return Ok(());
        }
        let weapon_motions = SkeletalResource::load_weapon_motion(unsafe { &ENV_PATH.asset_path }, anim)?;
        SKELETAL_RESOURCE
            .write()
            .unwrap()
            .set_weapon_tracks(anim, weapon_motions);
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
    weapon_motions: *const Symbol,
    weapon_tracks_len: u32,
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
        let mut weapon_tracks_paths = Vec::with_capacity(weapon_tracks_len as usize);
        for i in 0..weapon_tracks_len {
            weapon_tracks_paths.push(unsafe { *weapon_motions.offset(i as isize) });
        }

        {
            let resource = SKELETAL_RESOURCE.read().unwrap();
            skel_paths.retain(|skel_path| !resource.has_skeleton(*skel_path));
            anim_paths.retain(|anim_path| !resource.has_animation(*anim_path));
            weapon_tracks_paths.retain(|weapon_track_path| !resource.has_weapon_tracks(*weapon_track_path));
        }

        let mut skeleton_vec = Vec::<Arc<Skeleton>>::with_capacity(skel_paths.len());
        for skel_path in &skel_paths {
            let skeleton = SkeletalResource::load_skeleton(unsafe { &ENV_PATH.asset_path }, *skel_path)?;
            skeleton_vec.push(skeleton);
        }
        let mut animation_vec = Vec::<Arc<Animation>>::with_capacity(anim_paths.len());
        for anim_path in &anim_paths {
            let animation = SkeletalResource::load_animation(unsafe { &ENV_PATH.asset_path }, *anim_path)?;
            animation_vec.push(animation);
        }
        let mut weapon_tracks_vec = Vec::<Arc<WeaponMotion>>::with_capacity(weapon_tracks_paths.len());
        for weapon_track_path in &weapon_tracks_paths {
            let weapon_motions =
                SkeletalResource::load_weapon_motion(unsafe { &ENV_PATH.asset_path }, *weapon_track_path)?;
            weapon_tracks_vec.push(weapon_motions);
        }

        {
            let mut resource = SKELETAL_RESOURCE.write().unwrap();
            for (skel_path, skeleton) in skel_paths.iter().zip(skeleton_vec) {
                resource.set_skeleton(*skel_path, skeleton);
            }
            for (anim_path, animation) in anim_paths.iter().zip(animation_vec) {
                resource.set_animation(*anim_path, animation);
            }
            for (weapon_track_path, weapon_motions) in weapon_tracks_paths.iter().zip(weapon_tracks_vec) {
                resource.set_weapon_tracks(*weapon_track_path, weapon_motions);
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
pub extern "C" fn skeletal_resource_weapon_tracks_count() -> u32 {
    SKELETAL_RESOURCE.read().unwrap().weapon_tracks_count() as u32
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
