use crate::instance::ai_task::base::{InstAiTaskAny, InstAiTaskBase};
use crate::template::{At, TmplAiTaskIdle};
use crate::utils::{AiTaskType, F32Range, TmplID, extend};

#[repr(C)]
#[derive(Debug)]
pub struct InstAiTaskIdle {
    pub _base: InstAiTaskBase,
    pub enter_level: u16,
    pub keep_level: u16,
    pub action_idle: TmplID,
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
            enter_level: tmpl.enter_level.to_native(),
            keep_level: tmpl.keep_level.to_native(),
            action_idle: tmpl.action_idle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::{LEVEL_IDLE, id};

    #[test]
    fn test_new() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let tmpl = db.find_as::<TmplAiTaskIdle>(id!("AiTask.InstanceNpc.Idle^1")).unwrap();
        let inst = InstAiTaskIdle::new(tmpl);

        assert_eq!(inst.tmpl_id, id!("AiTask.InstanceNpc.Idle^1"));
        assert_eq!(inst.enter_level, LEVEL_IDLE);
        assert_eq!(inst.keep_level, LEVEL_IDLE);
        assert_eq!(inst.action_idle, id!("Action.InstanceNpc.Idle^1A"));
    }
}
