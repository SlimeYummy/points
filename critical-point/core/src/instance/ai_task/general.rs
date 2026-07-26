use crate::instance::ai_task::base::{InstAiTaskAny, InstAiTaskBase};
use crate::template::{At, TmplAiTaskGeneral};
use crate::utils::{AiIntention, AiTaskType, SmallVec, TmplID, extend};

#[repr(C)]
#[derive(Debug)]
pub struct InstAiTaskGeneral {
    pub _base: InstAiTaskBase,
    pub intention: AiIntention,
    pub next_intention: AiIntention,
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
        for action in &self.actions {
            actions.push(*action);
        }
    }
}

impl InstAiTaskGeneral {
    pub(crate) fn new(tmpl: At<TmplAiTaskGeneral>) -> InstAiTaskGeneral {
        InstAiTaskGeneral {
            _base: InstAiTaskBase { tmpl_id: tmpl.id },
            intention: tmpl.intention,
            next_intention: tmpl.next_intention,
            actions: tmpl.actions.iter().map(|id| (*id).into()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_new_ai_task_general() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let tmpl = db
            .find_as::<TmplAiTaskGeneral>(id!("AiTask.InstanceNpc.General^1"))
            .unwrap();
        let inst = InstAiTaskGeneral::new(tmpl);

        assert_eq!(inst.tmpl_id, id!("AiTask.InstanceNpc.General^1"));
        assert_eq!(inst.intention, AiIntention::Attack);
        assert_eq!(inst.next_intention, AiIntention::SquareOff);
        assert_eq!(inst.actions.len(), 2);
        assert_eq!(inst.actions[0], id!("Action.InstanceNpc.Idle^1A"));
        assert_eq!(inst.actions[1], id!("Action.InstanceNpc.Walk^1A"));
    }
}
