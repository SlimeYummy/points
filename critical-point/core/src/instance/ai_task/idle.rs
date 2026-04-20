use crate::instance::ai_task::base::{InstAiTaskAny, InstAiTaskBase};
use crate::template::{At, TmplAiTaskIdle};
use crate::utils::{AiTaskType, TimeRange, TmplID, extend};

#[repr(C)]
#[derive(Debug)]
pub struct InstAiTaskIdle {
    pub _base: InstAiTaskBase,
    pub max_repeat: u32,
    pub action_idle: TmplID,
    pub duration: TimeRange,
}

extend!(InstAiTaskIdle, InstAiTaskBase);

unsafe impl InstAiTaskAny for InstAiTaskIdle {
    #[inline]
    fn typ(&self) -> AiTaskType {
        AiTaskType::Idle
    }

    #[inline]
    fn actions(&self, actions: &mut Vec<TmplID>) {
        self.actions().for_each(|anime| actions.push(anime));
    }
}

impl InstAiTaskIdle {
    pub(crate) fn new(tmpl: At<TmplAiTaskIdle>) -> InstAiTaskIdle {
        InstAiTaskIdle {
            _base: InstAiTaskBase { tmpl_id: tmpl.id },
            max_repeat: tmpl.max_repeat.to_native(),
            action_idle: tmpl.action_idle,
            duration: tmpl.duration,
        }
    }

    #[inline]
    pub fn actions(&self) -> impl Iterator<Item = TmplID> {
        std::iter::once(self.action_idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_new_inst_ai_task_idle() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let tmpl = db.find_as::<TmplAiTaskIdle>(id!("AiTask.NpcInstance.Idle^1")).unwrap();
        let inst = InstAiTaskIdle::new(tmpl);

        assert_eq!(inst.tmpl_id, id!("AiTask.NpcInstance.Idle^1"));
        assert_eq!(inst.max_repeat, 1);
        assert_eq!(inst.action_idle, id!("Action.NpcInstance.Idle^1A"));
        assert_eq!(inst.duration.min(), 3.0);
        assert_eq!(inst.duration.max(), 5.0);
    }
}
