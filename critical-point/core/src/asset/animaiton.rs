use ozz_animation_rs::{Animation, Archive, Skeleton};
use std::rc::Rc;

use crate::animation::{HitMotion, JointWeightsTable, RootMotion, WeaponControl};
use crate::asset::loader::AssetLoader;
use crate::utils::{Symbol, XResult, xfromf};

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
        let root_motion =
            Rc::new(RootMotion::from_archive_with_path(&mut archive, &path).map_err(xfromf!("path={:?}", &path))?);
        self.root_motion_cache.insert(path_pattern.clone(), root_motion.clone());
        Ok(root_motion)
    }

    pub fn load_weapon_control(&mut self, path_pattern: Symbol) -> XResult<Rc<WeaponControl>> {
        if let Some(weapon_control) = self.weapon_control_cache.get(&path_pattern) {
            return Ok(weapon_control.clone());
        }
        let path = format!("{}.wc-ozz", &path_pattern[0..path_pattern.len() - 2]);
        let data_buf = self.load_buffer(&path)?;
        let mut archive = Archive::from_vec(data_buf).map_err(xfromf!("path={:?}", &path))?;
        let weapon_control = Rc::new(WeaponControl::from_archive(&mut archive).map_err(xfromf!("path={:?}", &path))?);
        self.weapon_control_cache
            .insert(path_pattern.clone(), weapon_control.clone());
        Ok(weapon_control)
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

    pub fn load_joint_weights_table_with(
        &mut self,
        path_pattern: Symbol,
        skeleton: &Skeleton,
    ) -> XResult<Rc<JointWeightsTable>> {
        if let Some(joint_weights_table) = self.joint_weights_table_cache.get(&path_pattern) {
            return Ok(joint_weights_table.clone());
        }
        let path = format!("{}.lw-json", &path_pattern[0..path_pattern.len() - 2]);
        let data_buf = self.load_buffer(&path)?;
        let joint_weights_table =
            Rc::new(JointWeightsTable::from_json_bytes_rc(&data_buf, skeleton).map_err(xfromf!("path={:?}", &path))?);
        self.joint_weights_table_cache
            .insert(path_pattern.clone(), joint_weights_table.clone());
        Ok(joint_weights_table)
    }
}
