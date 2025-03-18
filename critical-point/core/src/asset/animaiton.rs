use ozz_animation_rs::{Animation, Archive, Skeleton};
use std::rc::Rc;

use crate::asset::loader::AssetLoader;
use crate::utils::{xfromf, Symbol, XResult};

impl AssetLoader {
    pub fn load_skeleton(&mut self, path: &Symbol) -> XResult<Rc<Skeleton>> {
        if let Some(skeleton) = self.skeleton_cache.get(path) {
            return Ok(skeleton.clone());
        }
        let data_buf = self.load_buffer(path.as_str())?;
        let mut archive = Archive::from_vec(data_buf).map_err(xfromf!("path={:?}", path))?;
        let skeleton = Rc::new(Skeleton::from_archive(&mut archive).map_err(xfromf!("path={:?}", path))?);
        self.skeleton_cache.insert(path.clone(), skeleton.clone());
        Ok(skeleton)
    }

    pub fn load_animation(&mut self, path: &Symbol) -> XResult<Rc<Animation>> {
        if let Some(animation) = self.animation_cache.get(path) {
            return Ok(animation.clone());
        }
        let data_buf = self.load_buffer(path.as_str())?;
        let mut archive = Archive::from_vec(data_buf).map_err(xfromf!("path={:?}", path))?;
        let animation = Rc::new(Animation::from_archive(&mut archive).map_err(xfromf!("path={:?}", path))?);
        self.animation_cache.insert(path.clone(), animation.clone());
        Ok(animation)
    }
}
