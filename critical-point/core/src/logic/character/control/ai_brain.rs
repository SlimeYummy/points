use critical_point_csgen::CsOut;
use std::rc::Rc;

use crate::instance::{InstAiBrain, InstAiNode, InstCharacter};
use crate::logic::ai_task::{AiTaskReturn, ContextAiTask, new_logic_ai_task};
use crate::logic::character::physics::LogicCharaPhysics;
use crate::logic::character::value::LogicCharaValue;
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::utils::{NumID, XResult, ifelse, ok_or};

use super::control::*;

impl LogicCharaControl {
    pub(super) fn handle_ai_all(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_phy: &LogicCharaPhysics,
        chara_val: &LogicCharaValue,
    ) -> XResult<AiTaskReturn> {
        if self.current_task.is_some() {
            self.update_current_ai_task(ctx, chara_phy, chara_val)
        }
        else {
            self.find_next_ai_task(ctx)?;
            if self.current_task.is_some() {
                self.start_current_ai_task(ctx, chara_phy, chara_val)
            }
            else {
                Ok(AiTaskReturn::default())
            }
        }
    }

    fn find_next_ai_task(&mut self, ctx: &mut ContextUpdate) -> XResult<()> {
        let inst_ai_brain = ok_or!(self.inst_ai_brain.as_ref(); return Ok(()));

        inst_ai_brain.travel_idle(|task| {
            match task {
                InstAiNode::Task(_, task) => {
                    self.current_task = Some(new_logic_ai_task(ctx, (*task).clone(), self.inst_chara.clone())?);
                }
                InstAiNode::Branch(_) => {}
            }
            Ok(())
        })
    }

    fn start_current_ai_task(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_phy: &LogicCharaPhysics,
        chara_val: &LogicCharaValue,
    ) -> XResult<AiTaskReturn> {
        let zone = ok_or!(ctx.zone; return Ok(AiTaskReturn::default()));

        let mut task = self.current_task.take().unwrap();
        let res: XResult<AiTaskReturn> = try {
            let time_speed = ifelse!(chara_val.hit_lag_time().contains(ctx.time), 0.0, 1.0);
            let mut ctxt = ContextAiTask::new(self.chara_id.clone(), self, chara_phy, zone, time_speed);
            task.start(ctx, &mut ctxt)?
        };
        self.current_task = Some(task);

        res
    }

    fn update_current_ai_task(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_phy: &LogicCharaPhysics,
        chara_val: &LogicCharaValue,
    ) -> XResult<AiTaskReturn> {
        let zone = ok_or!(ctx.zone; return Ok(AiTaskReturn::default()));

        let mut task = self.current_task.take().unwrap();
        let res: XResult<AiTaskReturn> = try {
            let time_speed = ifelse!(chara_val.hit_lag_time().contains(ctx.time), 0.0, 1.0);
            let mut ctxt = ContextAiTask::new(self.chara_id.clone(), self, chara_phy, zone, time_speed);
            task.update(ctx, &mut ctxt)?
        };
        self.current_task = Some(task);

        res
    }
}
