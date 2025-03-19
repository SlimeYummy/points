use glam::{Quat, Vec3A};
use jolt_physics_rs::{BodyCreationSettings, BodyID, BodyInterface};

use crate::asset::loader::AssetLoader;
use crate::asset::shape::{default_position, default_rotation, AssetShape};
use crate::logic::PHY_LAYER_STATIC;
use crate::utils::{xerrf, xfrom, XResult};

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
pub struct AssetStage {
    shapes: Vec<AssetShape>,
    bodies: Vec<AssetStageBody>,
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize)]
pub struct AssetStageBody {
    shape_index: u32,
    #[serde(default = "default_position")]
    position: Vec3A,
    #[serde(default = "default_rotation")]
    rotation: Quat,
}

#[derive(Debug, Default)]
pub struct LoadedStage {
    pub bodies: Vec<BodyID>,
}

impl AssetLoader {
    pub fn load_stage(&mut self, file: &str, body_itf: &mut BodyInterface) -> XResult<LoadedStage> {
        let asset_stage = self.load_json::<AssetStage, _>(file)?;

        let mut jolt_shapes = Vec::with_capacity(asset_stage.shapes.len());
        for shape in &asset_stage.shapes {
            jolt_shapes.push(self.load_shape(shape)?);
        }

        let mut bodies = Vec::with_capacity(asset_stage.bodies.len());
        for asset_body in &asset_stage.bodies {
            let jolt_shape = jolt_shapes
                .get(asset_body.shape_index as usize)
                .ok_or_else(|| xerrf!(BadAsset; "file={} shape_index={}", file, asset_body.shape_index))?;
            let settings = BodyCreationSettings::new_static(
                jolt_shape.clone(),
                PHY_LAYER_STATIC,
                asset_body.position,
                asset_body.rotation,
            );
            let body = body_itf.create_body(&settings).map_err(xfrom!())?;
            bodies.push(body);
        }

        Ok(LoadedStage { bodies })
    }
}
