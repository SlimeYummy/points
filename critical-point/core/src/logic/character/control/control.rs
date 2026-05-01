use critical_point_csgen::CsOut;
use glam::Vec3A;
use glam_ext::{Transform3A, Vec2xz};
use std::collections::hash_map::Entry;
use std::mem;
use std::rc::Rc;

use crate::animation::{AnimationFileMeta, Animator, HitMotionSampler};
use crate::consts::{DEFAULT_TOWARD_DIR_2D, MAX_ACTION_ANIMATION};
use crate::instance::{InstActionAny, InstActionIdle, InstAiBrain, InstCharacter};
use crate::logic::action::{DeriveKeeping, LogicActionAny, StateActionAny};
use crate::logic::ai_task::{AiTaskReturn, LogicAiTaskAny};
use crate::logic::character::physics::LogicCharaPhysics;
use crate::logic::character::value::LogicCharaValue;
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::utils::{CustomEvent, DtHashMap, HistoryQueue, NumID, VirtualKey, XResult, xerr, xres};

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
pub struct StateCharaControl {
    pub event_cursor_id: u64,
    pub derive_keeping: DeriveKeeping,
    pub action_changed: bool,
    pub animation_changed: bool,
}

#[derive(Debug)]
pub(crate) struct LogicCharaControl {
    pub(super) chara_id: NumID,
    pub(super) inst_chara: Rc<InstCharacter>,
    pub(super) inst_idle_action: Rc<InstActionIdle>,
    pub(super) inst_ai_brain: Option<Rc<InstAiBrain>>,

    pub(super) action_queue: HistoryQueue<Box<dyn LogicActionAny>>,
    pub(super) current_task: Option<Box<dyn LogicAiTaskAny>>,
    pub(super) event_cursor_id: u64,
    pub(super) derive_keeping: DeriveKeeping,
    pub(super) action_changed: bool,
    pub(super) animation_changed: bool,

    pub(super) new_velocity: Vec3A,
    pub(super) new_direction: Vec2xz,
    pub(super) cache_states: Vec<Box<dyn StateActionAny>>,
    pub(super) action_events: Vec<CustomEvent>,

    pub(super) animator: Animator,
}

impl LogicCharaControl {
    pub(crate) fn new(
        ctx: &mut ContextUpdate,
        chara_id: NumID,
        inst_chara: Rc<InstCharacter>,
        inst_ai_brain: Option<Rc<InstAiBrain>>,
    ) -> XResult<LogicCharaControl> {
        let skeleton = ctx.asset.load_skeleton(inst_chara.skeleton_files)?;

        let inst_idle_action: Rc<InstActionIdle> = inst_chara
            .find_first_primary_action(&VirtualKey::Idle)
            .ok_or_else(|| xerr!(NotFound; "No idle action"))?;

        Ok(LogicCharaControl {
            chara_id,
            inst_chara,
            inst_ai_brain,
            inst_idle_action,

            action_queue: HistoryQueue::with_capacity(DEFAULT_ACTION_QUEUE_CAP),
            current_task: None,
            event_cursor_id: 0,
            derive_keeping: DeriveKeeping::default(),
            action_changed: false,
            animation_changed: false,

            new_velocity: Vec3A::ZERO,
            new_direction: DEFAULT_TOWARD_DIR_2D,
            cache_states: Vec::with_capacity(16),
            action_events: Vec::new(),

            animator: Animator::new(skeleton, DEFAULT_ACTION_QUEUE_CAP, MAX_ACTION_ANIMATION * 3)?,
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
        chara_phy: &LogicCharaPhysics,
        chara_val: &LogicCharaValue,
    ) -> XResult<()> {
        let mut ctxa;
        let mut next_action = None;
        if self.inst_chara.is_player {
            ctxa = self.make_ctxa_default(ctx, chara_phy, chara_val)?;
        }
        else {
            let mut ai_ret = self.handle_ai_all(ctx, chara_phy, chara_val)?;
            ai_ret.quick_switch = false;
            ctxa = self.make_ctxa_from_ai_return(ctx, chara_phy, chara_val, &ai_ret);
            next_action = NextAction::try_from_ai_return(&ai_ret);
        }
        self.handle_next_action(ctx, &mut ctxa, next_action)?;

        self.collect_states_and_cleanup(ctx, chara_phy, None)?;
        Ok(())
    }

    #[inline]
    pub(crate) fn update(
        &mut self,
        ctx: &mut ContextUpdate,
        chara_phy: &LogicCharaPhysics,
        chara_val: &LogicCharaValue,
    ) -> XResult<()> {
        if self.action_queue.is_empty() {
            return xres!(Unexpected; "action queue empty");
        }

        let mut next_action = self.handle_hit_events(ctx, chara_phy)?;

        let mut ctxa;
        if self.inst_chara.is_player {
            next_action = self.handle_player_inputs(ctx, chara_phy, next_action)?;
            ctxa = self.make_ctxa_from_inputs(ctx, chara_phy, chara_val)?;
        }
        else {
            let ai_ret = self.handle_ai_all(ctx, chara_phy, chara_val)?;
            ctxa = self.make_ctxa_from_ai_return(ctx, chara_phy, chara_val, &ai_ret);
            next_action = NextAction::try_from_ai_return(&ai_ret).or(next_action);
        }

        let previous_frame_state = self.handle_next_action(ctx, &mut ctxa, next_action)?;

        self.update_current_actions(ctx, &mut ctxa, chara_phy)?;

        self.collect_states_and_cleanup(ctx, chara_phy, previous_frame_state)?;
        Ok(())
    }

    pub fn restore(
        &mut self,
        ctx: &ContextRestore,
        state: &StateCharaControl,
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

    pub(crate) fn apply_animations(&mut self, ctx: &mut ContextUpdate) -> XResult<()> {
        let prev_ids = self.animator.action_animation_id();

        self.animator.discard(ctx.synced_frame);
        self.animator.update(ctx.frame, &self.cache_states, &mut ctx.asset)?;
        self.animator.animate()?;

        let current_ids = self.animator.action_animation_id();
        self.action_changed = prev_ids.0 != current_ids.0;
        self.animation_changed = prev_ids != current_ids;
        Ok(())
    }

    pub(crate) fn states(&self) -> XResult<&[Box<dyn StateActionAny>]> {
        if self.cache_states.is_empty() {
            return xres!(LogicBadState; "states already taken");
        }
        Ok(&self.cache_states)
    }

    pub(crate) fn take_states(&mut self) -> XResult<(StateCharaControl, Vec<Box<dyn StateActionAny>>, Vec<CustomEvent>)> {
        if self.cache_states.is_empty() {
            return xres!(LogicBadState; "states already taken");
        }
        Ok((
            StateCharaControl {
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
pub(super) struct NextAction {
    pub(super) action: Rc<dyn InstActionAny>,
    pub(super) dir: Option<Vec2xz>,
    pub(super) quick_switch: bool,
}

impl NextAction {
    #[inline]
    pub(super) fn new(action: Rc<dyn InstActionAny>, dir: Vec2xz, quick_switch: bool) -> NextAction {
        NextAction {
            action,
            dir: Some(dir),
            quick_switch,
        }
    }

    #[inline]
    pub(super) fn new_idle(action: Rc<dyn InstActionAny>) -> NextAction {
        NextAction {
            action,
            dir: None,
            quick_switch: false,
        }
    }

    #[inline]
    pub(super) fn try_from_ai_return(ai_ret: &AiTaskReturn) -> Option<NextAction> {
        match &ai_ret.next_action {
            Some(action) => Some(NextAction {
                action: action.clone(),
                dir: ai_ret.world_move.move_dir().map(|dir| dir.into()),
                quick_switch: ai_ret.quick_switch,
            }),
            None => None,
        }
    }
}
