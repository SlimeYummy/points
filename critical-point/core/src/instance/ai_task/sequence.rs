use crate::instance::ai_task::base::{InstAiTaskAny, InstAiTaskBase};
use crate::template::{At, TmplAiTaskSequence};
use crate::utils::{AiTaskType, TmplID, extend};

#[repr(C)]
#[derive(Debug)]
pub struct InstAiTaskSequence {
    pub _base: InstAiTaskBase,
    pub character_npc: TmplID,
    pub enter_level: u16,
    pub tasks: Vec<TmplID>,
}

extend!(InstAiTaskSequence, InstAiTaskBase);

unsafe impl InstAiTaskAny for InstAiTaskSequence {
    #[inline]
    fn typ(&self) -> AiTaskType {
        AiTaskType::Sequence
    }

    #[inline]
    fn actions(&self, _actions: &mut Vec<TmplID>) {}
}

impl InstAiTaskSequence {
    pub(crate) fn new(tmpl: At<TmplAiTaskSequence>) -> InstAiTaskSequence {
        InstAiTaskSequence {
            _base: InstAiTaskBase { tmpl_id: tmpl.id },
            character_npc: tmpl.character_npc,
            enter_level: tmpl.enter_level.to_native(),
            tasks: tmpl.tasks.iter().map(|id| (*id).into()).collect(),
        }
    }
}
