use critical_point_csgen::CsOut;
use std::rc::Rc;

use crate::instance::{InstAiBrain, InstAiNode, InstCharacter};
use crate::logic::ai_task::{ContextAiTask, LogicAiTaskAny};
use crate::logic::character::action::LogicCharaAction;
use crate::logic::character::physics::LogicCharaPhysics;
use crate::logic::character::value::LogicCharaValue;
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::utils::{ifelse, ok_or, NumID, XResult};

#[repr(C)]
#[derive(
    Debug,
    Default,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Value)]
pub struct StateCharaAiBrain {}

#[derive(Debug)]
pub(crate) struct LogicCharaAiBrain {
    chara_id: NumID,
    inst_chara: Rc<InstCharacter>,
    inst_ai_brain: Rc<InstAiBrain>,

    current_task: Option<Box<dyn LogicAiTaskAny>>,
}

impl LogicCharaAiBrain {
    pub(crate) fn new(
        _ctx: &mut ContextUpdate,
        chara_id: NumID,
        inst_chara: Rc<InstCharacter>,
        inst_ai_brain: Rc<InstAiBrain>,
    ) -> LogicCharaAiBrain {
        LogicCharaAiBrain {
            chara_id,
            inst_chara,
            inst_ai_brain,

            current_task: None,
        }
    }

    pub(crate) fn state(&self) -> StateCharaAiBrain {
        StateCharaAiBrain {}
    }

    pub(crate) fn restore(&mut self, _ctx: &ContextRestore, _state: &StateCharaAiBrain) -> XResult<()> {
        Ok(())
    }

    #[inline]
    pub(crate) fn init(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_phy: &LogicCharaPhysics,
        chara_action: &LogicCharaAction,
        chara_val: &LogicCharaValue,
    ) -> XResult<()> {
        // self.find_next_task(ctx)?;
        // self.run_current_task(ctx, chara_phy, chara_action, chara_val)?;
        Ok(())
    }

    #[inline]
    pub(crate) fn update(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_phy: &LogicCharaPhysics,
        chara_action: &LogicCharaAction,
        chara_val: &LogicCharaValue,
    ) -> XResult<()> {
        self.run_current_task(ctx, chara_phy, chara_action, chara_val)?;
        if self.current_task.is_none() {
            self.find_next_task(ctx)?;
        }
        Ok(())
    }
}

impl LogicCharaAiBrain {
    fn find_next_task(&mut self, ctx: &mut ContextUpdate) -> XResult<()> {
        self.inst_ai_brain.travel_idle(|task| {
            match task {
                InstAiNode::Task(_, task) => {
                    self.current_task = Some(crate::logic::ai_task::new_logic_ai_task(ctx, (*task).clone())?);
                }
                InstAiNode::Branch(_) => {}
            }
            Ok(())
        })?;
        Ok(())
    }

    fn run_current_task(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_phy: &LogicCharaPhysics,
        chara_action: &LogicCharaAction,
        chara_val: &LogicCharaValue,
    ) -> XResult<()> {
        let zone = ok_or!(ctx.zone; return Ok(()));
        let task = ok_or!(&mut self.current_task; return Ok(()));

        let time_speed = ifelse!(chara_val.hit_lag_time().contains(ctx.time), 0.0, 1.0);
        let mut ctxt = ContextAiTask::new(self.chara_id, chara_phy, chara_action, zone, time_speed);

        task.update(ctx, &mut ctxt)?;
        Ok(())
    }
}
