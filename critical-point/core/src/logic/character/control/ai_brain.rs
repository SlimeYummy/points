use crate::logic::ai_task::{AiTaskReturn, ContextAiTask, new_logic_ai_task};
use crate::logic::character::physics::LogicCharaPhysics;
use crate::logic::character::value::LogicCharaValue;
use crate::logic::game::ContextUpdate;
use crate::logic::system::StateSet;
use crate::utils::{NumID, TmplID, XResult, ok_or};

use super::control::*;

impl LogicCharaControl {
    fn update_ai_target(&mut self, ctx: &mut ContextUpdate, chara_phy: &LogicCharaPhysics) {
        let state_set = ok_or!(ctx.systems.state.get(ctx.frame.wrapping_sub(1)); return);
        let inst_ai_brain = ok_or!(self.inst_ai_brain.as_ref(); return);

        if self.target_chara_id.is_valid() {
            if let Some(idx) = state_set
                .chara_updates
                .iter()
                .position(|state| state.id == self.target_chara_id)
            {
                let target_pos = state_set.chara_updates[idx].physics.position;
                let dist_sq = (chara_phy.position() - target_pos).length_squared();

                // Target in aggro sphere.
                if dist_sq <= inst_ai_brain.aggro_sphere.radius_sq() {
                    self.aggro_last_time = ctx.time;
                }
                // Target out of aggro sphere, and lost time passed.
                else if ctx.time - self.aggro_last_time > inst_ai_brain.aggro_lost_time {
                    self.target_chara_id = NumID::INVALID;
                }

                // We still have a target.
                if self.target_chara_id.is_valid() {
                    self.ai_thinking.target_chara = self.target_chara_id;
                    self.ai_thinking.target_chara_idx = idx as u32;
                    self.ai_thinking.target_chara_pos = target_pos;
                    return;
                }
            }
            else {
                // Target dead.
                self.target_chara_id = NumID::INVALID;
            }
        }

        // Find new target.
        self.tmp_state_indexes.clear();

        // Find in alert cone first.
        state_set.search_chara_in_spherical_cone(
            true,
            &inst_ai_brain.alert_cone,
            chara_phy.position(),
            chara_phy.direction_xz(),
            &mut self.tmp_state_indexes,
        );
        if self.tmp_state_indexes.is_empty() {
            // Find in aggro sphere.
            state_set.search_chara_in_sphere(
                true,
                &inst_ai_brain.aggro_sphere,
                chara_phy.position(),
                &mut self.tmp_state_indexes,
            );
        }

        if !self.tmp_state_indexes.is_empty() {
            let rand = ctx.systems.rand.rand_u32() as usize % self.tmp_state_indexes.len();
            let idx = self.tmp_state_indexes[rand] as usize;
            let target = &state_set.chara_updates[idx];

            self.target_chara_id = target.id;
            self.aggro_last_time = ctx.time;

            self.ai_thinking.target_chara = self.target_chara_id;
            self.ai_thinking.target_chara_idx = idx as u32;
            self.ai_thinking.target_chara_pos = target.physics.position;
        }
        else {
            self.target_chara_id = NumID::INVALID;
            self.aggro_last_time = 0.0;
            self.ai_thinking.reset();
        }

        self.tmp_state_indexes.clear();
    }

    pub(super) fn handle_ai_all(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_phy: &LogicCharaPhysics,
        chara_val: &LogicCharaValue,
    ) -> XResult<AiTaskReturn> {
        self.update_ai_target(ctx, chara_phy);

        let mut ai_ret = AiTaskReturn::default();
        if self.current_task.is_some() {
            ai_ret = self.update_current_ai_task(ctx, chara_phy, chara_val)?;
        }

        // update may set current_task to None
        if self.current_task.is_none() {
            self.find_next_ai_task(ctx, chara_val)?;
            if self.current_task.is_some() {
                ai_ret = self.start_current_ai_task(ctx, chara_phy, chara_val)?;
            }
        }

        self.ai_thinking.purpose = ai_ret.ai_purpose;
        self.ai_thinking.move_dst_pos = ai_ret.ai_move_dst_pos;
        self.ai_thinking.move_dir = ai_ret.ai_move_dir;
        Ok(ai_ret)
    }

    fn find_next_ai_task(&mut self, ctx: &mut ContextUpdate, chara_val: &LogicCharaValue) -> XResult<()> {
        let inst_ai_brain = ok_or!(self.inst_ai_brain.as_ref(); return Ok(()));
        let func = ok_or!(self.ai_brain_execute.clone(); return Ok(()));

        let tasks = ctx.script.call_ai_brain_execute(func, &chara_val.ws)?.to_vec();

        for task in tasks.iter() {
            if let Some(inst_task) = inst_ai_brain.tasks.get(&task.id) {
                self.current_task = Some(new_logic_ai_task(ctx, inst_task.clone(), self.inst_chara.clone())?);
            }
        }

        Ok(())
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
            let mut ctxt = ContextAiTask::new(self.inst_chara.clone(), self, chara_phy, zone);
            ctxt.set_time_normalized(chara_val.time_speed());
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

        let mut ctxt = ContextAiTask::new(self.inst_chara.clone(), self, chara_phy, zone);
        ctxt.set_time_normalized(chara_val.time_speed());

        let res = task.update(ctx, &mut ctxt)?;

        self.current_task = match task.is_stopping() {
            true => {
                task.finalize(ctx, &mut ctxt)?;
                None
            }
            false => Some(task),
        };
        Ok(res)
    }
}
