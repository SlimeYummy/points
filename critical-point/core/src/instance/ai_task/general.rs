use glam_ext::Vec2xz;

use crate::instance::ai_task::base::{InstAiTaskAny, InstAiTaskBase};
use crate::template::{At, TmplAiTaskGeneral, TmplAiTaskGeneralMove};
use crate::utils::{AiTaskType, F32Range, SmallVec, TmplID, extend};

pub type InstAiTaskGeneralMove = TmplAiTaskGeneralMove;

#[repr(C)]
#[derive(Debug)]
pub struct InstAiTaskGeneral {
    pub _base: InstAiTaskBase,
    pub character_npc: TmplID,
    pub enter_level: u16,
    pub leave_level_moving: u16,
    pub keep_level_acting: u16,
    pub expected_distance: F32Range,
    pub moves: SmallVec<[InstAiTaskGeneralMove; 3]>,
    pub actions: SmallVec<[TmplID; 4]>,
}

extend!(InstAiTaskGeneral, InstAiTaskBase);

unsafe impl InstAiTaskAny for InstAiTaskGeneral {
    #[inline]
    fn typ(&self) -> AiTaskType {
        AiTaskType::General
    }

    #[inline]
    fn actions(&self, actions: &mut Vec<TmplID>) {
        self.actions().for_each(|id| actions.push(id));
    }
}

impl InstAiTaskGeneral {
    pub(crate) fn new(tmpl: At<TmplAiTaskGeneral>) -> InstAiTaskGeneral {
        InstAiTaskGeneral {
            _base: InstAiTaskBase { tmpl_id: tmpl.id },
            character_npc: tmpl.character_npc,
            enter_level: tmpl.enter_level.to_native(),
            leave_level_moving: tmpl.leave_level_moving.to_native(),
            keep_level_acting: tmpl.keep_level_acting.to_native(),
            expected_distance: tmpl.expected_distance,
            moves: tmpl.moves.iter().map(InstAiTaskGeneralMove::from_rkyv).collect(),
            actions: tmpl.actions.iter().map(|id| (*id).into()).collect(),
        }
    }

    #[inline]
    fn actions(&self) -> impl Iterator<Item = TmplID> + '_ {
        std::iter::from_coroutine(
            #[coroutine]
            || {
                for mv in &self.moves {
                    if mv.action.is_valid() {
                        yield mv.action;
                    }
                }
                for action in &self.actions {
                    yield *action;
                }
            },
        )
    }

    // in world space
    pub fn find_move_by_target(&self, distance: f32, _direction: Vec2xz) -> Option<(usize, &InstAiTaskGeneralMove)> {
        for (idx, mov) in self.moves.iter().enumerate() {
            if mov.distance.contains(distance) {
                return Some((idx, mov));
            }
        }
        None
    }
}
