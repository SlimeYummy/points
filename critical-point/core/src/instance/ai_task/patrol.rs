use crate::instance::ai_task::base::{InstAiTaskAny, InstAiTaskBase};
use crate::template::{At, TmplAiTaskPatrol, TmplAiTaskPatrolStep};
use crate::utils::{AiTaskType, TmplID, extend};

pub type InstAiTaskPatrolStep = TmplAiTaskPatrolStep;

#[repr(C)]
#[derive(Debug)]
pub struct InstAiTaskPatrol {
    pub _base: InstAiTaskBase,
    pub action_idle: TmplID,
    pub action_move: TmplID,
    pub route: Vec<InstAiTaskPatrolStep>,
}

extend!(InstAiTaskPatrol, InstAiTaskBase);

unsafe impl InstAiTaskAny for InstAiTaskPatrol {
    #[inline]
    fn typ(&self) -> AiTaskType {
        AiTaskType::Patrol
    }

    #[inline]
    fn actions(&self, actions: &mut Vec<TmplID>) {
        self.actions().for_each(|anime| actions.push(anime));
    }
}

impl InstAiTaskPatrol {
    pub(crate) fn new(tmpl: At<TmplAiTaskPatrol>) -> InstAiTaskPatrol {
        InstAiTaskPatrol {
            _base: InstAiTaskBase { tmpl_id: tmpl.id },
            action_idle: tmpl.action_idle,
            action_move: tmpl.action_move,
            route: tmpl.route.iter().map(InstAiTaskPatrolStep::from_rkyv).collect(),
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
    fn test_new_inst_ai_task_patrol() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let tmpl = db
            .find_as::<TmplAiTaskPatrol>(id!("AiTask.NpcInstance.Patrol^1"))
            .unwrap();
        let inst = InstAiTaskPatrol::new(tmpl);

        assert_eq!(inst.tmpl_id, id!("AiTask.NpcInstance.Patrol^1"));
        assert_eq!(inst.action_idle, id!("Action.NpcInstance.Idle^1A"));
        assert_eq!(inst.action_move, id!("Action.NpcInstance.Idle^1A"));

        assert_eq!(inst.route.len(), 3);
        assert_eq!(inst.route[0], InstAiTaskPatrolStep::Move(Vec3A::new(-3.0, 0.0, 0.0)));
        assert_eq!(inst.route[1], InstAiTaskPatrolStep::Idle(2.5));
        assert_eq!(inst.route[2], InstAiTaskPatrolStep::Move(Vec3A::new(0.0, 0.0, 3.0)));
    }
}
