use glam::Vec3A;
use glam_ext::{Transform3A, Vec2xz};
use std::mem;
use std::rc::Rc;
use std::collections::hash_map::Entry;

use super::physics::LogicCharaPhysics;
use crate::animation::{rest_poses_to_model_transforms, Animator};
use crate::consts::{DEFAULT_TOWARD_DIR_2D, MAX_ACTION_ANIMATION, MAX_ACTION_STATE};
use crate::instance::{InstActionAny, InstActionIdle, InstPlayer};
use crate::logic::action::{
    new_logic_action, try_reuse_logic_action, ContextAction, DeriveKeeping, LogicActionAny, StateActionAny,
};
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::logic::system::input::InputVariables;
use crate::utils::{ifelse, xerr, xres, DtHashMap, HistoryQueue, NumID, AnimationFileMeta, VirtualDir, VirtualKey, XResult};

#[derive(Debug)]
pub(crate) struct LogicCharaAction {
    player_id: NumID,
    inst_player: Rc<InstPlayer>,
    inst_idle_action: Rc<InstActionIdle>,
    action_queue: HistoryQueue<Box<dyn LogicActionAny>>,
    animator: Animator,
    model_transforms: Vec<Transform3A>,

    event_cursor_id: u64,
    derive_keeping: Option<DeriveKeeping>,
    new_velocity: Vec3A,
    new_direction: Vec2xz,
    cache_states: Vec<Box<dyn StateActionAny>>,
}

impl LogicCharaAction {
    pub fn new(
        ctx: &mut ContextUpdate<'_>,
        player_id: NumID,
        inst_player: Rc<InstPlayer>,
    ) -> XResult<LogicCharaAction> {
        let skeleton = ctx.asset.load_skeleton(inst_player.skeleton_files)?;
        let mut model_transforms = vec![Transform3A::ZERO; skeleton.num_joints()];
        rest_poses_to_model_transforms(&skeleton, &mut model_transforms)?;

        let inst_idle_action: Rc<InstActionIdle> = inst_player
            .find_first_primary_action(&VirtualKey::Idle)
            .ok_or_else(|| xerr!(NotFound; "No idle action"))?;

        Ok(LogicCharaAction {
            player_id,
            inst_player,
            inst_idle_action,
            action_queue: HistoryQueue::with_capacity(4),
            animator: Animator::new(
                skeleton, 2,
                4,
                // ((MAX_ACTION_STATE as f32) * 1.5).round() as usize,
                // MAX_ACTION_STATE * MAX_ACTION_ANIMATION,
            ),
            model_transforms,

            event_cursor_id: 0,
            derive_keeping: None,
            new_velocity: Vec3A::ZERO,
            new_direction: DEFAULT_TOWARD_DIR_2D,
            cache_states: Vec::with_capacity(16),
        })
    }

    pub fn preload_assets(&self, ctx: &mut ContextUpdate<'_>, inst_player: Rc<InstPlayer>) -> XResult<Vec<AnimationFileMeta>> {
        let mut animations = Vec::with_capacity(16);
        let mut animation_files = DtHashMap::default();

        for action in inst_player.actions.values() {
            action.animations(&mut animations);
            for anime in animations.iter() {
                ctx.asset.load_animation(anime.files)?;
                if anime.root_motion {
                    ctx.asset.load_root_motion(anime.files)?;
                }
                if anime.weapon_motion {
                    ctx.asset.load_weapon_motion(anime.files)?;
                }
                match animation_files.entry(anime.files) {
                    Entry::Vacant(e) => { e.insert(anime.file_meta()); }
                    Entry::Occupied(mut e) => {
                        e.get_mut().root_motion |= anime.root_motion;
                        e.get_mut().weapon_motion |= anime.weapon_motion;
                    }
                }
            }
            animations.clear();
        }
        Ok(animation_files.into_values().collect())
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

        let next_act = self.handle_inputs(ctx, chara_physics)?;
        self.update_states(ctx, chara_physics, initing, next_act)?;
        self.apply_animations(ctx)?;
        Ok(())
    }

    fn handle_inputs(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        chara_physics: &LogicCharaPhysics,
    ) -> XResult<Option<Rc<dyn InstActionAny>>> {
        let current_act = match self.action_queue.last() {
            Some(act) => act,
            None => return Ok(Some(self.inst_idle_action.clone())),
        };

        let dk_end = self.derive_keeping.map(|dk| dk.end_time > ctx.time).unwrap_or(false);
        if dk_end {
            self.derive_keeping = None;
        }

        let frame = ctx.frame;
        let player_dir = chara_physics.direction();
        let events = ctx.input.player_events(self.player_id)?;
        let events = events.borrow_mut();
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
                }
                else {
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
                }
                else {
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

        if next_act.is_some() {
            self.event_cursor_id = events.future_id(); // Currently, clear preinput after matching a new action
        }
        Ok(next_act)
    }

    fn find_next_action(
        &self,
        current_act: &Box<dyn LogicActionAny>,
        player_dir: Vec2xz,
        key: VirtualKey,
        move_dir: Vec2xz,
    ) -> Option<Rc<dyn InstActionAny>> {
        if let Some(derive_keeping) = self.derive_keeping {
            debug_assert!(current_act.inst.derive_keeping || current_act.tmpl_id() == derive_keeping.action_id);
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
            if inst_act.tmpl_id == current_act.tmpl_id() {
                continue;
            }
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
        player_dir: Vec2xz,
        move_dir: Vec2xz,
    ) -> bool {
        // Check derive level
        if new_inst_act.enter_level <= cur_derive_level {
            return false;
        }

        // Check enter direction (move combination key)
        let Some(enter_key) = new_inst_act.enter_key
        else {
            return false;
        };
        if let Some(dir) = enter_key.dir {
            let in_range = match dir {
                VirtualDir::Forward(cos) => player_dir.dot(move_dir) > cos,
                VirtualDir::Backward(cos) => (-player_dir).dot(move_dir) > cos,
                VirtualDir::Left(cos) => Vec2xz::new(-player_dir.z, player_dir.x).dot(move_dir) > cos,
                VirtualDir::Right(cos) => Vec2xz::new(player_dir.z, -player_dir.x).dot(move_dir) > cos,
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
        mut next_act: Option<Rc<dyn InstActionAny>>,
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

        // Update current action
        if let Some(current_act) = self.action_queue.last_mut() {
            let mut ctxa = ContextAction::new(self.player_id, chara_physics, variables);
            let ret = current_act.update(ctx, &mut ctxa)?;
            if let Some(new_velocity) = ret.new_velocity {
                self.new_velocity = new_velocity;
            }
            if let Some(new_direction) = ret.new_direction {
                self.new_direction = new_direction;
            }

            if current_act.is_stopping() && next_act.is_none() {
                // Trigger derive keeping, when current action actively stops.
                self.derive_keeping = ret.derive_keeping;

                // Enter idle action, when current action stops without next action.
                next_act = Some(self.inst_idle_action.clone());
            }
        }
        // No current & next action, enter idle as fallback.
        else if next_act.is_none() {
            next_act = Some(self.inst_idle_action.clone());
        }

        // Update previous fade action
        for act in self.action_queue.iter_mut().rev().take_while(|act| act.is_fading()) {
            let mut ctxa = ContextAction::new(self.player_id, chara_physics, variables);
            act.fade_update(ctx, &mut ctxa)?;
        }

        // Handle next action
        if let Some(next_act) = next_act {
            self.action_queue.enqueue_with(
                ctx,
                |ctx, logic_act| try_reuse_logic_action(logic_act, ctx, next_act.clone()),
                |ctx| new_logic_action(ctx, next_act.clone()),
            )?;

            let (prev_act, current_act) = self.action_queue.last2_mut();
            let current_act = current_act.unwrap();
            let prev_act = prev_act.map(|act| act.as_mut());

            // Start current action
            let ret = {
                let mut ctxa = ContextAction::new(self.player_id, chara_physics, variables);
                if prev_act.is_some() {
                    ctxa.prev_action = prev_act.as_deref();
                }
                current_act.start(ctx, &mut ctxa)?
            };

            // Clear derive keeping, if current action not supported.
            if !current_act.inst.derive_keeping {
                self.derive_keeping = None;
            }

            // Handle previous action
            let mut ctxa = ContextAction::new(self.player_id, chara_physics, variables);
            if let Some(prev_act) = prev_act {
                let prev_fade_update =
                    ret.prev_fade_update && prev_act.is_activing() && prev_act.fade_start(ctx, &mut ctxa)?;
                println!(
                    "prev_fade_update: {} {} {}",
                    prev_fade_update,
                    ret.prev_fade_update,
                    prev_act.is_activing()
                );

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
                        act.stop(ctx, &mut ctxa)?;
                    }
                }
            }
        }

        // Handle previous actions
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

        // Finalize actions
        let mut ctxa = ContextAction::new(self.player_id, chara_physics, variables);
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

    fn apply_animations(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<()> {
        self.animator.discard(ctx.synced_frame);
        self.animator
            .update(ctx.frame, &self.cache_states, |res| ctx.asset.load_animation(*res))?;
        self.animator.animate()?;
        self.animator.model_out_transforms(&mut self.model_transforms)
    }

    pub fn restore(&mut self, ctx: &ContextRestore, states: &[Box<dyn StateActionAny>]) -> XResult<()> {
        let mut state_iter = states.iter();
        self.action_queue.restore_when(|act| {
            if act.last_frame < ctx.frame {
                Ok(-1)
            }
            else if act.first_frame > ctx.frame {
                return Ok(1);
            }
            else if let Some(state) = state_iter.next() {
                act.restore(state.as_ref())?;
                return Ok(0);
            }
            else {
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
        &self.model_transforms
    }

    #[inline]
    pub(crate) fn new_velocity(&self) -> Vec3A {
        self.new_velocity
    }

    #[inline]
    pub(crate) fn new_direction(&self) -> Vec2xz {
        self.new_direction
    }
}
