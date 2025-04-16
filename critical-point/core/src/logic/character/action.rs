use glam::{Quat, Vec2, Vec3A};
use glam_ext::Transform3A;
use std::mem;
use std::rc::Rc;

use super::physics::LogicCharaPhysics;
use crate::animation::SkeletalAnimator;
use crate::consts::{MAX_ACTION_ANIMATION, WEIGHT_THRESHOLD};
use crate::instance::{InstAction, InstActionIdle, InstPlayer};
use crate::logic::action::{
    new_logic_action, try_reuse_logic_action, ContextAction, LogicAction, LogicActionIdle, StateAction,
};
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::logic::system::input::InputVariables;
use crate::template::TmplCharacter;
use crate::utils::{
    force_mut, xerr, xres, ASymbol, DtHashSet, HistoryQueue, NumID, Symbol, VirtualDirection, VirtualKey, XResult,
};

#[derive(Debug)]
pub(crate) struct LogicCharaAction {
    player_id: NumID,
    inst_player: Rc<InstPlayer>,
    action_queue: HistoryQueue<Box<dyn LogicAction>>,
    skeleton_animator: SkeletalAnimator,

    new_velocity: Vec3A,
    new_rotation: Quat,
    cache_states: Vec<Box<dyn StateAction>>,
}

impl LogicCharaAction {
    #[cfg(test)]
    pub(crate) fn mock(player_id: NumID, inst_player: Rc<InstPlayer>) -> LogicCharaAction {
        let skeleton = Rc::new(ozz_animation_rs::Skeleton::from_path("./test-asset/girl_skeleton_logic.ozz").unwrap());
        LogicCharaAction {
            player_id,
            inst_player,
            action_queue: HistoryQueue::with_capacity(4),
            skeleton_animator: SkeletalAnimator::new(skeleton.clone(), 0, 0, 0),

            new_velocity: Vec3A::ZERO,
            new_rotation: Quat::IDENTITY,
            cache_states: Vec::with_capacity(16),
        }
    }

    pub fn new(
        ctx: &mut ContextUpdate<'_>,
        player_id: NumID,
        inst_player: Rc<InstPlayer>,
    ) -> XResult<LogicCharaAction> {
        let tmpl_chara = ctx.tmpl_db.find_as::<TmplCharacter>(&inst_player.tmpl_character)?;
        let skeleton = ctx.asset.load_skeleton(&tmpl_chara.skeleton)?;

        let inst_idle = inst_player
            .find_first_primary_action(&VirtualKey::Idle)
            .ok_or_else(|| xerr!(NotFound; "idle key"))?;

        let mut chara_action: LogicCharaAction = LogicCharaAction {
            player_id,
            inst_player,
            action_queue: HistoryQueue::with_capacity(4),
            skeleton_animator: SkeletalAnimator::new(
                skeleton.clone(),
                SkeletalAnimator::OUT_MODEL_TRANSFORM,
                6,
                4 * MAX_ACTION_ANIMATION,
            ),

            new_velocity: Vec3A::ZERO,
            new_rotation: Quat::IDENTITY,
            cache_states: Vec::with_capacity(16),
        };

        chara_action
            .action_queue
            .enqueue_new(Box::new(LogicActionIdle::new(ctx, inst_idle)?));
        Ok(chara_action)
    }

    pub fn preload_assets(&self, ctx: &mut ContextUpdate<'_>, inst_player: Rc<InstPlayer>) -> XResult<Vec<ASymbol>> {
        let mut animations = Vec::with_capacity(16);
        let mut animation_files = DtHashSet::default();
        for action in inst_player.actions.values() {
            animations.clear();
            action.animations(&mut animations);
            animation_files.extend(animations.iter().map(|a| a.file.clone()));
        }
        for anime in animation_files.iter() {
            ctx.asset.load_animation(anime)?;
        }
        Ok(animation_files.into_iter().map(|a| ASymbol::from(&a)).collect())
    }

    pub fn update(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        chara_physics: &LogicCharaPhysics,
        is_initing: bool,
    ) -> XResult<()> {
        self.handle_inputs(ctx, chara_physics, is_initing)?;
        self.update_states(ctx, chara_physics, is_initing)?;
        self.apply_animations(ctx)?;
        Ok(())
    }

    fn handle_inputs(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        chara_physics: &LogicCharaPhysics,
        is_initing: bool,
    ) -> XResult<()> {
        if is_initing {
            return Ok(());
        }

        assert!(!self.action_queue.is_empty());

        let frame = ctx.frame;
        let player_direction = chara_physics.direction_2d();
        let events = ctx.input.player_events(self.player_id)?;
        let events = events.borrow_mut();

        let mut enter_any_action = false;

        {
            // Handle preinput events
            let mut next = None;
            for event in events.iter_preinput(frame, 0)? {
                let current_action = self.action_queue.last().unwrap();
                if let Some(candidate) =
                    self.find_next_action(current_action, player_direction, event.key.into(), event.world_move_dir)
                {
                    match &mut next {
                        next @ None => *next = Some(candidate),
                        Some(next) if candidate.enter_level >= next.enter_level => *next = candidate,
                        _ => (),
                    }
                }
            }

            if let Some(next) = next {
                // events.consume_preinput(frame)?;
                self.action_queue.enqueue_with(
                    ctx,
                    |ctx, logic_act| try_reuse_logic_action(logic_act, ctx, next.clone()),
                    |ctx| new_logic_action(ctx, next.clone()),
                )?;
                enter_any_action = true;
            }
        }

        {
            // Handle current frame events
            let mut consume_id = None;
            for event in events.iter_current(frame)? {
                let current_action = &self.action_queue.last().unwrap();
                if let Some(next) =
                    self.find_next_action(current_action, player_direction, event.key.into(), event.world_move_dir)
                {
                    self.action_queue.enqueue_with(
                        ctx,
                        |ctx, logic_act| try_reuse_logic_action(logic_act, ctx, next.clone()),
                        |ctx| new_logic_action(ctx, next.clone()),
                    )?;
                    enter_any_action = true;
                    consume_id = Some(event.id);
                }
            }

            if let Some(event_idx) = consume_id {
                // events.consume(event_idx)?;
            }
        }

        // No next action found, try Walk/Run.
        if !enter_any_action {
            let mov = events.variables(frame)?.optimized_world_move();
            if let Some(mov_dir) = mov.move_dir() {
                let current_action = &self.action_queue.last().unwrap();
                if let Some(next) = self.find_next_action(
                    current_action,
                    player_direction,
                    if mov.slow { VirtualKey::Walk } else { VirtualKey::Run },
                    mov_dir,
                ) {
                    self.action_queue.enqueue_with(
                        ctx,
                        |ctx, logic_act| try_reuse_logic_action(logic_act, ctx, next.clone()),
                        |ctx| new_logic_action(ctx, next.clone()),
                    )?;
                    // events.consume_frame(frame)?;
                }
            }
        }

        // Current action ended, enter Idle.
        if self.action_queue.last().unwrap().is_leaving {
            let idle: Rc<InstActionIdle> = self
                .inst_player
                .find_first_primary_action(&VirtualKey::Idle)
                .ok_or_else(|| xerr!(NotFound; "idle key"))?;
            self.action_queue.enqueue_with(
                ctx,
                |ctx, logic_act| try_reuse_logic_action(logic_act, ctx, idle.clone()),
                |ctx| new_logic_action(ctx, idle.clone()),
            )?;
        }

        Ok(())
    }

    fn find_next_action(
        &self,
        current_action: &Box<dyn LogicAction>,
        player_direction: Vec2,
        key: VirtualKey,
        move_dir: Vec2,
    ) -> Option<Rc<dyn InstAction>> {
        for inst_action in self.inst_player.filter_actions(&(current_action.tmpl_id.clone(), key)) {
            // Check derive level
            if inst_action.enter_level <= current_action.derive_level {
                continue;
            }

            // Check enter direction (move combination key)
            if let Some(enter_dir) = inst_action.enter_direction {
                let in_range = match enter_dir {
                    VirtualDirection::Forward(cos) => player_direction.dot(move_dir) > cos,
                    VirtualDirection::Backward(cos) => (-player_direction).dot(move_dir) > cos,
                    VirtualDirection::Left(cos) => {
                        Vec2::new(-player_direction.y, player_direction.x).dot(move_dir) > cos
                    }
                    VirtualDirection::Right(cos) => {
                        Vec2::new(player_direction.y, -player_direction.x).dot(move_dir) > cos
                    }
                };
                if !in_range {
                    continue;
                }
            }

            // TODO: Check custom script

            return Some(inst_action);
        }
        None
    }

    fn update_states(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        chara_physics: &LogicCharaPhysics,
        is_initing: bool,
    ) -> XResult<()> {
        self.new_velocity = Vec3A::ZERO;
        self.new_rotation = chara_physics.rotation();
        self.cache_states.clear();
        self.cache_states.reserve(self.action_queue.len() + 1);

        let variables = match is_initing {
            true => InputVariables::default(),
            false => ctx.input.player_events(self.player_id)?.borrow().variables(ctx.frame)?,
        };
        let mut ctxa = ContextAction::new(self.player_id, chara_physics, variables);
        ctxa.new_velocity = self.new_velocity;
        ctxa.new_rotation = self.new_rotation;
        ctxa.unused_weight = 1.0;

        let queue_len = self.action_queue.len();
        for idx in (0..queue_len).rev() {
            let action = unsafe { force_mut(&self.action_queue[idx]) }; // It's safe to force_mut here
            ctxa.next_action = self.action_queue.get(idx + 1).map(|a| a.as_ref());
            ctxa.prev_action = idx
                .checked_sub(1)
                .and_then(|i| self.action_queue.get(i))
                .map(|a| a.as_ref());
            if let Some(state) = action.update(ctx, &mut ctxa)? {
                self.cache_states.push(state);
            }
            if idx == queue_len - 1 {
                self.new_velocity = ctxa.new_velocity;
                self.new_rotation = ctxa.new_rotation;
            }
            // if ctxa.unused_weight < WEIGHT_THRESHOLD {
            //     break;
            // }
        }
        self.cache_states.reverse();

        self.action_queue.dequeue(|act| act.death_frame <= ctx.frame);
        self.action_queue.discard(|act| act.death_frame <= ctx.synced_frame);

        // for act in self.cache_states.iter() {
        //     println!("{:?}", act);
        // }
        Ok(())
    }

    fn apply_animations(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<()> {
        // println!("{:?}", self.cache_states);
        self.skeleton_animator.update(ctx.frame, &self.cache_states, |res| {
            ctx.asset.load_animation(&Symbol::from(&res))
        })?;
        self.skeleton_animator.animate()?;
        Ok(())
    }

    pub fn restore(&mut self, ctx: &ContextRestore, states: &[Box<dyn StateAction>]) -> XResult<()> {
        let mut state_iter = states.iter();
        self.action_queue.restore_when(|act| {
            if act.death_frame < ctx.frame {
                Ok(-1)
            } else if act.spawn_frame > ctx.frame {
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
        self.new_rotation = Quat::IDENTITY;
        Ok(())
    }

    pub fn states(&self) -> XResult<&[Box<dyn StateAction>]> {
        if self.cache_states.is_empty() {
            return xres!(LogicBadState; "states already taken");
        }
        Ok(&self.cache_states)
    }

    pub fn take_states(&mut self) -> XResult<Vec<Box<dyn StateAction>>> {
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
    pub(crate) fn new_rotation(&self) -> Quat {
        self.new_rotation
    }
}
