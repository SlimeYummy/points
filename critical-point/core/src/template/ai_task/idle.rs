use crate::template::base::impl_tmpl;
use crate::utils::{AiIntention, F32Range, TmplID};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiTaskIdle {
    pub id: TmplID,
    pub character_npc: TmplID,
    pub intention: AiIntention,
    pub next_intention: AiIntention,
    pub action_idle: TmplID,
    pub duration: Option<F32Range>,
    pub target_exit: bool,
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
        assert_eq!(task.character_npc, id!("CharacterNpc.Enemy"));
        assert_eq!(task.intention, AiIntention::Idle);
        assert_eq!(task.next_intention, AiIntention::Move);
        assert_eq!(task.action_idle, id!("Action.Enemy.Idle"));
        assert_eq!(task.duration, Some(F32Range::new(5.0, 10.0)));
        assert_eq!(task.target_exit, true);
    }
}
