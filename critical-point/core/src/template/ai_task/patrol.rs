use glam::Vec3A;

use crate::template::base::impl_tmpl;
use crate::utils::TmplID;

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiTaskPatrol {
    pub id: TmplID,
    pub character: TmplID,
    pub action_idle: TmplID,
    pub action_move: TmplID,
    pub route: Vec<TmplAiTaskPatrolStep>,
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
