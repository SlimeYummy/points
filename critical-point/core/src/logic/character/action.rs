use glam::{Vec2, Vec3A};
use glam_ext::Transform3A;
use std::mem;
use std::rc::Rc;

use super::physics::LogicCharaPhysics;
use crate::animation::SkeletalAnimator;
use crate::consts::{DEFAULT_TOWARD_DIR_2D, MAX_ACTION_ANIMATION};
use crate::instance::{InstActionAny, InstActionIdle, InstPlayer};
use crate::logic::action::{
    new_logic_action, try_reuse_logic_action, ContextAction, DeriveKeeping, LogicActionAny, StateActionAny,
};
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::logic::system::input::InputVariables;
use crate::utils::{
    ifelse, loose_gt, xerr, xres, DtHashSet, HistoryQueue, NumID, Symbol, VirtualDir, VirtualKey, XResult,
};

#[derive(Debug)]
pub(crate) struct LogicCharaAction {
    player_id: NumID,
    inst_player: Rc<InstPlayer>,
    action_queue: HistoryQueue<Box<dyn LogicActionAny>>,
    skeleton_animator: SkeletalAnimator,

    event_cursor_id: u64,
    derive_keeping: Option<DeriveKeeping>,
    new_velocity: Vec3A,
    new_direction: Vec2,
    cache_states: Vec<Box<dyn StateActionAny>>,
}

impl LogicCharaAction {
    pub fn new(
        ctx: &mut ContextUpdate<'_>,
        player_id: NumID,
        inst_player: Rc<InstPlayer>,
    ) -> XResult<LogicCharaAction> {
        let skeleton = ctx.asset.load_skeleton(&inst_player.skeleton_files)?;
        Ok(LogicCharaAction {
            player_id,
            inst_player,
            action_queue: HistoryQueue::with_capacity(4),
            skeleton_animator: SkeletalAnimator::new(
                skeleton,
                SkeletalAnimator::OUT_MODEL_TRANSFORM,
                6,
                4 * MAX_ACTION_ANIMATION,
            ),

            event_cursor_id: 0,
            derive_keeping: None,
            new_velocity: Vec3A::ZERO,
            new_direction: DEFAULT_TOWARD_DIR_2D,
            cache_states: Vec::with_capacity(16),
        })
    }

    pub fn preload_assets(&self, ctx: &mut ContextUpdate<'_>, inst_player: Rc<InstPlayer>) -> XResult<Vec<Symbol>> {
        let mut animations = Vec::with_capacity(16);
        let mut animation_files = DtHashSet::default();
        for action in inst_player.actions.values() {
            action.animations(&mut animations);
            animation_files.extend(animations.drain(..).map(|a| a.files.clone()));
        }
        for anime in animation_files.iter() {
            ctx.asset.load_animation(anime)?;
        }
        Ok(animation_files.into_iter().collect())
    }

    pub fn update(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        chara_physics: &LogicCharaPhysics,
        initing: bool,
    ) -> XResult<()> {
        if self.action_queue.is_empty() && !initing {
            return xres!(Unexpected; "action queue empty");
        }

        let next_act = self.handle_inputs(ctx, chara_physics, initing)?;
        self.update_states(ctx, chara_physics, initing, next_act)?;
        self.apply_animations(ctx)?;
        Ok(())
    }

    fn handle_inputs(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        chara_physics: &LogicCharaPhysics,
        initing: bool,
    ) -> XResult<Option<Rc<dyn InstActionAny>>> {
        if initing {
            let inst_idle: Rc<InstActionIdle> = self
                .inst_player
                .find_first_primary_action(&VirtualKey::Idle)
                .ok_or_else(|| xerr!(NotFound; "idle key"))?;
            return Ok(Some(inst_idle));
        }

        let dk_end = self.derive_keeping.map(|dk| dk.end_time > ctx.time).unwrap_or(false);
        if dk_end {
            self.derive_keeping = None;
        }

        let frame = ctx.frame;
        let player_dir = chara_physics.direction();
        let events = ctx.input.player_events(self.player_id)?;
        let events = events.borrow_mut();
        let current_act = self.action_queue.last().unwrap();
        let mut next_act: Option<Rc<dyn InstActionAny>> = None;

        // Handle preinput events
        for event in events.iter_preinput(frame, self.event_cursor_id)? {
            if event.pressed {
                continue;
            }
            if let Some(candidate) =
                self.find_next_action(current_act, player_dir, event.key.into(), event.world_move_dir)
            {
                if let Some(next_act) = &mut next_act {
                    if candidate.enter_level >= next_act.enter_level {
                        *next_act = candidate;
                    }
                } else {
                    next_act = Some(candidate);
                }
            }
        }

        // Handle current frame events
        for event in events.iter_current(frame)? {
            if event.pressed {
                continue;
            }
            if let Some(candidate) =
                self.find_next_action(current_act, player_dir, event.key.into(), event.world_move_dir)
            {
                if let Some(next_act) = &mut next_act {
                    if candidate.enter_level >= next_act.enter_level {
                        *next_act = candidate;
                    }
                } else {
                    next_act = Some(candidate);
                }
            }
        }

        // No next action found, try Walk/Run.
        if next_act.is_none() {
            let mov = events.variables(frame)?.optimized_world_move();
            if let Some(mov_dir) = mov.move_dir() {
                let mov_key = ifelse!(mov.slow, VirtualKey::Walk, VirtualKey::Run);
                if let Some(candidate) = self.find_next_action(current_act, player_dir, mov_key, mov_dir) {
                    next_act = Some(candidate);
                }
            }
        }

        // Current action ended, enter Idle.
        if next_act.is_none() {
            if !current_act.is_activing() && !current_act.is_starting() {
                let idle: Rc<InstActionIdle> = self
                    .inst_player
                    .find_first_primary_action(&VirtualKey::Idle)
                    .ok_or_else(|| xerr!(NotFound; "idle key"))?;
                next_act = Some(idle);
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
        player_dir: Vec2,
        key: VirtualKey,
        move_dir: Vec2,
    ) -> Option<Rc<dyn InstActionAny>> {
        if let Some(derive_keeping) = self.derive_keeping {
            assert!(current_act.inst.derive_keeping || current_act.tmpl_id() == derive_keeping.action_id);
            let DeriveKeeping {
                action_id,
                derive_level,
                ..
            } = derive_keeping;
            for inst_act in self.inst_player.filter_derive_actions(&(action_id, key)) {
                if Self::check_enter_action(inst_act.as_ref(), derive_level, player_dir, move_dir) {
                    return Some(inst_act);
                }
            }
        }

        for inst_act in self.inst_player.filter_actions(&(current_act.tmpl_id(), key)) {
            let derive_level = match current_act.is_activing() {
                true => current_act.derive_level,
                false => 0, // TODO: error!!!
            };
            if Self::check_enter_action(inst_act.as_ref(), derive_level, player_dir, move_dir) {
                return Some(inst_act);
            }
        }

        None
    }

    fn check_enter_action(
        new_inst_act: &dyn InstActionAny,
        cur_derive_level: u16,
        player_dir: Vec2,
        move_dir: Vec2,
    ) -> bool {
        // Check derive level
        if new_inst_act.enter_level <= cur_derive_level {
            return false;
        }

        // Check enter direction (move combination key)
        let enter_key = match new_inst_act.enter_key {
            Some(enter_key) => enter_key,
            None => return false,
        };
        if let Some(dir) = enter_key.dir {
            let in_range = match dir {
                VirtualDir::Forward(cos) => player_dir.dot(move_dir) > cos,
                VirtualDir::Backward(cos) => (-player_dir).dot(move_dir) > cos,
                VirtualDir::Left(cos) => Vec2::new(-player_dir.y, player_dir.x).dot(move_dir) > cos,
                VirtualDir::Right(cos) => Vec2::new(player_dir.y, -player_dir.x).dot(move_dir) > cos,
            };
            if !in_range {
                return false;
            }
        }

        // TODO: Check custom script

        true
    }

    fn update_states(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        chara_physics: &LogicCharaPhysics,
        initing: bool,
        next_act: Option<Rc<dyn InstActionAny>>,
    ) -> XResult<()> {
        // Clear temporary values
        self.new_velocity = Vec3A::ZERO;
        self.new_direction = chara_physics.direction();
        self.cache_states.clear();
        self.cache_states.reserve(self.action_queue.len() + 1);

        let variables = match initing {
            true => InputVariables::default(),
            false => ctx.input.player_events(self.player_id)?.borrow().variables(ctx.frame)?,
        };
        let mut ctxa = ContextAction::new(self.player_id, chara_physics, variables);

        // Handle next action
        if let Some(next_act) = next_act {
            if let Some(prev_act) = self.action_queue.last_mut() {
                if prev_act.is_activing() || prev_act.is_starting() {
                    prev_act.stop(ctx, &mut ctxa)?;
                }
                ctxa.prev_action = Some(prev_act.inst.clone());
            }

            self.action_queue.enqueue_with(
                ctx,
                |ctx, logic_act| try_reuse_logic_action(logic_act, ctx, next_act.clone()),
                |ctx| new_logic_action(ctx, next_act.clone()),
            )?;

            let current_act = self.action_queue.last_mut().unwrap();
            current_act.start(ctx, &mut ctxa)?;
            if !current_act.inst.derive_keeping {
                self.derive_keeping = None;
            }
        }

        // Update current action
        let current_act = self.action_queue.last_mut().unwrap();
        let ret = current_act.update(ctx, &mut ctxa)?;
        self.cache_states.push(ret.state);
        if let Some(new_velocity) = ret.new_velocity {
            self.new_velocity = new_velocity;
        }
        if let Some(new_direction) = ret.new_direction {
            self.new_direction = new_direction;
        }

        if current_act.is_stopping() {
            self.derive_keeping = ret.derive_keeping;
        }

        // Handle previous actions
        if loose_gt!(current_act.fade_in_weight, 1.0) {
            for logic_act in self.action_queue.iter_mut().rev().skip(1) {
                logic_act.finalize(ctx, &mut ctxa)?;
            }
        } else {
            let mut unused_weight = (1.0 - current_act.fade_in_weight).max(0.0);
            for logic_act in self.action_queue.iter().rev().skip(1) {
                let mut act_state = logic_act.save();
                act_state.fade_in_weight = (unused_weight * act_state.fade_in_weight).clamp(0.0, 1.0);
                unused_weight = (unused_weight - act_state.fade_in_weight).max(0.0);
                self.cache_states.push(act_state);
            }
            self.cache_states.reverse();
        }

        // Clear stopped actions
        self.action_queue.dequeue(|act| act.is_finalized());
        self.action_queue.discard(|act| act.last_frame <= ctx.synced_frame);
        Ok(())
    }

    fn apply_animations(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<()> {
        self.skeleton_animator
            .update(ctx.frame, &self.cache_states, |res| ctx.asset.load_animation(&res))?;
        self.skeleton_animator.animate()?;
        Ok(())
    }

    pub fn restore(&mut self, ctx: &ContextRestore, states: &[Box<dyn StateActionAny>]) -> XResult<()> {
        let mut state_iter = states.iter();
        self.action_queue.restore_when(|act| {
            if act.last_frame < ctx.frame {
                Ok(-1)
            } else if act.first_frame > ctx.frame {
                return Ok(1);
            } else if let Some(state) = state_iter.next() {
                act.restore(state.as_ref())?;
                return Ok(0);
            } else {
                return xres!(LogicBadState; "states order");
            }
        })?;
        self.cache_states.clear();

        self.new_velocity = Vec3A::ZERO;
        self.new_direction = DEFAULT_TOWARD_DIR_2D;
        Ok(())
    }

    pub fn states(&self) -> XResult<&[Box<dyn StateActionAny>]> {
        if self.cache_states.is_empty() {
            return xres!(LogicBadState; "states already taken");
        }
        Ok(&self.cache_states)
    }

    pub fn take_states(&mut self) -> XResult<Vec<Box<dyn StateActionAny>>> {
        if self.cache_states.is_empty() {
            return xres!(LogicBadState; "states already taken");
        }
        Ok(mem::take(&mut self.cache_states))
    }

    #[inline]
    pub(crate) fn model_transforms(&self) -> &[Transform3A] {
        self.skeleton_animator.model_transforms().unwrap()
    }

    #[inline]
    pub(crate) fn new_velocity(&self) -> Vec3A {
        self.new_velocity
    }

    #[inline]
    pub(crate) fn new_direction(&self) -> Vec2 {
        self.new_direction
    }
}
