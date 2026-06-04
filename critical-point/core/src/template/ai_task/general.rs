use crate::template::base::impl_tmpl;
use crate::utils::{F32Range, TmplID};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiTaskGeneral {
    pub id: TmplID,
    pub character_npc: TmplID,
    pub enter_level: u16,
    pub keep_level: u16,
    pub actions: Vec<TmplID>,
}

impl_tmpl!(TmplAiTaskGeneral, AiTaskGeneral, "AiTaskGeneral");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::{LEVEL_ATTACK, id};

    #[test]
    fn test_load_ai_task_general() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let task = db.find_as::<TmplAiTaskGeneral>(id!("AiTask.Enemy.Attack")).unwrap();
        assert_eq!(task.id, id!("AiTask.Enemy.Attack"));
        assert_eq!(task.character_npc, id!("CharacterNpc.Enemy"));
        assert_eq!(task.enter_level, LEVEL_ATTACK + 1);
        assert_eq!(task.keep_level, LEVEL_ATTACK + 1);
        assert_eq!(task.actions, vec![id!("Action.Enemy.Idle"), id!("Action.Enemy.Walk")]);
    }
}
