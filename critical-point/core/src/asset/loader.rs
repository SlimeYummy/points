// use jolt_physics_rs::{JRef, Shape};
use ozz_animation_rs::{Animation, Skeleton};
use rustc_hash::FxBuildHasher;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::animation::{HitMotion, RootMotion, WeaponMotion};
use crate::utils::{xfromf, xresf, DtHashMap, Symbol, XResult};

pub struct AssetLoader {
    asset_path: PathBuf,

    // pub(super) shape_mesh_cache: DtHashMap<Symbol, JRef<Shape>>,
    // pub(super) shape_heigh_tfield_cache: DtHashMap<Symbol, JRef<Shape>>,
    pub(super) skeleton_cache: DtHashMap<Symbol, Rc<Skeleton>>,
    pub(super) animation_cache: DtHashMap<Symbol, Rc<Animation>>,
    pub(super) root_motion_cache: DtHashMap<Symbol, Rc<RootMotion>>,
    pub(super) weapon_motion_cache: DtHashMap<Symbol, Rc<WeaponMotion>>,
    pub(super) hit_motion_cache: DtHashMap<Symbol, Rc<HitMotion>>,
}

#[cfg(feature = "debug-print")]
impl Drop for AssetLoader {
    fn drop(&mut self) {
        log::debug!("AssetLoader::drop()");
    }
}

impl AssetLoader {
    pub fn new<P: AsRef<Path>>(asset_path: P) -> XResult<AssetLoader> {
        if !asset_path.as_ref().is_dir() {
            return xresf!(BadArgument; "asset_path={:?}", asset_path.as_ref());
        }

        return Ok(AssetLoader {
            asset_path: asset_path.as_ref().to_path_buf(),

            // shape_mesh_cache: DtHashMap::with_capacity_and_hasher(64, FxBuildHasher),
            // shape_heigh_tfield_cache: DtHashMap::with_capacity_and_hasher(16, FxBuildHasher),
            skeleton_cache: DtHashMap::with_capacity_and_hasher(64, FxBuildHasher),
            animation_cache: DtHashMap::with_capacity_and_hasher(512, FxBuildHasher),
            root_motion_cache: DtHashMap::with_capacity_and_hasher(384, FxBuildHasher),
            weapon_motion_cache: DtHashMap::with_capacity_and_hasher(384, FxBuildHasher),
            hit_motion_cache: DtHashMap::with_capacity_and_hasher(384, FxBuildHasher),
        });
    }

    pub(super) fn load_buffer<P: AsRef<Path>>(&mut self, path: P) -> XResult<Vec<u8>> {
        let full_path = self.asset_path.join(path);
        let mut file = OpenOptions::new()
            .read(true)
            .write(false)
            .create_new(false)
            .open(&full_path)
            .map_err(xfromf!("full_path={:?}", &full_path))?;
        let mut data_buf = Vec::new();
        file.read_to_end(&mut data_buf)
            .map_err(xfromf!("full_path={:?}", &full_path))?;
        Ok(data_buf)
    }

    pub(super) fn load_json<T: serde::de::DeserializeOwned, P: AsRef<Path>>(&mut self, json_path: P) -> XResult<T> {
        let data_buf = self.load_buffer(&json_path)?;
        let asset = serde_json::from_slice(&data_buf).map_err(xfromf!("json_path={:?}", json_path.as_ref()))?;
        Ok(asset)
    }

    // pub(super) fn create_body(&mut self, settings: &BodyCreationSettings) -> XResult<BodyID> {
    //     Ok(self.body_itf.create_body(settings)?)
    // }
}
