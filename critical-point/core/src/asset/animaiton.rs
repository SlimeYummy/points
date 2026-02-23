use ozz_animation_rs::{Animation, Archive, Skeleton};
use std::fs::File;
use std::rc::Rc;

use crate::animation::{HitMotion, RootMotion, WeaponMotion};
use crate::asset::loader::AssetLoader;
use crate::utils::{xfromf, Symbol, XResult};

impl AssetLoader {
    pub fn load_skeleton(&mut self, path_pattern: Symbol) -> XResult<Rc<Skeleton>> {
        if let Some(skeleton) = self.skeleton_cache.get(&path_pattern) {
            return Ok(skeleton.clone());
        }
        let path = format!("{}.ls-ozz", &path_pattern[0..path_pattern.len() - 2]);
        let data_buf = self.load_buffer(&path)?;
        let mut archive = Archive::from_vec(data_buf).map_err(xfromf!("path={:?}", &path))?;
        let skeleton = Rc::new(Skeleton::from_archive(&mut archive).map_err(xfromf!("path={:?}", &path))?);
        self.skeleton_cache.insert(path_pattern.clone(), skeleton.clone());
        Ok(skeleton)
    }

    pub fn load_animation(&mut self, path_pattern: Symbol) -> XResult<Rc<Animation>> {
        if let Some(animation) = self.animation_cache.get(&path_pattern) {
            return Ok(animation.clone());
        }
        let path = format!("{}.la-ozz", &path_pattern[0..path_pattern.len() - 2]);
        let data_buf = self.load_buffer(&path)?;
        let mut archive = Archive::from_vec(data_buf).map_err(xfromf!("path={:?}", &path))?;
        let animation = Rc::new(Animation::from_archive(&mut archive).map_err(xfromf!("path={:?}", &path))?);
        self.animation_cache.insert(path_pattern.clone(), animation.clone());
        Ok(animation)
    }

    pub fn load_root_motion(&mut self, path_pattern: Symbol) -> XResult<Rc<RootMotion>> {
        if let Some(root_motion) = self.root_motion_cache.get(&path_pattern) {
            return Ok(root_motion.clone());
        }
        let path = format!("{}.rm-ozz", &path_pattern[0..path_pattern.len() - 2]);
        let data_buf = self.load_buffer(&path)?;
        let mut archive = Archive::from_vec(data_buf).map_err(xfromf!("path={:?}", &path))?;
        let root_motion = Rc::new(RootMotion::from_archive(&mut archive).map_err(xfromf!("path={:?}", &path))?);
        self.root_motion_cache.insert(path_pattern.clone(), root_motion.clone());
        Ok(root_motion)
    }

    pub fn load_weapon_motion(&mut self, path_pattern: Symbol) -> XResult<Rc<WeaponMotion>> {
        if let Some(weapon_motion) = self.weapon_motion_cache.get(&path_pattern) {
            return Ok(weapon_motion.clone());
        }
        let path = format!("{}.wm-ozz", &path_pattern[0..path_pattern.len() - 2]);
        let data_buf = self.load_buffer(&path)?;
        let mut archive = Archive::from_vec(data_buf).map_err(xfromf!("path={:?}", &path))?;
        let weapon_motion = Rc::new(WeaponMotion::from_archive(&mut archive).map_err(xfromf!("path={:?}", &path))?);
        self.weapon_motion_cache
            .insert(path_pattern.clone(), weapon_motion.clone());
        Ok(weapon_motion)
    }

    pub fn load_hit_motion(&mut self, path_pattern: Symbol) -> XResult<Rc<HitMotion>> {
        if let Some(hit_motion) = self.hit_motion_cache.get(&path_pattern) {
            return Ok(hit_motion.clone());
        }
        let path = format!("{}.hm-json", &path_pattern[0..path_pattern.len() - 2]);
        let data_buf = self.load_buffer(&path)?;
        let hit_motion =
            Rc::new(HitMotion::from_json_bytes(&data_buf, Some(path.as_str())).map_err(xfromf!("path={:?}", &path))?);
        self.hit_motion_cache.insert(path_pattern.clone(), hit_motion.clone());
        Ok(hit_motion)
    }
}
