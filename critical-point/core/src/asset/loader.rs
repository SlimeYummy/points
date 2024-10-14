use ahash::HashMapExt;
use jolt_physics_rs::{BodyID, BodyInterface, BodySettings, RefShape};
use ozz_animation_rs::{Animation, Skeleton};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::asset::shape::ShapeKey;
use crate::utils::{DtHashMap, NumID, SymbolMap, XError, XResult};

pub struct AssetLoader {
    asset_path: PathBuf,

    pub(super) body_itf: BodyInterface,
    pub(super) shape_cache: DtHashMap<ShapeKey, RefShape>,
    pub(super) shape_ex_cache: DtHashMap<NumID, RefShape>,

    pub(super) skeleton_cache: SymbolMap<Rc<Skeleton>>,
    pub(super) animation_cache: SymbolMap<Rc<Animation>>,
}

impl AssetLoader {
    pub fn new<P: AsRef<Path>>(body_itf: BodyInterface, asset_path: P) -> XResult<AssetLoader> {
        let path = asset_path.as_ref();
        if !path.is_dir() {
            return Err(XError::bad_argument("AssetLoader::new() asset_path"));
        }

        return Ok(AssetLoader {
            asset_path: asset_path.as_ref().to_path_buf(),

            body_itf,
            shape_cache: DtHashMap::with_capacity(512),
            shape_ex_cache: DtHashMap::with_capacity(512),

            skeleton_cache: SymbolMap::with_capacity(64),
            animation_cache: SymbolMap::with_capacity(512),
        });
    }

    pub(super) fn load_buffer(&mut self, path: &str) -> XResult<Vec<u8>> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(false)
            .create_new(false)
            .open(self.asset_path.join(path))?;
        let mut data_buf = Vec::new();
        file.read_to_end(&mut data_buf)?;
        Ok(data_buf)
    }

    pub(super) fn load_json<T: serde::de::DeserializeOwned>(&mut self, json_path: &str) -> XResult<T> {
        let data_buf = self.load_buffer(json_path)?;
        let asset = serde_json::from_slice(&data_buf)?;
        Ok(asset)
    }

    pub(super) fn create_body(&mut self, settings: &BodySettings) -> XResult<BodyID> {
        match self.body_itf.create_body(settings) {
            Some(body) => Ok(body),
            None => Err(XError::PhysicBodyFailed),
        }
    }
}
