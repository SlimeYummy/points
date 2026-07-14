use crate::template::base::impl_tmpl;
use crate::utils::{AiIntention, F32Range, TmplID};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiTaskGeneral {
    pub id: TmplID,
    pub character_npc: TmplID,
    pub intention: AiIntention,
    pub next_intention: AiIntention,
    pub actions: Vec<TmplID>,
}

impl_tmpl!(TmplAiTaskGeneral, AiTaskGeneral, "AiTaskGeneral");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_load_ai_task_general() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let task = db.find_as::<TmplAiTaskGeneral>(id!("AiTask.Enemy.Attack")).unwrap();
        assert_eq!(task.id, id!("AiTask.Enemy.Attack"));
        assert_eq!(task.character_npc, id!("CharacterNpc.Enemy"));
        assert_eq!(task.intention, AiIntention::Attack);
        assert_eq!(task.next_intention, AiIntention::SquareOff);
        assert_eq!(task.actions, vec![id!("Action.Enemy.Idle"), id!("Action.Enemy.Walk")]);
    }
}
