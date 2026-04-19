use crate::template::base::impl_tmpl;
use crate::utils::{TimeRange, TmplID};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiTaskIdle {
    pub id: TmplID,
    pub character: TmplID,
    pub max_repeat: u32,
    pub action_idle: TmplID,
    pub duration: TimeRange,
}

impl_tmpl!(TmplAiTaskIdle, AiTaskIdle, "AiTaskIdle");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_load_ai_task_idle() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let task = db.find_as::<TmplAiTaskIdle>(id!("AiTask.Enemy.Idle")).unwrap();
        assert_eq!(task.id, id!("AiTask.Enemy.Idle"));
        assert_eq!(task.character, id!("NpcCharacter.Enemy"));
        assert_eq!(task.max_repeat, 1);
        assert_eq!(task.action_idle, id!("Action.Enemy.Idle"));
        assert_eq!(task.duration.min(), 4.0);
        assert_eq!(task.duration.max(), 6.0);
    }
}
