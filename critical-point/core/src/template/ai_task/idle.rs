use crate::template::base::impl_tmpl;
use crate::utils::TmplID;

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiTaskIdle {
    pub id: TmplID,
    pub character_npc: TmplID,
    pub enter_level: u16,
    pub keep_level: u16,
    pub action_idle: TmplID,
}

impl_tmpl!(TmplAiTaskIdle, AiTaskIdle, "AiTaskIdle");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::{LEVEL_IDLE, id};

    #[test]
    fn test_load_ai_task_idle() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let task = db.find_as::<TmplAiTaskIdle>(id!("AiTask.Enemy.Idle")).unwrap();
        assert_eq!(task.id, id!("AiTask.Enemy.Idle"));
        assert_eq!(task.character_npc, id!("CharacterNpc.Enemy"));
        assert_eq!(task.enter_level, LEVEL_IDLE + 1);
        assert_eq!(task.keep_level, LEVEL_IDLE + 1);
        assert_eq!(task.action_idle, id!("Action.Enemy.Idle"));
    }
}
