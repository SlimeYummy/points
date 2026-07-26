use rkyv::option::ArchivedOption;

use crate::instance::ai_task::base::{InstAiTaskAny, InstAiTaskBase};
use crate::template::{At, TmplAiTaskIdle};
use crate::utils::{AiIntention, AiTaskType, F32Range, TmplID, extend};

#[repr(C)]
#[derive(Debug)]
pub struct InstAiTaskIdle {
    pub _base: InstAiTaskBase,
    pub intention: AiIntention,
    pub next_intention: AiIntention,
    pub action_idle: TmplID,
    pub duration: Option<F32Range>,
    pub target_exit: bool,
}

extend!(InstAiTaskIdle, InstAiTaskBase);

unsafe impl InstAiTaskAny for InstAiTaskIdle {
    #[inline]
    fn typ(&self) -> AiTaskType {
        AiTaskType::Idle
    }

    #[inline]
    fn actions(&self, actions: &mut Vec<TmplID>) {
        actions.push(self.action_idle);
    }
}

impl InstAiTaskIdle {
    pub(crate) fn new(tmpl: At<TmplAiTaskIdle>) -> InstAiTaskIdle {
        InstAiTaskIdle {
            _base: InstAiTaskBase { tmpl_id: tmpl.id },
            intention: tmpl.intention,
            next_intention: tmpl.next_intention,
            action_idle: tmpl.action_idle,
            duration: match tmpl.duration {
                ArchivedOption::Some(dura) => Some(dura),
                ArchivedOption::None => None,
            },
            target_exit: tmpl.target_exit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_new() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let tmpl = db.find_as::<TmplAiTaskIdle>(id!("AiTask.InstanceNpc.Idle^1")).unwrap();
        let inst = InstAiTaskIdle::new(tmpl);

        assert_eq!(inst.tmpl_id, id!("AiTask.InstanceNpc.Idle^1"));
        assert_eq!(inst.intention, AiIntention::Idle);
        assert_eq!(inst.next_intention, AiIntention::Idle);
        assert_eq!(inst.action_idle, id!("Action.InstanceNpc.Idle^1A"));
        assert_eq!(inst.duration, None);
        assert_eq!(inst.target_exit, false);
    }
}
