use std::rc::Rc;
use std::usize;

use crate::instance::{InstAiRoutine, InstAiRoutineItem, InstAiTaskAny};
use crate::logic::ai_task::{AiTaskReturn, ContextAiTask, new_logic_ai_task};
use crate::logic::base::LogicAny;
use crate::logic::character::physics::LogicCharaPhysics;
use crate::logic::character::value::LogicCharaValue;
use crate::logic::game::ContextUpdateEx;
use crate::utils::{NumID, TmplID, XResult, ok_or};
use crate::xresf;

use super::control::*;

impl LogicCharaControl {
    pub(super) fn update_ai_target(&mut self, ctx: &mut ContextUpdateEx, chara_phy: &LogicCharaPhysics) {
        let inst_ai_brain = ok_or!(self.inst_ai_brain.as_ref(); return);

        self.ai_thinking.reset();
        let old_target_chara = self.target_chara;

        if self.target_chara.is_valid() {
            if let Some(idx) = ctx.characters.iter().position(|c| c.id() == self.target_chara) {
                let target_pos = ctx.characters[idx].physics().position();
                let dist_sq = (chara_phy.position() - target_pos).length_squared();

                if dist_sq <= inst_ai_brain.aggro_sphere.radius_sq() {
                    // Target in aggro sphere.
                    self.aggro_last_time = ctx.time.time;
                }
                else if ctx.time.time - self.aggro_last_time > inst_ai_brain.aggro_lost_time {
                    // Target out of aggro sphere, and lost time passed, clear target.
                    self.target_chara = NumID::INVALID;
                }

                // We still have a target.
                if self.target_chara.is_valid() {
                    debug_assert_eq!(self.target_chara, old_target_chara);
                    self.ai_thinking.target_chara = self.target_chara;
                    self.ai_thinking.target_changed = false;
                    self.ai_thinking.target_chara_idx = idx as u32;
                    self.ai_thinking.target_chara_pos = target_pos;
                    return;
                }
            }
            else {
                // Target dead, do nothing.
            }
        }

        self.tmp_target_indexes.clear();
        // Find new target in alert cone first.
        ctx.characters.search_chara_in_spherical_cone(
            true,
            &inst_ai_brain.alert_cone,
            chara_phy.position(),
            chara_phy.direction_xz(),
            &mut self.tmp_target_indexes,
        );
        if self.tmp_target_indexes.is_empty() {
            // Find new target in aggro sphere.
            ctx.characters.search_chara_in_sphere(
                true,
                &inst_ai_brain.aggro_sphere,
                chara_phy.position(),
                &mut self.tmp_target_indexes,
            );
        }

        if !self.tmp_target_indexes.is_empty() {
            let rand = ctx.systems.rand.rand_u32() as usize % self.tmp_target_indexes.len();
            let idx = self.tmp_target_indexes[rand] as usize;
            let target = ctx.characters[idx].as_ref();

            self.aggro_last_time = ctx.time.time;

            self.target_chara = target.id();
            self.ai_thinking.target_chara = self.target_chara;
            self.ai_thinking.target_chara_idx = idx as u32;
            self.ai_thinking.target_chara_pos = target.physics().position();
        }
        else {
            self.aggro_last_time = 0.0;
        }
        self.ai_thinking.target_changed = self.target_chara != old_target_chara;
        if self.ai_thinking.target_changed {
            log::info!(
                "LogicCharaControl::update_ai_target(), chara_id={}, target_id={}",
                self.chara_id,
                self.target_chara
            );
        }

        self.tmp_target_indexes.clear();
    }

    pub(super) fn handle_ai_all(
        &mut self,
        ctx: &mut ContextUpdateEx,
        chara_phy: &LogicCharaPhysics,
        chara_val: &LogicCharaValue,
    ) -> XResult<AiTaskReturn> {
        let mut ai_ret = self.update_current_ai_task(ctx, chara_phy, chara_val)?;

        if let Some(next_task) = self.execute_ai_routine(ctx, chara_phy, chara_val)? {
            ai_ret = self.start_ai_task(ctx, chara_phy, chara_val, next_task)?;
        }

        if self.current_task.is_none() {
            match self.call_ai_brain_execute(ctx, chara_phy, chara_val)? {
                ExecuteResult::Routine(routine) => {
                    if let Some(next_task) = self.start_ai_routine(ctx, routine, chara_phy, chara_val)? {
                        ai_ret = self.start_ai_task(ctx, chara_phy, chara_val, next_task)?;
                    }
                }
                ExecuteResult::Task(task) => {
                    ai_ret = self.start_ai_task(ctx, chara_phy, chara_val, task)?;
                }
                ExecuteResult::None => {}
            }
        }

        self.ai_thinking.move_dst_pos = ai_ret.ai_move_dst_pos;
        self.ai_thinking.move_dir = ai_ret.ai_move_dir;
        Ok(ai_ret)
    }

    fn update_current_ai_task(
        &mut self,
        ctx: &mut ContextUpdateEx,
        chara_phy: &LogicCharaPhysics,
        chara_val: &LogicCharaValue,
    ) -> XResult<AiTaskReturn> {
        let mut task = ok_or!(self.current_task.take(); return Ok(AiTaskReturn::default()));

        let mut ctxt = ContextAiTask::new(self.inst_chara.clone(), self, chara_phy, &self.ai_thinking, ctx.zone);
        ctxt.set_time_normalized(chara_val.time_speed());

        let res = task.update(ctx, &mut ctxt)?;

        if task.is_stopping() {
            task.finalize(ctx, &mut ctxt)?;
            self.ws.ai_intention = task.intention;
            self.current_task = None;
        }
        else {
            self.ws.ai_intention = task.intention;
            self.current_task = Some(task);
        }
        Ok(res)
    }

    fn call_ai_brain_execute(
        &mut self,
        ctx: &mut ContextUpdateEx,
        chara_phy: &LogicCharaPhysics,
        chara_val: &LogicCharaValue,
    ) -> XResult<ExecuteResult> {
        let ai_brain = ok_or!(self.inst_ai_brain.as_ref(); return Ok(ExecuteResult::None));
        let func_execute = ok_or!(self.ai_brain_execute.clone(); return Ok(ExecuteResult::None));

        let tgt_chara = match self.ai_thinking.target_chara.is_valid() {
            true => Some(&ctx.characters[self.ai_thinking.target_chara_idx as usize]),
            false => None,
        };

        self.tmp_ai_do_list.clear();
        ctx.systems.script.call_ai_brain_execute(
            func_execute,
            &self.ws,
            chara_phy.ws(),
            chara_val.ws(),
            tgt_chara.map(|c| c.physics().ws()),
            tgt_chara.map(|c| c.value().ws()),
            &mut self.tmp_ai_do_list,
        )?;

        for candidate in self.tmp_ai_do_list.drain(..) {
            if candidate.id.prefix == crate::utils::TmplPrefix::AiRoutine {
                let new_routine = match ai_brain.routines.get(&candidate.id) {
                    Some(routine) => routine,
                    None => {
                        log::warn!(
                            "LogicCharaControl::call_ai_brain_execute(), chara_id={}, ai_brain={:?}, routine_id={}, not found",
                            self.chara_id,
                            self.inst_chara.ai_brain.as_ref().map(|b| b.tmpl_id),
                            candidate.id
                        );
                        continue;
                    }
                };
                return Ok(ExecuteResult::Routine(new_routine.clone()));
            }
            else {
                let new_task = match ai_brain.tasks.get(&candidate.id) {
                    Some(task) => task,
                    None => {
                        log::warn!(
                            "LogicCharaControl::call_ai_brain_execute(), chara_id={}, ai_brain={:?}, task_id={}, not found",
                            self.chara_id,
                            self.inst_chara.ai_brain.as_ref().map(|b| b.tmpl_id),
                            candidate.id
                        );
                        continue;
                    }
                };
                return Ok(ExecuteResult::Task(new_task.clone()));
            }
        }
        Ok(ExecuteResult::None)
    }

    fn start_ai_task(
        &mut self,
        ctx: &mut ContextUpdateEx,
        chara_phy: &LogicCharaPhysics,
        chara_val: &LogicCharaValue,
        inst_task: Rc<dyn InstAiTaskAny>,
    ) -> XResult<AiTaskReturn> {
        if let Some(mut old_task) = self.current_task.take() {
            let mut ctxt = ContextAiTask::new(self.inst_chara.clone(), self, chara_phy, &self.ai_thinking, ctx.zone);
            ctxt.set_time_normalized(chara_val.time_speed());
            old_task.stop(ctx, &mut ctxt)?;
            old_task.finalize(ctx, &mut ctxt)?;
        }

        let mut task = new_logic_ai_task(ctx, inst_task, self.inst_chara.clone())?;

        let mut ctxt = ContextAiTask::new(self.inst_chara.clone(), self, chara_phy, &self.ai_thinking, ctx.zone);
        ctxt.set_time_normalized(chara_val.time_speed());
        let ret = task.start(ctx, &mut ctxt)?;

        if task.is_stopping() {
            task.finalize(ctx, &mut ctxt)?;
            self.ws.current_task = TmplID::INVALID;
            self.ws.ai_intention = task.intention;
            self.current_task = None;
        }
        else {
            self.ws.current_task = task.inst.tmpl_id;
            self.ws.ai_intention = task.intention;
            self.current_task = Some(task);
        }
        Ok(ret)
    }

    fn start_ai_routine(
        &mut self,
        ctx: &mut ContextUpdateEx,
        routine: Rc<InstAiRoutine>,
        chara_phy: &LogicCharaPhysics,
        chara_val: &LogicCharaValue,
    ) -> XResult<Option<Rc<dyn InstAiTaskAny>>> {
        self.current_routine = Some(routine.clone());
        self.current_routine_exec = 0;
        self.ws.current_routine = routine.tmpl_id;
        self.execute_ai_routine(ctx, chara_phy, chara_val)
    }

    fn execute_ai_routine(
        &mut self,
        ctx: &mut ContextUpdateEx,
        chara_phy: &LogicCharaPhysics,
        chara_val: &LogicCharaValue,
    ) -> XResult<Option<Rc<dyn InstAiTaskAny>>> {
        if self.current_task.is_some() {
            return Ok(None); // Have a running task, do not update routine.
        }

        let ai_brain = ok_or!(self.inst_ai_brain.clone(); return Ok(None));
        let routine = ok_or!(self.current_routine.clone(); return Ok(None));
        let tasks = &routine.tasks;

        let tgt_chara = match self.ai_thinking.target_chara.is_valid() {
            true => Some(&ctx.characters[self.ai_thinking.target_chara_idx as usize]),
            false => None,
        };

        let mut loop_count = 0;
        while (self.current_routine_exec as usize) < tasks.len() {
            loop_count += 1;
            if loop_count > 100 {
                return xresf!(LogicException; "routine={}, AI routine infinite loop", routine.tmpl_id);
            }

            match &tasks[self.current_routine_exec as usize] {
                InstAiRoutineItem::Task { id } => {
                    self.current_routine_exec += 1;
                    return match ai_brain.tasks.get(id) {
                        Some(task) => Ok(Some(task.clone())),
                        None => xresf!(LogicNotFound; "routine={}, task={}, not found", routine.tmpl_id, id),
                    };
                }
                InstAiRoutineItem::If { script, jump } => {
                    let func = ctx.systems.script.get_ai_routine_if(routine.tmpl_id, *script)?;
                    let res = ctx.systems.script.call_ai_routine_if(
                        func,
                        &self.ws,
                        chara_phy.ws(),
                        chara_val.ws(),
                        tgt_chara.map(|c| c.physics().ws()),
                        tgt_chara.map(|c| c.value().ws()),
                    )?;
                    if res {
                        self.current_routine_exec += 1;
                    }
                    else {
                        self.current_routine_exec = *jump;
                    }
                }
                InstAiRoutineItem::Else { jump } => {
                    self.current_routine_exec = *jump;
                }
            }
        }

        self.ws.current_routine = TmplID::INVALID;
        Ok(None)
    }
}

#[derive(Debug)]
enum ExecuteResult {
    Task(Rc<dyn InstAiTaskAny>),
    Routine(Rc<InstAiRoutine>),
    None,
}
