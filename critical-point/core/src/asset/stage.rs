use glam::{Quat, Vec3A};
use jolt_physics_rs::{self as jolt, BodyID, PHY_LAYER_STATIC};
use serde::Deserialize;

use crate::asset::loader::AssetLoader;
use crate::asset::shape::AssetShapeEx;
use crate::utils::{NumID, XResult};

#[derive(Debug, Default, Deserialize)]
pub struct AssetStage {
    shapes: Vec<AssetShapeEx>,
    bodies: Vec<AssetStageBody>,
}

#[derive(Debug, Default, Deserialize)]
pub struct AssetStageBody {
    shape_id: NumID,
    #[serde(default = "crate::utils::default_position")]
    position: Vec3A,
    #[serde(default = "crate::utils::default_rotation")]
    rotation: Quat,
}

impl AssetLoader {
    pub fn load_stage(&mut self, asset_id: &str) -> XResult<Vec<BodyID>> {
        let stage = self.load_json::<AssetStage>(&format!("{}.json", asset_id))?;
        for shape in &stage.shapes {
            self.load_shape_ex(shape)?;
        }

        let mut bodies = Vec::with_capacity(stage.bodies.len());
        for object in &stage.bodies {
            let ref_shape = self.get_shape_ex(object.shape_id)?;
            let settings =
                jolt::BodySettings::new_static(ref_shape, PHY_LAYER_STATIC, object.position, object.rotation);
            let body = self.create_body(&settings)?;
            bodies.push(body);
        }

        Ok(bodies)
    }
}
