use crate::template::base::impl_tmpl;
use crate::utils::{AiIntention, TmplID};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiTaskKeepDistance {
    pub id: TmplID,
    pub character_npc: TmplID,
    pub intention: AiIntention,
    pub next_intention: AiIntention,
    pub dodge_action: TmplID,
    pub expected_distance: f32,
}

impl_tmpl!(TmplAiTaskKeepDistance, AiTaskKeepDistance, "AiTaskKeepDistance");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_load_ai_task_keep_distance() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let task = db
            .find_as::<TmplAiTaskKeepDistance>(id!("AiTask.InstanceNpc.KeepDistance^1"))
            .unwrap();
        assert_eq!(task.id, id!("AiTask.InstanceNpc.KeepDistance^1"));
        assert_eq!(task.character_npc, id!("CharacterNpc.InstanceNpc^1"));
        assert_eq!(task.intention, AiIntention::Attack);
        assert_eq!(task.next_intention, AiIntention::SquareOff);
        assert_eq!(task.dodge_action, id!("Action.InstanceNpc.Dodge"));
        assert_eq!(task.expected_distance, 0.0);
    }
}
