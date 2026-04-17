use glam::{Quat, Vec3A};
use jolt_physics_rs::{self as jolt, JRef, Shape, StaticCompoundShapeSettings, SubShapeSettings};

use crate::asset::AssetIndxedCompoundShape;
use crate::asset::loader::AssetLoader;
use crate::asset::shape::AssetShape;
use crate::utils::{Symbol, XResult, default_position, default_rotation, xerr, xerrf, xfrom};

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
pub struct AssetZonePhysics {
    shapes: Vec<AssetShape>,
    #[serde(default)]
    compound_shapes: Vec<AssetIndxedCompoundShape>,
    bodies: Vec<AssetZoneBody>,
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
pub struct AssetZoneBody {
    shape_index: u32,
    #[serde(default = "default_position")]
    position: Vec3A,
    #[serde(default = "default_rotation")]
    rotation: Quat,
}

#[derive(Debug, Default)]
pub struct LoadedZonePhysics {
    pub bodies: Vec<LoadedZoneBody>,
}

#[derive(Debug)]
pub struct LoadedZoneBody {
    pub shape: JRef<Shape>,
    pub position: Vec3A,
    pub rotation: Quat,
}

impl AssetLoader {
    pub fn load_zone_physics(&mut self, path_pattern: Symbol) -> XResult<LoadedZonePhysics> {
        let rkyv_path = format!("{}.zp-rkyv", &path_pattern[0..path_pattern.len() - 2]);
        if let Ok(buf) = self.load_buffer(&rkyv_path) {
            let asset = unsafe { rkyv::access_unchecked::<ArchivedAssetZonePhysics>(&buf) };
            from_archived_asset(&rkyv_path, asset)
        }
        else {
            let json_path = format!("{}.zp-json", &path_pattern[0..path_pattern.len() - 2]);
            let asset = self.load_json::<AssetZonePhysics, _>(&json_path)?;
            from_asset(&json_path, asset)
        }
    }
}

pub fn from_asset(path: &str, asset: AssetZonePhysics) -> XResult<LoadedZonePhysics> {
    let mut jolt_shapes = Vec::with_capacity(asset.shapes.len() + asset.compound_shapes.len());
    for shape in &asset.shapes {
        jolt_shapes.push(shape.create_physics()?);
    }

    let mut buf: Vec<SubShapeSettings> = Vec::with_capacity(8);
    for compound_shape in &asset.compound_shapes {
        for sub_shape in &compound_shape.sub_shapes {
            let jolt_shape = jolt_shapes
                .get(sub_shape.shape_index as usize)
                .ok_or_else(|| xerrf!(BadAsset; "file={}, shape_index={}", path, sub_shape.shape_index))?;
            buf.push(SubShapeSettings::new(
                jolt_shape.clone(),
                sub_shape.position,
                sub_shape.rotation,
            ));
        }
        if !buf.is_empty() {
            let settings = StaticCompoundShapeSettings::new(&buf);
            let jolt_shape = jolt::create_static_compound_shape(&settings).map_err(xfrom!())?;
            jolt_shapes.push(jolt_shape.into());
            buf.clear();
        }
    }

    let mut bodies = Vec::with_capacity(asset.bodies.len());
    for asset_box in &asset.bodies {
        let jolt_shape = jolt_shapes
            .get(asset_box.shape_index as usize)
            .ok_or_else(|| xerrf!(BadAsset; "file={}, shape_index={}", path, asset_box.shape_index))?;
        bodies.push(LoadedZoneBody {
            shape: jolt_shape.clone(),
            position: asset_box.position,
            rotation: asset_box.rotation,
        });
    }

    Ok(LoadedZonePhysics { bodies })
}

pub fn from_archived_asset(path: &str, asset: &ArchivedAssetZonePhysics) -> XResult<LoadedZonePhysics> {
    let mut jolt_shapes = Vec::with_capacity(asset.shapes.len() + asset.compound_shapes.len());
    for shape in asset.shapes.iter() {
        let shape = rkyv::deserialize::<AssetShape, rkyv::rancor::Error>(shape).map_err(|_| xerr!(Rkyv))?;
        jolt_shapes.push(shape.create_physics()?);
    }

    let mut buf: Vec<SubShapeSettings> = Vec::with_capacity(8);
    for compound_shape in asset.compound_shapes.iter() {
        for sub_shape in compound_shape.sub_shapes.iter() {
            let shape_index = sub_shape.shape_index.to_native();
            let jolt_shape = jolt_shapes
                .get(shape_index as usize)
                .ok_or_else(|| xerrf!(BadAsset; "file={}, shape_index={}", path, shape_index))?;
            buf.push(SubShapeSettings::new(
                jolt_shape.clone(),
                sub_shape.position,
                sub_shape.rotation,
            ));
        }
        if !buf.is_empty() {
            let settings = StaticCompoundShapeSettings::new(&buf);
            let jolt_shape = jolt::create_static_compound_shape(&settings).map_err(xfrom!())?;
            jolt_shapes.push(jolt_shape.into());
            buf.clear();
        }
    }

    let mut bodies = Vec::with_capacity(asset.bodies.len());
    for asset_box in asset.bodies.iter() {
        let shape_index = asset_box.shape_index.to_native();
        let jolt_shape = jolt_shapes
            .get(shape_index as usize)
            .ok_or_else(|| xerrf!(BadAsset; "file={}, shape_index={}", path, shape_index))?;
        bodies.push(LoadedZoneBody {
            shape: jolt_shape.clone(),
            position: asset_box.position,
            rotation: asset_box.rotation,
        });
    }

    Ok(LoadedZonePhysics { bodies })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_ASSET_PATH;
    use crate::utils::sb;

    #[test]
    fn test_load_zone_physics() {
        let mut loader = AssetLoader::new(TEST_ASSET_PATH).unwrap();
        let zone_phy = loader.load_zone_physics(sb!("Zones/TestZone.*")).unwrap();
        assert!(zone_phy.bodies.len() > 0);
    }
}
