use approx::abs_diff_ne;
use glam::Vec3A;
use glam_ext::Vec2xz;
use std::rc::Rc;

use crate::consts::DEFAULT_TOWARD_DIR_2D;
use crate::instance::InstActionAny;
use crate::logic::action::{
    ActionStartArgs, ContextAction, DeriveKeeping, LogicActionAny, LogicActionStatus, StateActionAny, new_logic_action,
    try_reuse_logic_action,
};
use crate::logic::ai_task::AiTaskReturn;
use crate::logic::character::physics::LogicCharaPhysics;
use crate::logic::character::value::LogicCharaValue;
use crate::logic::game::ContextUpdate;
use crate::ok_or;
use crate::utils::{InputDir, VirtualKey, XResult, ifelse};

use super::control::*;

impl LogicCharaControl {
    pub(super) fn handle_hit_events(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_phy: &LogicCharaPhysics,
    ) -> XResult<Option<NextAction>> {
        let current_act = self.action_queue.last().unwrap(); // verified
        let player_dir = chara_phy.direction();
        let mut next_act = None;

        for ev_idx in chara_phy.be_hit_events().iter().cloned() {
            let event = &ctx.hit_events[ev_idx];
            let mut dir = Vec2xz::new(event.character_vector.x, event.character_vector.z);
            dir = match abs_diff_ne!(dir, Vec2xz::ZERO) {
                true => dir.normalize(),
                false => -DEFAULT_TOWARD_DIR_2D,
            };
            // TODO: special handle for hits
            next_act = self.find_next_action(current_act, next_act, player_dir, VirtualKey::Hit1, dir);
        }
        Ok(next_act)
    }

    pub(super) fn handle_player_inputs(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_phy: &LogicCharaPhysics,
        mut next_act: Option<NextAction>,
    ) -> XResult<Option<NextAction>> {
        let current_act = self.action_queue.last().unwrap(); // verified
        let frame = ctx.frame;
        let player_dir = chara_phy.direction();
        let events = ctx.input.player_events(self.chara_id)?;
        let events = events.borrow_mut();

        if self.derive_keeping.is_valid() && self.derive_keeping.end_time > ctx.time {
            self.derive_keeping.clear();
        }

        // Handle preinput events
        for event in events.iter_preinput(frame, self.event_cursor_id)? {
            if event.pressed {
                continue;
            }
            next_act = self.find_next_action(current_act, next_act, player_dir, event.key, event.world_move_dir);
        }

        // Handle current frame events
        for event in events.iter_current(frame)? {
            if event.pressed {
                continue;
            }
            next_act = self.find_next_action(current_act, next_act, player_dir, event.key, event.world_move_dir);
        }

        // No next action found, try Walk/Run/Dash.
        if next_act.is_none() {
            let mov = events.variables(frame)?.optimized_world_move();
            if let Some(mov_dir) = mov.move_dir() {
                let mov_key = mov.speed.to_virtual_key();
                next_act = self.find_next_action(current_act, None, player_dir, mov_key, mov_dir);
            }
        }

        if next_act.is_some() {
            self.event_cursor_id = events.future_id(); // Currently, clear preinput after matching a new action
        }
        Ok(next_act)
    }

    fn find_next_action(
        &self,
        current_act: &Box<dyn LogicActionAny>,
        candidate_act: Option<NextAction>,
        player_dir: Vec2xz,
        key: VirtualKey,
        dir: Vec2xz,
    ) -> Option<NextAction> {
        let check_enter_action = |cur_derive_level: u16,
                                  new_inst_act: &dyn InstActionAny,
                                  new_enter_level: u16,
                                  new_enter_dir: Option<InputDir>| {
            // Check not current action
            if !current_act.derive_self() && new_inst_act.tmpl_id == current_act.tmpl_id() {
                return false;
            }

            // Check derive level
            if new_enter_level <= cur_derive_level {
                return false;
            }

            // Check enter direction (move combination key)
            if let Some(new_enter_dir) = new_enter_dir {
                let in_range = match new_enter_dir {
                    InputDir::Forward(cos) => player_dir.dot(dir) > cos,
                    InputDir::Backward(cos) => (-player_dir).dot(dir) > cos,
                    InputDir::Left(cos) => Vec2xz::new(-player_dir.z, player_dir.x).dot(dir) > cos,
                    InputDir::Right(cos) => Vec2xz::new(player_dir.z, -player_dir.x).dot(dir) > cos,
                };
                if !in_range {
                    return false;
                }
            }

            // TODO: Check custom script

            true
        };

        let compare_with_candidate = |new_inst_act: Rc<dyn InstActionAny>, new_enter_level: u16| {
            if let Some(action) = candidate_act.as_ref().map(|x| &x.action) {
                if action.enter_level >= new_enter_level {
                    return candidate_act.clone();
                }
            }
            Some(NextAction::new(new_inst_act, dir, true))
        };

        if self.derive_keeping.is_valid() {
            debug_assert!(current_act.inst.derive_keeping || current_act.tmpl_id() == self.derive_keeping.action_id);
            let DeriveKeeping {
                action_id,
                derive_level,
                ..
            } = self.derive_keeping;
            for (rule, inst_act) in self.inst_chara.filter_derive_actions(&(action_id, key)) {
                if check_enter_action(derive_level, inst_act.as_ref(), rule.level, rule.dir) {
                    return compare_with_candidate(inst_act, rule.level);
                }
            }
        }

        let derive_level = match current_act.is_running() {
            true => current_act.derive_level,
            false => 0, // TODO: error!!!
        };

        for (rule, inst_act) in self.inst_chara.filter_derive_actions(&(current_act.tmpl_id(), key)) {
            if check_enter_action(derive_level, inst_act.as_ref(), rule.level, rule.dir) {
                return compare_with_candidate(inst_act, rule.level);
            }
        }

        for inst_act in self.inst_chara.filter_primary_actions(&key) {
            let enter_level = inst_act.enter_level;
            let enter_dir = inst_act.enter_key.and_then(|k| k.dir);
            if check_enter_action(derive_level, inst_act.as_ref(), enter_level, enter_dir) {
                return compare_with_candidate(inst_act, enter_level);
            }
        }

        candidate_act.clone()
    }

    pub(super) fn make_ctxa_default<'a>(
        &self,
        ctx: &mut ContextUpdate,
        chara_phy: &'a LogicCharaPhysics,
        chara_val: &LogicCharaValue,
    ) -> XResult<ContextAction<'a>> {
        let time_speed = ifelse!(chara_val.hit_lag_time().contains(ctx.time), 0.0, 1.0);
        let mut ctxa = ContextAction::new(self.chara_id, chara_phy);
        ctxa.set_time_normalized(time_speed);
        Ok(ctxa)
    }

    pub(super) fn make_ctxa_from_inputs<'a>(
        &self,
        ctx: &mut ContextUpdate,
        chara_phy: &'a LogicCharaPhysics,
        chara_val: &LogicCharaValue,
    ) -> XResult<ContextAction<'a>> {
        let time_speed = ifelse!(chara_val.hit_lag_time().contains(ctx.time), 0.0, 1.0);
        let mut ctxa = ContextAction::new(self.chara_id, chara_phy);
        ctxa.set_time_normalized(time_speed);

        let events = ctx.input.player_events(self.chara_id)?;
        let events = events.borrow();
        ctxa.set_player_inputs(events.variables(ctx.frame)?, events.future_id())?;
        Ok(ctxa)
    }

    pub(super) fn make_ctxa_from_ai_return<'a>(
        &self,
        ctx: &mut ContextUpdate,
        chara_phy: &'a LogicCharaPhysics,
        chara_val: &LogicCharaValue,
        ai_ret: &AiTaskReturn,
    ) -> ContextAction<'a> {
        let time_speed = ifelse!(chara_val.hit_lag_time().contains(ctx.time), 0.0, 1.0);
        let mut ctxa = ContextAction::new(self.chara_id, chara_phy);
        ctxa.set_time_normalized(time_speed);

        ctxa.optimized_world_move = ai_ret.world_move;
        ctxa.view_dir_2d = ai_ret.view_dir_2d;
        ctxa.view_dir_3d = Vec3A::new(ai_ret.view_dir_2d.x, 0.0, ai_ret.view_dir_2d.z);
        ctxa
    }

    pub(super) fn update_current_actions(
        &mut self,
        ctx: &mut ContextUpdate,
        ctxa: &mut ContextAction,
        chara_phy: &LogicCharaPhysics,
    ) -> XResult<()> {
        // Clear temporary values
        self.new_velocity = Vec3A::ZERO;
        self.new_direction = chara_phy.direction();
        self.action_events = Vec::new();

        let current_act = self.action_queue.last_mut().unwrap(); // verified

        // Update current action
        let ret = current_act.update(ctx, ctxa)?;
        if let Some(new_velocity) = ret.new_velocity {
            self.new_velocity = new_velocity;
        }
        if let Some(new_direction) = ret.new_direction {
            self.new_direction = new_direction;
        }

        if ret.clear_preinput {
            self.event_cursor_id = ctxa.future_id;
        }

        self.action_events = ret.custom_events;

        if current_act.is_stopping() {
            // Trigger derive keeping, when current action actively stops.
            self.derive_keeping = ret.derive_keeping;
        }

        // Update previous fade action
        for act in self.action_queue.iter_mut().rev().take_while(|act| act.is_fading()) {
            act.fade_update(ctx, ctxa)?;
        }
        Ok(())
    }

    pub(super) fn handle_next_action(
        &mut self,
        ctx: &mut ContextUpdate,
        ctxa: &mut ContextAction,
        next_act: Option<NextAction>,
    ) -> XResult<Option<Box<dyn StateActionAny>>> {
        let next_act = match next_act {
            Some(next_act) => next_act,
            None => {
                if let Some(current_act) = self.action_queue.last()
                    && current_act.status.is_active()
                {
                    return Ok(None);
                }
                NextAction::new_idle(self.inst_idle_action.clone())
            }
        };

        self.action_queue.enqueue_with(
            ctx,
            |ctx, logic_act| try_reuse_logic_action(logic_act, ctx, next_act.action.clone()),
            |ctx| new_logic_action(ctx, next_act.action.clone()),
        )?;

        let (prev_act, current_act) = self.action_queue.last2_mut(); // verified
        let prev_act = prev_act.map(|act| act.as_mut());
        let current_act = current_act.unwrap();

        // Start current action
        let ret = {
            let args = ActionStartArgs::new(prev_act.as_deref(), next_act.dir);
            current_act.start(ctx, ctxa, &args)?
        };

        let mut previous_frame_state = current_act.save();
        previous_frame_state.set_previous_frame(true);

        if ret.clear_preinput {
            self.event_cursor_id = ctxa.future_id;
        }

        self.action_events.extend(ret.custom_events);

        // Clear derive keeping, if current action not supported.
        if !current_act.inst.derive_keeping {
            self.derive_keeping.clear();
        }

        // Handle previous action
        if let Some(prev_act) = prev_act {
            let prev_fade_update = ret.prev_fade_update && prev_act.is_running() && prev_act.fade_start(ctx, ctxa)?;
            // println!(
            //     "prev_fade_update: {} {} {}",
            //     prev_fade_update,
            //     ret.prev_fade_update,
            //     prev_act.is_running()
            // );

            if !prev_fade_update {
                let start = self.action_queue.len()
                    - self
                        .action_queue
                        .iter()
                        .rev()
                        .take_while(|act| !act.is_stopping())
                        .count();
                let end = self.action_queue.len() - 1;
                for act in self.action_queue.range_mut(start..end) {
                    act.stop(ctx, ctxa)?;
                }
            }
        }

        Ok(Some(previous_frame_state))
    }

    pub(crate) fn quick_switch_next_action(
        &mut self,
        ctx: &mut ContextUpdate,
        ctxa: &mut ContextAction,
        chara_phy: &LogicCharaPhysics,
    ) -> XResult<Option<Box<dyn StateActionAny>>> {
        let current_act = ok_or!(self.action_queue.last(); return Ok(None));

        let mut state = current_act.save();
        state.set_previous_frame(true);
        debug_assert_eq!(state.fade_in_weight, 0.0, "quick_switch_next_action: {}", state.tmpl_id);

        self.update_current_actions(ctx, ctxa, chara_phy)?;
        Ok(Some(state))
    }

    pub(super) fn collect_states_and_cleanup(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_phy: &LogicCharaPhysics,
        previous_frame_state: Option<Box<dyn StateActionAny>>,
    ) -> XResult<()> {
        self.cache_states.clear();
        self.cache_states.reserve(self.action_queue.len());

        // Collect states
        let mut unused_weight = 1.0;
        let mut zero_count = 0;
        for logic_act in self.action_queue.iter_mut().rev() {
            if unused_weight <= 0.0 {
                zero_count += 1;
            }

            let mut act_state = logic_act.save();
            #[cfg(debug_assertions)]
            {
                debug_assert!(
                    act_state.fade_in_weight >= 0.0 && act_state.fade_in_weight <= 1.0,
                    "{}",
                    act_state.tmpl_id
                );
                for anim_state in &act_state.animations {
                    if !anim_state.is_empty() {
                        debug_assert!(
                            anim_state.weight >= 0.0 && anim_state.weight <= 1.0,
                            "{} {}",
                            act_state.tmpl_id,
                            anim_state.files
                        );
                        debug_assert!(
                            anim_state.ratio >= 0.0 && anim_state.ratio <= 1.0,
                            "{} {}",
                            act_state.tmpl_id,
                            anim_state.files
                        );
                    }
                }
            }

            act_state.fade_in_weight = (unused_weight * act_state.fade_in_weight).clamp(0.0, 1.0);
            unused_weight = (unused_weight - act_state.fade_in_weight).max(0.0);
            self.cache_states.push(act_state);
        }
        self.cache_states.reverse();

        if let Some(previous_frame_state) = previous_frame_state {
            // Insert previous frame state before current frame state.
            self.cache_states.push(previous_frame_state);
            let len = self.cache_states.len();
            self.cache_states.swap(len - 1, len - 2);
        }

        // Finalize actions
        let mut ctxa = ContextAction::new(self.chara_id, chara_phy);
        ctxa.set_time_normalized(1.0);
        for idx in 0..zero_count {
            if self.action_queue[idx].is_fading() {
                self.action_queue[idx].stop(ctx, &mut ctxa)?;
            }
            self.action_queue[idx].finalize(ctx, &mut ctxa)?;
        }

        // Clear unused actions
        self.action_queue.dequeue(|act| act.is_finalized());
        self.action_queue.discard(|act| act.last_frame <= ctx.synced_frame);
        Ok(())
    }
}
