use glam::{Quat, Vec3A};
use jolt_physics_rs::{JRef, Shape};

use crate::asset::loader::AssetLoader;
use crate::asset::shape::AssetShape;
use crate::utils::{default_position, default_rotation, xerr, xerrf, Symbol, XResult};

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
struct AssetCharacterPhysics {
    bounding: AssetShape,
    parts: Vec<Symbol>,
    shapes: Vec<AssetShape>,
    bodies: Vec<AssetCharacterBody>,
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
struct AssetCharacterBody {
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

#[derive(Debug)]
pub struct LoadedCharacterPhysics {
    pub bounding: JRef<Shape>,
    pub parts: Vec<Symbol>,
    pub bodies: Vec<LoadedCharacterBody>,
}

#[derive(Debug)]
pub struct LoadedCharacterBody {
    pub shape: JRef<Shape>,
    pub position: Vec3A,
    pub rotation: Quat,
    pub part: Symbol,
    pub joint: Symbol,
    pub ratio: f32,
    pub joint2: Symbol,
}

impl AssetLoader {
    pub fn load_character_physics(&mut self, path_pattern: Symbol) -> XResult<LoadedCharacterPhysics> {
        let rkyv_path = format!("{}.cp-rkyv", &path_pattern[0..path_pattern.len() - 2]);
        if let Ok(buf) = self.load_buffer(&rkyv_path) {
            let asset = unsafe { rkyv::access_unchecked::<ArchivedAssetCharacterPhysics>(&buf) };
            from_archived_asset(&rkyv_path, asset)
        }
        else {
            let json_path = format!("{}.cp-json", &path_pattern[0..path_pattern.len() - 2]);
            let asset = self.load_json::<AssetCharacterPhysics, _>(&json_path)?;
            from_asset(&json_path, asset)
        }
    }
}

fn from_asset(path: &str, mut asset: AssetCharacterPhysics) -> XResult<LoadedCharacterPhysics> {
    let mut loaded = LoadedCharacterPhysics {
        bounding: asset.bounding.create_physics()?,
        parts: asset.parts,
        bodies: Vec::with_capacity(asset.bodies.len()),
    };

    let mut jolt_shapes = Vec::with_capacity(asset.shapes.len());
    for shape in &asset.shapes {
        jolt_shapes.push(shape.create_physics()?);
    }

    for asset_box in asset.bodies.drain(..) {
        let jolt_shape = jolt_shapes
            .get(asset_box.shape_index as usize)
            .ok_or_else(|| xerrf!(BadAsset; "path={}, shape_index={}", path, asset_box.shape_index))?;
        loaded.bodies.push(LoadedCharacterBody {
            shape: jolt_shape.clone(),
            position: asset_box.position,
            rotation: asset_box.rotation,
            part: asset_box.part,
            joint: asset_box.joint,
            ratio: asset_box.ratio,
            joint2: asset_box.joint2,
        });
    }
    Ok(loaded)
}

fn from_archived_asset(path: &str, asset: &ArchivedAssetCharacterPhysics) -> XResult<LoadedCharacterPhysics> {
    let bounding = rkyv::deserialize::<AssetShape, rkyv::rancor::Error>(&asset.bounding).map_err(|_| xerr!(Rkyv))?;

    let mut parts = Vec::with_capacity(asset.parts.len());
    for part in asset.parts.iter() {
        parts.push(Symbol::new(part.as_str())?)
    }

    let mut loaded = LoadedCharacterPhysics {
        bounding: bounding.create_physics()?,
        parts,
        bodies: Vec::with_capacity(asset.bodies.len()),
    };

    let mut jolt_shapes = Vec::with_capacity(asset.shapes.len());
    for shape in asset.shapes.iter() {
        let shape = rkyv::deserialize::<AssetShape, rkyv::rancor::Error>(shape).map_err(|_| xerr!(Rkyv))?;
        jolt_shapes.push(shape.create_physics()?);
    }

    for asset_box in asset.bodies.iter() {
        let shape_index: u32 = asset_box.shape_index.into();
        let jolt_shape = jolt_shapes
            .get(shape_index as usize)
            .ok_or_else(|| xerrf!(BadAsset; "path={}, shape_index={}", path, shape_index))?;
        loaded.bodies.push(LoadedCharacterBody {
            shape: jolt_shape.clone(),
            position: asset_box.position,
            rotation: asset_box.rotation,
            part: Symbol::new(asset_box.part.as_str())?,
            joint: Symbol::new(asset_box.joint.as_str())?,
            ratio: asset_box.ratio.into(),
            joint2: Symbol::new(asset_box.joint2.as_str())?,
        });
    }
    Ok(loaded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_ASSET_PATH;
    use crate::utils::sb;

    #[test]
    fn test_load_character_physics() {
        let mut loader = AssetLoader::new(TEST_ASSET_PATH).unwrap();
        let chara_phy = loader.load_character_physics(sb!("Girl.*")).unwrap();
        assert!(chara_phy.parts.len() > 0);
        assert!(chara_phy.bodies.len() > 0);
    }
}
