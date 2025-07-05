use ozz_animation_rs::{Animation, Archive, Skeleton};
use std::rc::Rc;

use crate::animation::RootMotionTrack;
use crate::asset::loader::AssetLoader;
use crate::utils::{xfromf, Symbol, XResult};

impl AssetLoader {
    pub fn load_skeleton(&mut self, path_prefix: &Symbol) -> XResult<Rc<Skeleton>> {
        if let Some(skeleton) = self.skeleton_cache.get(path_prefix) {
            return Ok(skeleton.clone());
        }
        let path = format!("{}.logic-skel.ozz", path_prefix);
        let data_buf = self.load_buffer(&path)?;
        let mut archive = Archive::from_vec(data_buf).map_err(xfromf!("path={:?}", &path))?;
        let skeleton = Rc::new(Skeleton::from_archive(&mut archive).map_err(xfromf!("path={:?}", &path))?);
        self.skeleton_cache.insert(path_prefix.clone(), skeleton.clone());
        Ok(skeleton)
    }

    pub fn load_animation(&mut self, path_prefix: &Symbol) -> XResult<Rc<Animation>> {
        if let Some(animation) = self.animation_cache.get(path_prefix) {
            return Ok(animation.clone());
        }
        let path = format!("{}.logic-anim.ozz", path_prefix);
        let data_buf = self.load_buffer(&path)?;
        let mut archive = Archive::from_vec(data_buf).map_err(xfromf!("path={:?}", &path))?;
        let animation = Rc::new(Animation::from_archive(&mut archive).map_err(xfromf!("path={:?}", &path))?);
        self.animation_cache.insert(path_prefix.clone(), animation.clone());
        Ok(animation)
    }

    pub fn load_root_motion(&mut self, path_prefix: &Symbol) -> XResult<Rc<RootMotionTrack>> {
        if let Some(root_motion) = self.root_motion_cache.get(path_prefix) {
            return Ok(root_motion.clone());
        }
        let path = format!("{}.logic-moti.ozz", path_prefix);
        let data_buf = self.load_buffer(&path)?;
        let mut archive = Archive::from_vec(data_buf).map_err(xfromf!("path={:?}", &path))?;
        let root_motion = Rc::new(RootMotionTrack::from_archive(&mut archive).map_err(xfromf!("path={:?}", &path))?);
        self.root_motion_cache.insert(path_prefix.clone(), root_motion.clone());
        Ok(root_motion)
    }
}
