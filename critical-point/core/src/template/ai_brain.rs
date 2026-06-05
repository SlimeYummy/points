use crate::template::base::impl_tmpl;
use crate::utils::{ShapeSphere, ShapeSphericalCone, TmplID};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiBrain {
    pub id: TmplID,
    pub character_npc: TmplID,
    pub alert_sphere: ShapeSphere,
    pub alert_cone: ShapeSphericalCone,
    pub aggro_sphere: ShapeSphere,
    pub aggro_lost_time: f32,
    pub tasks: Vec<TmplID>,
    pub execute: bool,
}

impl_tmpl!(TmplAiBrain, AiBrain, "AiBrain");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_load_ai_executor() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let executor = db.find_as::<TmplAiBrain>(id!("AiBrain.Enemy")).unwrap();
        assert_eq!(executor.id, id!("AiBrain.Enemy"));
        assert_eq!(executor.character_npc, id!("CharacterNpc.Enemy"));

        assert_eq!(executor.alert_sphere.radius, 5.0);
        assert_eq!(executor.alert_cone.radius, 10.0);
        assert_eq!(executor.alert_cone.half_angle, 45.0f32.to_radians());
        assert_eq!(executor.aggro_sphere.radius, 10.0);
        assert_eq!(executor.aggro_lost_time, 15.0);
        assert_eq!(executor.execute, true);
        assert_eq!(executor.tasks.len(), 1);
        assert_eq!(executor.tasks[0], id!("AiTask.Enemy.Idle"));
    }
}
