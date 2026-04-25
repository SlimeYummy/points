use critical_point_csgen::CsOut;
use std::fmt::Debug;
use std::rc::Rc;

use crate::instance::{InstActionIdle, InstAiTaskIdle, InstCharacter};
use crate::logic::ai_task::base::{
    AiTaskReturn, ContextAiTask, LogicAiTaskAny, LogicAiTaskBase, StateAiTaskAny, StateAiTaskBase, impl_state_ai_task,
};
use crate::logic::game::ContextUpdate;
use crate::utils::{AiTaskType, Castable, XResult, extend, ok_or, xres, xresf};

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
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
            None => return xres!(InstNotFound, inst_task.action_idle),
        };
        Ok(LogicAiTaskIdle {
            _base: LogicAiTaskBase::new(ctx.gene.gen_ai_task_id(), inst_task.clone()),
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

    fn restore(&mut self, state: &(dyn StateAiTaskAny + 'static)) -> XResult<()> {
        if state.id() != self._base.id {
            return xresf!(LogicIDMismatch; "state.id={}, self.id={}", state.id(), self._base.id);
        }
        let state = state.cast::<StateAiTaskIdle>()?;
        self._base.restore(&state._base);
        Ok(())
    }

    fn start(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self._base.start(ctx, ctxt)?;
        Ok(AiTaskReturn::default())
    }

    fn update(&mut self, ctx: &mut ContextUpdate, ctxt: &mut ContextAiTask) -> XResult<AiTaskReturn> {
        self._base.update(ctx, ctxt)?;
        Ok(AiTaskReturn::default())
    }

    fn save(&self) -> Box<dyn StateAiTaskAny> {
        Box::new(StateAiTaskIdle {
            _base: self._base.save(self.typ()),
        })
    }
}
