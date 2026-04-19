use crate::template::base::impl_tmpl;
use crate::utils::{ShapeSphere, ShapeSphericalCone, TmplID};

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct AiIf {}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
#[serde(tag = "T")]
pub enum AiNode {
    Task(AiNodeTask),
    Branch(AiNodeBranch),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct AiNodeTask {
    pub task: TmplID,
    pub weight: f32,
    pub priority: i32,
    pub conditions: Vec<AiIf>,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
#[rkyv(serialize_bounds(
    __S:rkyv::ser::Writer + rkyv::ser::Allocator,
    __S::Error: rkyv::rancor::Source
))]
#[rkyv(deserialize_bounds(__D::Error: rkyv::rancor::Source))]
#[rkyv(bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)))]
pub struct AiNodeBranch {
    pub conditions: Vec<AiIf>,
    #[rkyv(omit_bounds)]
    pub nodes: Vec<AiNode>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiBrain {
    pub id: TmplID,
    pub character: TmplID,
    pub alert_sphere: ShapeSphere,
    pub alert_cone: ShapeSphericalCone,
    pub attack_exit_delay: f32,
    pub idle: AiNodeBranch,
    // pub attack_plan: TmplID,
}

impl_tmpl!(TmplAiBrain, AiBrain, "AiBrain");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::id;
    use rkyv::rancor::Error;

    #[test]
    fn test_load_ai_executor() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let executor = db.find_as::<TmplAiBrain>(id!("AiBrain.Enemy")).unwrap();
        assert_eq!(executor.id, id!("AiBrain.Enemy"));
        assert_eq!(executor.character, id!("NpcCharacter.Enemy"));

        assert_eq!(executor.alert_sphere.radius, 5.0);
        assert_eq!(executor.alert_cone.radius, 10.0);
        assert_eq!(executor.alert_cone.half_angle, 45.0f32.to_radians());
        assert_eq!(executor.attack_exit_delay, 30.0);

        assert_eq!(executor.idle.nodes.len(), 1);
        assert_eq!(executor.idle.conditions.len(), 0);
        assert_eq!(
            rkyv::deserialize::<AiNode, Error>(&executor.idle.nodes[0]).unwrap(),
            AiNode::Task(AiNodeTask {
                task: id!("AiTask.Enemy.Idle"),
                weight: 1.0,
                priority: 0,
                conditions: vec![],
            })
        );
    }
}
