use crate::template::base::impl_tmpl;
use crate::utils::{F32Range, TmplID};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplAiTaskMoveToCharacter {
    pub id: TmplID,
    pub character_npc: TmplID,
    pub enter_level: u16,
    pub keep_level: u16,
    pub move_action: TmplID,
    pub turn_action: TmplID,
    pub expected_distance: F32Range,
    /// Half angle (in XZ plane) between character's forward and vector from character to target.
    pub expected_toward: f32,
}

impl_tmpl!(
    TmplAiTaskMoveToCharacter,
    AiTaskMoveToCharacter,
    "AiTaskMoveToCharacter"
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::{LEVEL_MOVE, id};

    #[test]
    fn test_load_ai_task_reposition() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let task = db
            .find_as::<TmplAiTaskMoveToCharacter>(id!("AiTask.Enemy.MoveTo"))
            .unwrap();
        assert_eq!(task.id, id!("AiTask.Enemy.MoveTo"));
        assert_eq!(task.character_npc, id!("CharacterNpc.Enemy"));
        assert_eq!(task.enter_level, LEVEL_MOVE + 1);
        assert_eq!(task.keep_level, LEVEL_MOVE + 1);
        assert_eq!(task.expected_distance, F32Range::new(4.0, 6.0));
        assert_eq!(task.expected_toward.to_native(), 180.0_f32.to_radians());
        assert_eq!(task.move_action, id!("Action.Enemy.Walk"));
        assert_eq!(task.turn_action, id!("Action.Enemy.Walk"));
    }
}
