use approx::abs_diff_ne;
use critical_point_csgen::CsOut;
use glam::Vec3A;
use glam_ext::{Transform3A, Vec2xz};
use std::collections::hash_map::Entry;
use std::mem;
use std::rc::Rc;

use crate::animation::{AnimationFileMeta, Animator, HitMotionSampler};
use crate::consts::{DEFAULT_TOWARD_DIR_2D, MAX_ACTION_ANIMATION};
use crate::instance::{InstActionAny, InstActionIdle, InstCharacter};
use crate::logic::action::{
    new_logic_action, try_reuse_logic_action, ActionStartArgs, ContextAction, DeriveKeeping, LogicActionAny,
    StateActionAny,
};
use crate::logic::character::hit::LogicCharaHit;
use crate::logic::character::physics::LogicCharaPhysics;
use crate::logic::character::value::LogicCharaValue;
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::logic::system::input::InputVariables;
use crate::utils::{xerr, xres, CustomEvent, DtHashMap, HistoryQueue, InputDir, NumID, VirtualKey, XResult};

const DEFAULT_ACTION_QUEUE_CAP: usize = 8;

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
pub struct StateCharaAction {
    pub event_cursor_id: u64,
    pub derive_keeping: DeriveKeeping,
    pub action_changed: bool,
    pub animation_changed: bool,
}

#[derive(Debug)]
pub(crate) struct LogicCharaAction {
    chara_id: NumID,
    inst_chara: Rc<InstCharacter>,
    inst_idle_action: Rc<InstActionIdle>,
    action_queue: HistoryQueue<Box<dyn LogicActionAny>>,
    animator: Animator,

    event_cursor_id: u64,
    derive_keeping: DeriveKeeping,
    action_changed: bool,
    animation_changed: bool,

    new_velocity: Vec3A,
    new_direction: Vec2xz,
    cache_states: Vec<Box<dyn StateActionAny>>,
    action_events: Vec<CustomEvent>,
}

impl LogicCharaAction {
    pub(crate) fn new(
        ctx: &mut ContextUpdate,
        chara_id: NumID,
        inst_chara: Rc<InstCharacter>,
    ) -> XResult<LogicCharaAction> {
        let skeleton = ctx.asset.load_skeleton(inst_chara.skeleton_files)?;

        let inst_idle_action: Rc<InstActionIdle> = inst_chara
            .find_first_primary_action(&VirtualKey::Idle)
            .ok_or_else(|| xerr!(NotFound; "No idle action"))?;

        Ok(LogicCharaAction {
            chara_id,
            inst_chara,
            inst_idle_action,
            action_queue: HistoryQueue::with_capacity(DEFAULT_ACTION_QUEUE_CAP),
            animator: Animator::new(skeleton, DEFAULT_ACTION_QUEUE_CAP, MAX_ACTION_ANIMATION * 3)?,

            event_cursor_id: 0,
            derive_keeping: DeriveKeeping::default(),
            action_changed: false,
            animation_changed: false,

            new_velocity: Vec3A::ZERO,
            new_direction: DEFAULT_TOWARD_DIR_2D,
            cache_states: Vec::with_capacity(16),
            action_events: Vec::new(),
        })
    }

    #[inline]
    pub fn preload_assets(&self, ctx: &mut ContextUpdate) -> XResult<Vec<AnimationFileMeta>> {
        let mut animations = Vec::with_capacity(16);
        let mut animation_files = DtHashMap::default();

        for action in self.inst_chara.actions.values() {
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
                    Entry::Vacant(e) => {
                        e.insert(anime.file_meta());
                    }
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

    #[inline]
    pub(crate) fn init(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_physics: &LogicCharaPhysics,
        chara_value: &LogicCharaValue,
    ) -> XResult<()> {
        self.update_actions(ctx, chara_physics, chara_value, false, None)?;
        self.apply_animations(ctx)?;
        Ok(())
    }

    #[inline]
    pub(crate) fn update(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_physics: &LogicCharaPhysics,
        chara_value: &LogicCharaValue,
        chara_hit: &LogicCharaHit,
    ) -> XResult<()> {
        if self.action_queue.is_empty() {
            return xres!(Unexpected; "action queue empty");
        }

        let next_act = match self.inst_chara.is_player {
            true => self.handle_inputs(ctx, chara_physics)?,
            false => self.handle_virtual_input(ctx, chara_physics, chara_hit)?,
        };

        self.update_actions(ctx, chara_physics, chara_value, self.inst_chara.is_player, next_act)?;
        self.apply_animations(ctx)?;
        Ok(())
    }

    fn handle_inputs(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_physics: &LogicCharaPhysics,
    ) -> XResult<Option<NextAction>> {
        let current_act = match self.action_queue.last() {
            Some(act) => act,
            None => return Ok(Some(NextAction::new_idle(self.inst_idle_action.clone()))),
        };

        if self.derive_keeping.is_valid() && self.derive_keeping.end_time > ctx.time {
            self.derive_keeping.clear();
        }

        let frame = ctx.frame;
        let player_dir = chara_physics.direction();
        let events = ctx.input.player_events(self.chara_id)?;
        let events = events.borrow_mut();
        let mut next_act: Option<NextAction> = None;

        // Handle preinput events
        for event in events.iter_preinput(frame, self.event_cursor_id)? {
            if event.pressed {
                continue;
            }
            next_act = self.find_next_action(
                current_act,
                next_act,
                player_dir,
                event.key.into(),
                event.world_move_dir,
            );
        }

        // Handle current frame events
        for event in events.iter_current(frame)? {
            if event.pressed {
                continue;
            }
            next_act = self.find_next_action(
                current_act,
                next_act,
                player_dir,
                event.key.into(),
                event.world_move_dir,
            );
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

    fn handle_virtual_input(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_physics: &LogicCharaPhysics,
        chara_hit: &LogicCharaHit,
    ) -> XResult<Option<NextAction>> {
        let current_act = match self.action_queue.last() {
            Some(act) => act,
            None => return Ok(Some(NextAction::new_idle(self.inst_idle_action.clone()))),
        };
        let player_dir = chara_physics.direction();

        let mut next_act: Option<NextAction> = None;

        for ev_idx in chara_hit.be_hit_events().iter().cloned() {
            let event = &ctx.hit_events[ev_idx];
            let mut dir = Vec2xz::new(event.character_vector.x, event.character_vector.z);
            dir = match abs_diff_ne!(dir, Vec2xz::ZERO) {
                true => dir.normalize(),
                false => -DEFAULT_TOWARD_DIR_2D,
            };
            next_act = self.find_next_action(current_act, next_act, player_dir, VirtualKey::Hit1, dir);
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
            Some(NextAction::new(new_inst_act, key, dir))
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

        let derive_level = match current_act.is_activing() {
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

    fn update_actions(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_physics: &LogicCharaPhysics,
        chara_value: &LogicCharaValue,
        has_input_events: bool,
        mut next_act: Option<NextAction>,
    ) -> XResult<()> {
        // Clear temporary values
        self.new_velocity = Vec3A::ZERO;
        self.new_direction = chara_physics.direction();
        self.cache_states.clear();
        self.cache_states.reserve(self.action_queue.len() + 1);
        self.action_events = Vec::new();

        let mut input_vars = InputVariables::default();
        let mut input_future_id = 0;
        if has_input_events {
            let events = ctx.input.player_events(self.chara_id)?;
            input_vars = events.borrow().variables(ctx.frame)?;
            input_future_id = events.borrow().future_id();
        }

        let time_speed = match chara_value.hit_lag_time().contains(ctx.time) {
            true => 0.0,
            false => 1.0,
        };

        // Update current action
        if let Some(current_act) = self.action_queue.last_mut() {
            let mut ctxa = ContextAction::new_normalized(self.chara_id, chara_physics, input_vars, time_speed);
            let ret = current_act.update(ctx, &mut ctxa)?;
            if let Some(new_velocity) = ret.new_velocity {
                self.new_velocity = new_velocity;
            }
            if let Some(new_direction) = ret.new_direction {
                self.new_direction = new_direction;
            }

            if ret.clear_preinput {
                self.event_cursor_id = input_future_id;
            }

            self.action_events = ret.custom_events;

            if current_act.is_stopping() && next_act.is_none() {
                // Trigger derive keeping, when current action actively stops.
                self.derive_keeping = ret.derive_keeping;

                // Enter idle action, when current action stops without next action.
                next_act = Some(NextAction::new_idle(self.inst_idle_action.clone()));
            }
        }
        // No current & next action, enter idle as fallback.
        else if next_act.is_none() {
            next_act = Some(NextAction::new_idle(self.inst_idle_action.clone()));
        }

        // Update previous fade action
        for act in self.action_queue.iter_mut().rev().take_while(|act| act.is_fading()) {
            let mut ctxa = ContextAction::new_normalized(self.chara_id, chara_physics, input_vars, time_speed);
            act.fade_update(ctx, &mut ctxa)?;
        }

        // Handle next action
        if let Some(next_act) = next_act {
            self.action_queue.enqueue_with(
                ctx,
                |ctx, logic_act| try_reuse_logic_action(logic_act, ctx, next_act.action.clone()),
                |ctx| new_logic_action(ctx, next_act.action.clone()),
            )?;

            let (prev_act, current_act) = self.action_queue.last2_mut();
            let prev_act = prev_act.map(|act| act.as_mut());
            let current_act = current_act.unwrap();

            // Start current action
            let ret = {
                let mut ctxa = ContextAction::new_normalized(self.chara_id, chara_physics, input_vars, time_speed);
                let args = ActionStartArgs::new(prev_act.as_deref(), next_act.key, next_act.dir);
                current_act.start(ctx, &mut ctxa, &args)?
            };

            if ret.clear_preinput {
                self.event_cursor_id = input_future_id;
            }

            self.action_events.extend(ret.custom_events);

            // Clear derive keeping, if current action not supported.
            if !current_act.inst.derive_keeping {
                self.derive_keeping.clear();
            }

            // Handle previous action
            let mut ctxa = ContextAction::new_normalized(self.chara_id, chara_physics, input_vars, time_speed);
            if let Some(prev_act) = prev_act {
                let prev_fade_update =
                    ret.prev_fade_update && prev_act.is_activing() && prev_act.fade_start(ctx, &mut ctxa)?;
                // println!(
                //     "prev_fade_update: {} {} {}",
                //     prev_fade_update,
                //     ret.prev_fade_update,
                //     prev_act.is_activing()
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
        let mut ctxa = ContextAction::new_normalized(self.chara_id, chara_physics, input_vars, time_speed);
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

    fn apply_animations(&mut self, ctx: &mut ContextUpdate) -> XResult<()> {
        let prev_ids = self.animator.action_animation_id();

        self.animator.discard(ctx.synced_frame);
        self.animator.update(ctx.frame, &self.cache_states, &mut ctx.asset)?;
        self.animator.animate()?;

        let current_ids = self.animator.action_animation_id();
        self.action_changed = prev_ids.0 != current_ids.0;
        self.animation_changed = prev_ids != current_ids;
        Ok(())
    }

    pub fn restore(
        &mut self,
        ctx: &ContextRestore,
        state: &StateCharaAction,
        states: &[Box<dyn StateActionAny>],
    ) -> XResult<()> {
        self.event_cursor_id = state.event_cursor_id;
        self.derive_keeping = state.derive_keeping;
        self.action_changed = state.action_changed;
        self.animation_changed = state.animation_changed;

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

    pub fn take_states(&mut self) -> XResult<(StateCharaAction, Vec<Box<dyn StateActionAny>>, Vec<CustomEvent>)> {
        if self.cache_states.is_empty() {
            return xres!(LogicBadState; "states already taken");
        }
        Ok((
            StateCharaAction {
                event_cursor_id: self.event_cursor_id,
                derive_keeping: self.derive_keeping,
                action_changed: self.action_changed,
                animation_changed: self.animation_changed,
            },
            mem::take(&mut self.cache_states),
            mem::take(&mut self.action_events),
        ))
    }

    #[inline]
    pub(crate) fn model_transforms(&self) -> &[Transform3A] {
        self.animator.model_transforms()
    }

    #[inline]
    pub(crate) fn new_velocity(&self) -> Vec3A {
        self.new_velocity
    }

    #[inline]
    pub(crate) fn new_direction(&self) -> Vec2xz {
        self.new_direction
    }

    #[inline]
    pub(crate) fn action_changed(&self) -> bool {
        self.action_changed
    }

    #[inline]
    pub(crate) fn animation_changed(&self) -> bool {
        self.animation_changed
    }

    #[inline]
    pub(crate) fn current_action(&self) -> Option<&dyn LogicActionAny> {
        self.action_queue.last().map(|act| act.as_ref())
    }

    pub(crate) fn current_action_with_log(&self) -> Option<&dyn LogicActionAny> {
        match self.current_action() {
            Some(act) => Some(act),
            None => {
                log::warn!(
                    "character_id={}, character={}, style={}, current action is none",
                    self.chara_id,
                    self.inst_chara.tmpl_character,
                    self.inst_chara.tmpl_style,
                );
                None
            }
        }
    }

    #[inline]
    pub(crate) fn hit_motion_sampler(&self) -> Option<&HitMotionSampler> {
        self.animator.hit_motion_sampler()
    }

    pub(crate) fn hit_motion_sampler_with_log(&self) -> Option<&HitMotionSampler> {
        match self.hit_motion_sampler() {
            Some(sampler) => Some(sampler),
            None => {
                log::warn!(
                    "character_id={}, character={}, style={}, action={:?}, HitMotionSampler is none",
                    self.chara_id,
                    self.inst_chara.tmpl_character,
                    self.inst_chara.tmpl_style,
                    self.current_action().map(|act| act.tmpl_id())
                );
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
struct NextAction {
    action: Rc<dyn InstActionAny>,
    key: VirtualKey,
    dir: Option<Vec2xz>,
}

impl NextAction {
    #[inline]
    fn new(action: Rc<dyn InstActionAny>, key: VirtualKey, dir: Vec2xz) -> NextAction {
        NextAction {
            action,
            key,
            dir: Some(dir),
        }
    }

    #[inline]
    fn new_idle(action: Rc<dyn InstActionAny>) -> NextAction {
        NextAction {
            action,
            key: VirtualKey::Idle,
            dir: None,
        }
    }
}
