use crate::instance::ai_task::base::{InstAiTaskAny, InstAiTaskBase};
use crate::template::{At, TmplAiTaskKeepDistance};
use crate::utils::{AiIntention, AiTaskType, TmplID, extend};

#[repr(C)]
#[derive(Debug)]
pub struct InstAiTaskKeepDistance {
    pub _base: InstAiTaskBase,
    pub intention: AiIntention,
    pub next_intention: AiIntention,
    pub dodge_action: TmplID,
    pub expected_distance: f32,
}

extend!(InstAiTaskKeepDistance, InstAiTaskBase);

unsafe impl InstAiTaskAny for InstAiTaskKeepDistance {
    #[inline]
    fn typ(&self) -> AiTaskType {
        AiTaskType::KeepDistance
    }

    #[inline]
    fn actions(&self, actions: &mut Vec<TmplID>) {
        actions.push(self.dodge_action);
    }
}

impl InstAiTaskKeepDistance {
    pub(crate) fn new(tmpl: At<TmplAiTaskKeepDistance>) -> InstAiTaskKeepDistance {
        InstAiTaskKeepDistance {
            _base: InstAiTaskBase { tmpl_id: tmpl.id },
            intention: tmpl.intention,
            next_intention: tmpl.next_intention,
            dodge_action: tmpl.dodge_action,
            expected_distance: tmpl.expected_distance.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_new_ai_task_keep_distance() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let tmpl = db
            .find_as::<TmplAiTaskKeepDistance>(id!("AiTask.InstanceNpc.KeepDistance^1"))
            .unwrap();
        let inst = InstAiTaskKeepDistance::new(tmpl);

        assert_eq!(inst.tmpl_id, id!("AiTask.InstanceNpc.KeepDistance^1"));
        assert_eq!(inst.intention, AiIntention::Attack);
        assert_eq!(inst.next_intention, AiIntention::SquareOff);
        assert_eq!(inst.dodge_action, id!("Action.InstanceNpc.Dodge"));
        assert_eq!(inst.expected_distance, 0.0);
    }
}
