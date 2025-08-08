use glam::{Quat, Vec3A};
use jolt_physics_rs::{JRef, Shape};

use crate::asset::loader::AssetLoader;
use crate::asset::shape::{default_position, default_rotation, AssetShape};
use crate::utils::{xerrf, Symbol, XResult};

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
struct AssetTargetBox {
    parts: Vec<Symbol>,
    shapes: Vec<AssetShape>,
    bindings: Vec<AssetTargetBinding>,
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
struct AssetTargetBinding {
    shape_index: u32,
    #[serde(default = "default_position")]
    position: Vec3A,
    #[serde(default = "default_rotation")]
    rotation: Quat,
    part: Symbol,
    joint: Symbol,
    ratio: f32,
    #[serde(default)]
    joint2: Symbol,
}

#[derive(Debug, Default)]
pub struct LoadedTargetBox {
    pub parts: Vec<Symbol>,
    pub bindings: Vec<LoadedTargetBinding>,
}

#[derive(Debug)]
pub struct LoadedTargetBinding {
    pub shape: JRef<Shape>,
    pub position: Vec3A,
    pub rotation: Quat,
    pub part: Symbol,
    pub joint: Symbol,
    pub ratio: f32,
    pub joint2: Symbol,
}

impl AssetLoader {
    pub fn load_target_box(&mut self, path: &Symbol) -> XResult<LoadedTargetBox> {
        let mut asset_zone = self.load_json::<AssetTargetBox, _>(path.as_str())?;

        let mut loaded = LoadedTargetBox::default();
        loaded.parts = asset_zone.parts;

        let mut jolt_shapes = Vec::with_capacity(asset_zone.shapes.len());
        for shape in &asset_zone.shapes {
            jolt_shapes.push(self.load_shape(shape)?);
        }

        loaded.bindings = Vec::with_capacity(asset_zone.bindings.len());
        for asset_binding in asset_zone.bindings.drain(..) {
            let jolt_shape = jolt_shapes
                .get(asset_binding.shape_index as usize)
                .ok_or_else(|| xerrf!(BadAsset; "path={}, shape_index={}", path, asset_binding.shape_index))?;
            loaded.bindings.push(LoadedTargetBinding {
                shape: jolt_shape.clone(),
                position: asset_binding.position,
                rotation: asset_binding.rotation,
                part: asset_binding.part,
                joint: asset_binding.joint,
                ratio: asset_binding.ratio,
                joint2: asset_binding.joint2,
            });
        }
        Ok(loaded)
    }
}
