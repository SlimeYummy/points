use std::rc::Rc;

use super::physics::LogicCharaPhysics;
use crate::animation::SkeletalAnimator;
use crate::consts::{MAX_ACTION_ANIMATION, WEIGHT_THRESHOLD};
use crate::instance::InstPlayer;
use crate::logic::action::{
    new_logic_action, try_reuse_logic_action, ContextActionNext, ContextActionUpdate, LogicAction, LogicActionIdle,
    StateAction,
};
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::template::TmplCharacter;
use crate::utils::{force_mut, DtHashSet, HistoryQueue, KeyCode, NumID, Symbol, XError, XResult};

#[derive(Debug)]
pub(crate) struct LogicCharaAction {
    player_id: NumID,
    inst_player: Rc<InstPlayer>,
    action_queue: HistoryQueue<Box<dyn LogicAction>>,
    skeleton_animator: SkeletalAnimator,
}

impl LogicCharaAction {
    #[cfg(test)]
    pub(crate) fn mock(player_id: NumID, inst_player: Rc<InstPlayer>) -> LogicCharaAction {
        let skeleton = Rc::new(ozz_animation_rs::Skeleton::from_path("./test-asset/girl_skeleton_logic.ozz").unwrap());
        LogicCharaAction {
            player_id,
            inst_player,
            action_queue: HistoryQueue::with_capacity(4),
            skeleton_animator: SkeletalAnimator::new(skeleton, false, 0, 0),
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
            .find_first_primary_action(&KeyCode::Idle)
            .ok_or(XError::not_found("LogicCharaAction::new() Idle"))?;

        let mut chara_action: LogicCharaAction = LogicCharaAction {
            player_id,
            inst_player,
            action_queue: HistoryQueue::with_capacity(4),
            skeleton_animator: SkeletalAnimator::new(skeleton, false, 6, 4 * MAX_ACTION_ANIMATION),
        };
        chara_action
            .action_queue
            .enqueue_new(Box::new(LogicActionIdle::new(ctx, inst_idle)?));
        Ok(chara_action)
    }

    pub fn preload_assets(&self, ctx: &mut ContextUpdate<'_>, inst_player: Rc<InstPlayer>) -> XResult<Vec<Symbol>> {
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
        Ok(animation_files.into_iter().collect())
    }

    pub fn init(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        chara_physics: &mut LogicCharaPhysics,
    ) -> XResult<Vec<Box<dyn StateAction>>> {
        let states = self.update_states(ctx, chara_physics)?;
        self.apply_animations(ctx, &states)?;
        Ok(states)
    }

    pub fn update(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        chara_physics: &mut LogicCharaPhysics,
    ) -> XResult<Vec<Box<dyn StateAction>>> {
        self.next_action(ctx)?;
        let states = self.update_states(ctx, chara_physics)?;
        self.apply_animations(ctx, &states)?;
        Ok(states)
    }

    fn next_action(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<()> {
        let mut inst_next = None;
        if let Some(action) = self.action_queue.get_mut(0) {
            let ctx_an = ContextActionNext::new(self.player_id, self.inst_player.clone());
            inst_next = action.next(ctx, &ctx_an)?;
        }

        if let Some(inst_next) = inst_next {
            self.action_queue.enqueue_with(
                ctx,
                |ctx, logic_act| try_reuse_logic_action(logic_act, ctx, inst_next.clone()),
                |ctx| new_logic_action(ctx, inst_next.clone()),
            )?;
        }
        Ok(())
    }

    fn update_states(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        chara_physics: &mut LogicCharaPhysics,
    ) -> XResult<Vec<Box<dyn StateAction>>> {
        let mut ctx_au = ContextActionUpdate::new(self.player_id, self.inst_player.clone(), chara_physics, 0);
        ctx_au.unused_weight = 1.0;
        ctx_au.states = Vec::with_capacity(self.action_queue.len() + 1);

        for idx in (0..self.action_queue.len()).rev() {
            let action = unsafe { force_mut(&self.action_queue[idx]) }; // It's safe to force_mut here
            ctx_au.next_action = self.action_queue.get(idx + 1).map(|a| a.as_ref());
            ctx_au.prev_action = idx
                .checked_sub(1)
                .and_then(|i| self.action_queue.get(i))
                .map(|a| a.as_ref());
            action.update(ctx, &mut ctx_au)?;
            if ctx_au.unused_weight < WEIGHT_THRESHOLD {
                break;
            }
        }
        ctx_au.states.reverse();
        let states = ctx_au.states;

        self.action_queue.dequeue(|act| act.death_frame <= ctx.frame);
        self.action_queue.discard(|act| act.death_frame <= ctx.synced_frame);
        Ok(states)
    }

    fn apply_animations(&mut self, ctx: &mut ContextUpdate<'_>, states: &[Box<dyn StateAction>]) -> XResult<()> {
        self.skeleton_animator
            .update(ctx.frame, states, |res| ctx.asset.load_animation(res))?;
        self.skeleton_animator.animate()
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
                return Err(XError::unexpected("LogicCharaAction::restore() action states order"));
            }
        })?;
        Ok(())
    }
}
