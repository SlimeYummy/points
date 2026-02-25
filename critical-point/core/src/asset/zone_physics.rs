use glam::{Quat, Vec3A};
use jolt_physics_rs::{self as jolt, JRef, Shape, StaticCompoundShapeSettings, SubShapeSettings};

use crate::asset::AssetIndxedCompoundShape;
use crate::asset::loader::AssetLoader;
use crate::asset::shape::AssetShape;
use crate::utils::{default_position, default_rotation, xerrf, XResult, xfrom};

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
pub struct AssetZonePhysics {
    shapes: Vec<AssetShape>,
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
    pub fn load_zone_physics(&mut self, file: &str) -> XResult<LoadedZonePhysics> {
        let asset_zone = self.load_json::<AssetZonePhysics, _>(file)?;

        let mut jolt_shapes = Vec::with_capacity(asset_zone.shapes.len() + asset_zone.compound_shapes.len());
        for shape in &asset_zone.shapes {
            jolt_shapes.push(shape.create_physics()?);
        }

        let mut buf: Vec<SubShapeSettings> = Vec::with_capacity(8);
        for compound_shape in &asset_zone.compound_shapes {
            for sub_shape in &compound_shape.sub_shapes {
                let jolt_shape = jolt_shapes
                    .get(sub_shape.shape_index as usize)
                    .ok_or_else(|| xerrf!(BadAsset; "file={}, shape_index={}", file, sub_shape.shape_index))?;
                buf.push(SubShapeSettings::new(jolt_shape.clone(), sub_shape.position, sub_shape.rotation));
            }
            if !buf.is_empty() {
                let settings = StaticCompoundShapeSettings::new(&buf);
                let jolt_shape = jolt::create_static_compound_shape(&settings).map_err(xfrom!())?;
                jolt_shapes.push(jolt_shape.into());
                buf.clear();
            }
        }

        let mut bodies = Vec::with_capacity(asset_zone.bodies.len());
        for asset_box in &asset_zone.bodies {
            let jolt_shape = jolt_shapes
                .get(asset_box.shape_index as usize)
                .ok_or_else(|| xerrf!(BadAsset; "file={}, shape_index={}", file, asset_box.shape_index))?;
            bodies.push(LoadedZoneBody {
                shape: jolt_shape.clone(),
                position: asset_box.position,
                rotation: asset_box.rotation,
            });
        }

        Ok(LoadedZonePhysics { bodies })
    }
}

// pub fn from_asset(body_itf: &mut BodyInterface, path: &str, asset: AssetZonePhysics) -> XResult<LoadedZonePhysics> {
//     let mut jolt_shapes = Vec::with_capacity(asset.shapes.len());
//     for shape in &asset.shapes {
//         jolt_shapes.push(shape.create_physics()?);
//     }

//     let mut bodies = Vec::with_capacity(asset.bodies.len());
//     for asset_box in &asset.bodies {
//         let jolt_shape: &jolt_physics_rs::JRef<jolt_physics_rs::Shape> = jolt_shapes
//             .get(asset_box.shape_index as usize)
//             .ok_or_else(|| xerrf!(BadAsset; "file={}, shape_index={}", path, asset_box.shape_index))?;
//         let settings = BodyCreationSettings::new_static(
//             jolt_shape.clone(),
//             phy_layer!(StaticScenery, All),
//             asset_box.position,
//             asset_box.rotation,
//         );
//         let body = body_itf.create_body(&settings).map_err(xfrom!())?;
//         bodies.push(body);
//     }

//     Ok(LoadedZonePhysics { bodies })
// }

// pub fn from_archived_asset(body_itf: &mut BodyInterface, path: &str, asset: &ArchivedAssetZonePhysics) -> XResult<LoadedZonePhysics> {
//     let mut jolt_shapes = Vec::with_capacity(asset.shapes.len());
//     for shape in asset.shapes.iter() {
//         let shape = rkyv::deserialize::<AssetShape, rkyv::rancor::Error>(shape).map_err(|_| xerr!(Rkyv))?;
//         jolt_shapes.push(shape.create_physics()?);
//     }

//     let mut bodies = Vec::with_capacity(asset.bodies.len());
//     for asset_box in asset.bodies.iter() {
//         let shape_index: u32 = asset_box.shape_index.into();
//         let jolt_shape: &jolt_physics_rs::JRef<jolt_physics_rs::Shape> = jolt_shapes
//             .get(shape_index as usize)
//             .ok_or_else(|| xerrf!(BadAsset; "file={}, shape_index={}", path, asset_box.shape_index))?;
//         let settings = BodyCreationSettings::new_static(
//             jolt_shape.clone(),
//             phy_layer!(StaticScenery, All),
//             asset_box.position,
//             asset_box.rotation,
//         );
//         let body = body_itf.create_body(&settings).map_err(xfrom!())?;
//         bodies.push(body);
//     }

//     Ok(LoadedZonePhysics { bodies })
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_ASSET_PATH;

    #[test]
    fn test_load_zone_physics() {
        let mut loader = AssetLoader::new(TEST_ASSET_PATH).unwrap();
        let zone_phy = loader.load_zone_physics("TestZone.json").unwrap();
        assert!(zone_phy.bodies.len() > 0);
    }
}
