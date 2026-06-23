use critical_point_macros::csharp_out;
use std::fmt::Debug;
use std::rc::Rc;

use crate::instance::{InstActionIdle, InstAiTaskIdle, InstCharacter};
use crate::logic::ai_task::base::{
    AiBrainPurpose, AiBrainThinking, AiTaskReturn, ContextAiTask, LogicAiTaskAny, LogicAiTaskBase, StateAiTaskAny,
    StateAiTaskBase, impl_state_ai_task,
};
use crate::logic::game::ContextUpdate;
use crate::utils::{AiTaskType, Castable, XResult, extend, xres};

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct StateAiTaskIdle {
    pub _base: StateAiTaskBase,
}

extend!(StateAiTaskIdle, StateAiTaskBase);
impl_state_ai_task!(StateAiTaskIdle, Idle, "Idle");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicAiTaskIdle {
    _base: LogicAiTaskBase,
    inst: Rc<InstAiTaskIdle>,
    inst_idle: Rc<InstActionIdle>,
}

extend!(LogicAiTaskIdle, LogicAiTaskBase);

impl LogicAiTaskIdle {
    pub fn new(
        ctx: &mut ContextUpdate,
        inst_task: Rc<InstAiTaskIdle>,
        inst_chara: Rc<InstCharacter>,
    ) -> XResult<LogicAiTaskIdle> {
        let inst_idle: Rc<InstActionIdle> = match inst_chara.actions.get(&inst_task.action_idle) {
            Some(inst) => inst.clone().cast()?,
            None => return xres!(InstNotFound),
        };

        Ok(LogicAiTaskIdle {
            _base: LogicAiTaskBase::new(ctx.identity.gen_ai_task_id(), inst_task.clone()),
            inst: inst_task,
            inst_idle,
        })
    }
}

unsafe impl LogicAiTaskAny for LogicAiTaskIdle {
    #[inline]
    fn typ(&self) -> AiTaskType {
        AiTaskType::Idle
    }

    fn save(&self) -> Box<dyn StateAiTaskAny> {
        Box::new(StateAiTaskIdle {
            _base: self._base.save(self.typ()),
        })
    }

    fn restore(&mut self, state: &(dyn StateAiTaskAny + 'static)) -> XResult<()> {
        if state.id() != self._base.id {
            return xres!(LogicIDMismatch);
        }
        let state = state.cast::<StateAiTaskIdle>()?;
        self._base.restore(&state._base);
        Ok(())
    }

    fn start(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self._base.start(ctx, ctxt)?;
        let mut ret = AiTaskReturn::default();
        ret.next_action = Some(self.inst_idle.clone());
        ret.ai_purpose = AiBrainPurpose::None;
        Ok(ret)
    }

    fn update(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self._base.update(ctx, ctxt)?;

        if self.current_action != self.inst_idle.tmpl_id {
            self.stop(ctx, ctxt)?;
            return Ok(AiTaskReturn::default());
        }

        let mut ret = AiTaskReturn::default();
        ret.ai_purpose = AiBrainPurpose::None;
        Ok(ret)
    }
}
