use ahash::HashMapExt;
use jolt_physics_rs::{JRef, Shape};
use ozz_animation_rs::{Animation, Skeleton};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::utils::{xfromf, xresf, DtHashMap, Symbol, SymbolMap, XResult};

pub struct AssetLoader {
    asset_path: PathBuf,

    pub(super) shape_mesh_cache: DtHashMap<Symbol, JRef<Shape>>,
    pub(super) shape_heigh_tfield_cache: DtHashMap<Symbol, JRef<Shape>>,

    pub(super) skeleton_cache: SymbolMap<Rc<Skeleton>>,
    pub(super) animation_cache: SymbolMap<Rc<Animation>>,
}

impl AssetLoader {
    pub fn new<P: AsRef<Path>>(asset_path: P) -> XResult<AssetLoader> {
        if !asset_path.as_ref().is_dir() {
            return xresf!(BadArgument; "asset_path={:?}", asset_path.as_ref());
        }

        return Ok(AssetLoader {
            asset_path: asset_path.as_ref().to_path_buf(),

            shape_mesh_cache: DtHashMap::with_capacity(64),
            shape_heigh_tfield_cache: DtHashMap::with_capacity(16),

            skeleton_cache: SymbolMap::with_capacity(64),
            animation_cache: SymbolMap::with_capacity(512),
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
