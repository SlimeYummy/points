use glam::Vec3A;

use crate::template::base::impl_tmpl;
use crate::utils::{AiIntention, TmplID};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiTaskPatrol {
    pub id: TmplID,
    pub character_npc: TmplID,
    pub intention: AiIntention,
    pub next_intention: AiIntention,
    pub action_idle: TmplID,
    pub action_move: TmplID,
    pub route: Vec<TmplAiTaskPatrolStep>,
    pub target_exit: bool,
}

impl_tmpl!(TmplAiTaskPatrol, AiTaskPatrol, "AiTaskPatrol");

#[derive(Debug, Clone, Copy, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub enum TmplAiTaskPatrolStep {
    Move(Vec3A),
    Idle(f32),
}

impl TmplAiTaskPatrolStep {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplAiTaskPatrolStep) -> Self {
        match archived {
            ArchivedTmplAiTaskPatrolStep::Move(pos) => TmplAiTaskPatrolStep::Move(*pos),
            ArchivedTmplAiTaskPatrolStep::Idle(duration) => TmplAiTaskPatrolStep::Idle(duration.to_native()),
        }
    }
}

const _: () = {
    use serde::de::{Deserialize, Deserializer, Error, SeqAccess, Visitor};
    use serde::ser::{Serialize, SerializeTuple, Serializer};
    use std::fmt;

    impl Serialize for TmplAiTaskPatrolStep {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self {
                TmplAiTaskPatrolStep::Move(pos) => {
                    let mut tuple = serializer.serialize_tuple(2)?;
                    tuple.serialize_element("Move")?;
                    tuple.serialize_element(&[pos.x, pos.y, pos.z])?;
                    tuple.end()
                }
                TmplAiTaskPatrolStep::Idle(duration) => {
                    let mut tuple = serializer.serialize_tuple(2)?;
                    tuple.serialize_element("Idle")?;
                    tuple.serialize_element(duration)?;
                    tuple.end()
                }
            }
        }
    }

    impl<'de> Deserialize<'de> for TmplAiTaskPatrolStep {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct StepVisitor;

            impl<'de> Visitor<'de> for StepVisitor {
                type Value = TmplAiTaskPatrolStep;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a tuple [\"Move\", [x, y, z]] or [\"Idle\", duration]")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let tag: &str = seq.next_element()?.ok_or_else(|| Error::invalid_length(0, &self))?;

                    match tag {
                        "Move" => {
                            let pos: Vec3A = seq.next_element()?.ok_or_else(|| Error::invalid_length(1, &self))?;
                            Ok(TmplAiTaskPatrolStep::Move(pos))
                        }
                        "Idle" => {
                            let duration: f32 = seq.next_element()?.ok_or_else(|| Error::invalid_length(1, &self))?;
                            Ok(TmplAiTaskPatrolStep::Idle(duration))
                        }
                        _ => Err(Error::unknown_variant(tag, &["Move", "Idle"])),
                    }
                }
            }

            deserializer.deserialize_tuple(2, StepVisitor)
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::id;
    use glam::vec3a;

    #[test]
    fn test_load_ai_task_patrol() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let task = db.find_as::<TmplAiTaskPatrol>(id!("AiTask.Enemy.Patrol")).unwrap();
        assert_eq!(task.id, id!("AiTask.Enemy.Patrol"));
        assert_eq!(task.character_npc, id!("CharacterNpc.Enemy"));
        assert_eq!(task.intention, AiIntention::Move);
        assert_eq!(task.next_intention, AiIntention::Idle);
        assert_eq!(task.action_idle, id!("Action.Enemy.Idle"));
        assert_eq!(task.action_move, id!("Action.Enemy.Walk"));

        assert_eq!(task.route.len(), 3);
        assert_eq!(
            TmplAiTaskPatrolStep::from_rkyv(&task.route[0]),
            TmplAiTaskPatrolStep::Move(vec3a(10.0, 0.0, 0.0))
        );
        assert_eq!(
            TmplAiTaskPatrolStep::from_rkyv(&task.route[1]),
            TmplAiTaskPatrolStep::Idle(1.0)
        );
        assert_eq!(
            TmplAiTaskPatrolStep::from_rkyv(&task.route[2]),
            TmplAiTaskPatrolStep::Move(vec3a(3.0, -4.0, -5.0))
        );
        assert_eq!(task.target_exit, true);
    }
}
