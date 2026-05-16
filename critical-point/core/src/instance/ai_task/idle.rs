use crate::instance::ai_task::base::{InstAiTaskAny, InstAiTaskBase};
use crate::template::{At, TmplAiTaskIdle, TmplAiTaskIdleStep};
use crate::utils::{AiTaskType, TmplID, extend};

pub type InstAiTaskIdleStep = TmplAiTaskIdleStep;

#[repr(C)]
#[derive(Debug)]
pub struct InstAiTaskIdle {
    pub _base: InstAiTaskBase,
    pub action_idle: TmplID,
    pub action_move: TmplID,
    pub route: Vec<InstAiTaskIdleStep>,
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
            action_idle: tmpl.action_idle,
            action_move: tmpl.action_move,
            route: tmpl.route.iter().map(InstAiTaskIdleStep::from_rkyv).collect(),
        }
    }

    #[inline]
    fn actions(&self) -> impl Iterator<Item = TmplID> + '_ {
        std::iter::from_coroutine(
            #[coroutine]
            || {
                yield self.action_idle;
                yield self.action_move;
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::id;
    use glam::Vec3A;

    #[test]
    fn test_new_inst_ai_task_idle() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let tmpl = db.find_as::<TmplAiTaskIdle>(id!("AiTask.InstanceNpc.Idle^1")).unwrap();
        let inst = InstAiTaskIdle::new(tmpl);

        assert_eq!(inst.tmpl_id, id!("AiTask.InstanceNpc.Idle^1"));
        assert_eq!(inst.action_idle, id!("Action.InstanceNpc.Idle^1A"));
        assert_eq!(inst.action_move, id!("Action.InstanceNpc.Walk^1A"));

        assert_eq!(inst.route.len(), 3);
        assert_eq!(inst.route[0], InstAiTaskIdleStep::Move(Vec3A::new(-3.0, 0.0, 0.0)));
        assert_eq!(inst.route[1], InstAiTaskIdleStep::Idle(2.5));
        assert_eq!(inst.route[2], InstAiTaskIdleStep::Move(Vec3A::new(0.0, 0.0, 3.0)));
    }
}
