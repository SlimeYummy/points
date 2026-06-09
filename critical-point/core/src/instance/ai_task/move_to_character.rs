use crate::instance::ai_task::base::{InstAiTaskAny, InstAiTaskBase};
use crate::template::{At, TmplAiTaskMoveToCharacter};
use crate::utils::{AiTaskType, F32Range, TmplID, extend};

#[repr(C)]
#[derive(Debug)]
pub struct InstAiTaskMoveToCharacter {
    pub _base: InstAiTaskBase,
    pub character_npc: TmplID,
    pub enter_level: u16,
    pub keep_level: u16,
    pub move_action: TmplID,
    pub turn_action: TmplID,
    pub expected_distance: F32Range,
    pub expected_toward: f32,
}

extend!(InstAiTaskMoveToCharacter, InstAiTaskBase);

unsafe impl InstAiTaskAny for InstAiTaskMoveToCharacter {
    #[inline]
    fn typ(&self) -> AiTaskType {
        AiTaskType::MoveToCharacter
    }

    #[inline]
    fn actions(&self, actions: &mut Vec<TmplID>) {
        self.actions().for_each(|id| actions.push(id));
    }
}

impl InstAiTaskMoveToCharacter {
    pub(crate) fn new(tmpl: At<TmplAiTaskMoveToCharacter>) -> InstAiTaskMoveToCharacter {
        InstAiTaskMoveToCharacter {
            _base: InstAiTaskBase { tmpl_id: tmpl.id },
            character_npc: tmpl.character_npc,
            enter_level: tmpl.enter_level.to_native(),
            keep_level: tmpl.keep_level.to_native(),
            move_action: tmpl.move_action,
            turn_action: tmpl.turn_action,
            expected_distance: tmpl.expected_distance,
            expected_toward: tmpl.expected_toward.to_native(),
        }
    }

    #[inline]
    fn actions(&self) -> impl Iterator<Item = TmplID> + '_ {
        std::iter::from_coroutine(
            #[coroutine]
            || {
                if self.move_action.is_valid() {
                    yield self.move_action;
                }
                if self.turn_action.is_valid() {
                    yield self.turn_action;
                }
            },
        )
    }

    pub fn is_in_range(&self, distance: f32) -> bool {
        self.expected_distance.contains(distance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::{LEVEL_MOVE, id};

    #[test]
    fn test_new() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let tmpl = db
            .find_as::<TmplAiTaskMoveToCharacter>(id!("AiTask.InstanceNpc.MoveTo^1"))
            .unwrap();
        let inst = InstAiTaskMoveToCharacter::new(tmpl);

        assert_eq!(inst.tmpl_id, id!("AiTask.InstanceNpc.MoveTo^1"));
        assert_eq!(inst.character_npc, id!("CharacterNpc.InstanceNpc^1"));
        assert_eq!(inst.enter_level, LEVEL_MOVE);
        assert_eq!(inst.keep_level, LEVEL_MOVE);
        assert_eq!(inst.expected_distance, F32Range::new(5.0, 8.0));
        assert_eq!(inst.move_action, id!("Action.InstanceNpc.Walk^1A"));
        assert_eq!(inst.turn_action, id!("Action.InstanceNpc.Walk^1A"));
    }
}
