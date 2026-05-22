use critical_point_csgen::{CsEnum, CsOut};
use glam::Vec3Swizzles;
use glam_ext::Vec2xz;
use libm;
use std::f32::consts::PI;
use std::fmt::Debug;
use std::rc::Rc;

use crate::consts::MAX_ACTION_ANIMATION;
use crate::instance::{InstActionMoveNpc, InstAnimation};
use crate::logic::action::base::{
    ActionStartArgs, ActionStartReturn, ActionUpdateReturn, ContextAction, LogicActionAny, LogicActionBase,
    StateActionAnimation, StateActionAny, StateActionBase, impl_state_action,
};
use crate::logic::action::root_motion::{LogicMultiRootMotion, StateMultiRootMotion};
use crate::logic::game::ContextUpdate;
use crate::utils::{
    ActionType, Castable, LEVEL_MOVE, XResult, extend, ifelse, loose_ge, loose_le, s2ff_round, xres, xresf,
};

#[repr(u8)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsEnum,
)]
#[rkyv(derive(Debug))]
pub enum ActionMoveNpcMode {
    Start,
    Move,
    Stop,
}

#[repr(C)]
#[derive(
    Debug, PartialEq, rkyv::Archive, serde::Serialize, serde::Deserialize, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateActionMoveNpc {
    pub _base: StateActionBase,

    pub mode: ActionMoveNpcMode,
    pub current_time: f32,
    pub local_fade_in_weight: f32,

    pub stop_anim_idx: u16,
    pub prepare_stop_time: f32,

    pub root_motion: StateMultiRootMotion,
}

extend!(StateActionMoveNpc, StateActionBase);
impl_state_action!(StateActionMoveNpc, MoveNpc, "MoveNpc");

///
/// General move action logic.
/// Applicable to NPC movement controlled by programs or AI.
///
/// This action assumes that the phases of start, loop, turn and stop, during the movement process
/// will not be suddenly changed or interrupted by external inputs.
///
#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionMoveNpc {
    _base: LogicActionBase,
    inst: Rc<InstActionMoveNpc>,
    turn_angle_step: Vec2xz,
    turn_cos_step: f32,

    mode: ActionMoveNpcMode,
    current_time: f32,
    local_fade_in_weight: f32,

    stop_anim_idx: u16,
    prepare_stop_time: f32,

    root_motion: LogicMultiRootMotion,
    prev_anim_queue: Vec<StateActionAnimation>,
}

extend!(LogicActionMoveNpc, LogicActionBase);

impl LogicActionMoveNpc {
    pub fn new(ctx: &mut ContextUpdate, inst_act: Rc<InstActionMoveNpc>) -> XResult<LogicActionMoveNpc> {
        let root_motion =
            LogicMultiRootMotion::new_with_capacity(ctx, inst_act.animations(), inst_act.animations_count())?;

        let turn_angle_step = Vec2xz::from_angle(PI / s2ff_round(inst_act.turn_time).max(1.0));
        let turn_cos_step = libm::cosf(PI / s2ff_round(inst_act.turn_time).max(1.0));

        Ok(LogicActionMoveNpc {
            _base: LogicActionBase {
                derive_level: LEVEL_MOVE,
                poise_level: inst_act.poise_level,
                ..LogicActionBase::new(ctx.gene.gen_action_id(), inst_act.clone())
            },
            inst: inst_act,
            turn_angle_step,
            turn_cos_step,

            mode: ActionMoveNpcMode::Start,
            current_time: 0.0,
            local_fade_in_weight: 1.0,

            stop_anim_idx: u16::MAX,
            prepare_stop_time: 0.0,

            root_motion,
            prev_anim_queue: Vec::new(),
        })
    }
}

unsafe impl LogicActionAny for LogicActionMoveNpc {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::MoveNpc
    }

    fn restore(&mut self, state: &(dyn StateActionAny + 'static)) -> XResult<()> {
        if state.id != self._base.id {
            return xresf!(LogicIDMismatch; "state.id={}, self.id={}", state.id, self._base.id);
        }
        let state = state.cast::<StateActionMoveNpc>()?;

        self._base.restore(&state._base);
        self.mode = state.mode;
        self.current_time = state.current_time;
        self.local_fade_in_weight = state.local_fade_in_weight;
        self.stop_anim_idx = state.stop_anim_idx;
        self.prepare_stop_time = state.prepare_stop_time;
        self.root_motion.restore(&state.root_motion);

        self.prev_anim_queue.clear();
        let prev_len = state.animations.len().saturating_sub(1);
        for anim in state.animations.iter().take(prev_len) {
            self.prev_anim_queue.push(anim.clone());
        }
        Ok(())
    }

    fn save(&self) -> Box<dyn StateActionAny> {
        let mut state = Box::new(StateActionMoveNpc {
            _base: self._base.save(self.typ()),
            mode: self.mode,
            current_time: self.current_time,
            local_fade_in_weight: self.local_fade_in_weight,
            stop_anim_idx: self.stop_anim_idx,
            prepare_stop_time: self.prepare_stop_time,
            root_motion: self.root_motion.save(),
        });

        debug_assert!(self.prev_anim_queue.len() + 1 <= MAX_ACTION_ANIMATION);
        state.animations.extend(self.prev_anim_queue.iter().cloned());
        state.animations.push(self.save_current_animation());
        state
    }

    fn start(
        &mut self,
        ctx: &mut ContextUpdate,
        ctxa: &mut ContextAction,
        args: &ActionStartArgs,
    ) -> XResult<ActionStartReturn> {
        self._base.start(ctx, ctxa, args)?;
        self.prepare_start(ctxa)
    }

    fn update(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<ActionUpdateReturn> {
        self._base.update(ctx, ctxa)?;

        let res = match self.mode {
            ActionMoveNpcMode::Start => self.update_start(ctxa)?,
            ActionMoveNpcMode::Move => self.update_move(ctxa)?,
            ActionMoveNpcMode::Stop => self.update_stop(ctxa)?,
        };

        if let Operation::Enter(new_mode) = res.operation {
            // Save previous animation
            self.prev_anim_queue.push(self.save_current_animation());
            while self.prev_anim_queue.len() >= MAX_ACTION_ANIMATION {
                self.prev_anim_queue.remove(0);
            }

            match new_mode {
                ActionMoveNpcMode::Move => self.prepare_move(ctxa)?,
                ActionMoveNpcMode::Stop => self.prepare_stop(ctxa)?,
                ActionMoveNpcMode::Start => return xres!(Unexpected; "unreachable start")?,
            }
        }
        else if matches!(res.operation, Operation::Exit) {
            self.stop(ctx, ctxa)?;
        }

        // Clear saved animation, if fade in is complete
        if self.local_fade_in_weight >= 1.0 {
            self.prev_anim_queue.clear();
        }

        let mut ret = ActionUpdateReturn::new();
        ret.set_direction(res.new_direction);
        ret.set_velocity(res.new_direction.as_vec3a() * res.new_speed);
        Ok(ret)
    }
}

#[derive(Debug)]
struct UpdateRes {
    new_direction: Vec2xz,
    new_speed: f32,
    operation: Operation,
}

impl UpdateRes {
    #[inline]
    fn new(new_direction: Vec2xz) -> Self {
        Self {
            new_direction,
            new_speed: 0.0,
            operation: Operation::Keep,
        }
    }

    // #[inline]
    // fn is_keep(&self) -> bool {
    //     matches!(self.operation, Operation::Keep)
    // }

    #[inline]
    fn set_dir_speed(&mut self, direction: Vec2xz, speed: f32) {
        self.new_direction = direction;
        self.new_speed = speed;
    }

    #[inline]
    fn enter(&mut self, new_mode: ActionMoveNpcMode) {
        self.operation = Operation::Enter(new_mode);
    }

    #[inline]
    fn exit(&mut self) {
        self.operation = Operation::Exit;
    }
}

#[derive(Debug, Default)]
enum Operation {
    #[default]
    Keep,
    Enter(ActionMoveNpcMode),
    Exit,
}

impl LogicActionMoveNpc {
    #[inline]
    fn init_anim(&mut self, mode: ActionMoveNpcMode) {
        self.mode = mode;
        self.current_time = 0.0;
        self.local_fade_in_weight = 1.0;
    }

    #[inline]
    fn set_prepare_stop(&mut self, stop_anim_idx: u16, prepare_stop_time: f32) {
        self.stop_anim_idx = stop_anim_idx;
        self.prepare_stop_time = prepare_stop_time;
    }

    #[inline]
    fn clear_prepare_stop(&mut self) {
        self.stop_anim_idx = u16::MAX;
        self.prepare_stop_time = 0.0;
    }

    fn prepare_start(&mut self, _ctxa: &mut ContextAction) -> XResult<ActionStartReturn> {
        self.init_anim(ActionMoveNpcMode::Start);
        self.root_motion.set_local_id(self.inst.anim_start.local_id, 0.0)?;
        Ok(ActionStartReturn::new())
    }

    fn update_start(&mut self, ctxa: &mut ContextAction) -> XResult<UpdateRes> {
        let inst_act = self.inst.clone();
        let chara_dir = ctxa.chara_phy.direction_xz();
        let mut res = UpdateRes::new(chara_dir);

        self.current_time += ctxa.time_step;
        self.handle_fade_in(&inst_act.anim_start, ctxa.time_step);

        self.root_motion
            .update(inst_act.anim_start.ratio_saturating(self.current_time))?;
        let speed = self.root_motion.position_delta().xz().length() * ctxa.frac_1_time_step * self.inst.speed_ratio;
        res.set_dir_speed(chara_dir, speed);

        let move_dir = match ctxa.ai_thinking {
            Some(ai_thinking) => ai_thinking.move_dir,
            None => {
                log::warn!(
                    "LogicActionMoveNpc::update_start() missing ai_thinking, action_id={}, tmpl_action={}, chara_id={}, tmpl_character={}",
                    self.id,
                    self.inst.tmpl_id,
                    ctxa.chara_id,
                    ctxa.inst_chara.tmpl_character,
                );
                res.exit();
                return Ok(res);
            }
        };

        if loose_le!(move_dir.length(), 0.0) {
            if self.stop_anim_idx == u16::MAX {
                let anim_ratio = inst_act.anim_start.ratio_saturating(self.current_time);
                if let Some((stop_idx, stop_time)) =
                    self.find_stop_by_anim_ratio(&inst_act.anim_start, anim_ratio, false)
                {
                    self.set_prepare_stop(stop_idx, stop_time);
                }
                else {
                    if let Some((stop_idx, stop_time)) = self.find_stop_by_anim_ratio(&inst_act.anim_move, 0.0, true) {
                        self.set_prepare_stop(stop_idx, stop_time + inst_act.anim_start.duration - self.current_time);
                    }
                    else {
                        res.exit();
                        return Ok(res);
                    }
                }
            }
        }
        else {
            self.clear_prepare_stop();
            res.set_dir_speed(self.turn_towards(chara_dir, move_dir), speed);
        }

        // Check preparing stop
        if self.stop_anim_idx != u16::MAX {
            if loose_ge!(self.current_time, self.prepare_stop_time) {
                res.enter(ActionMoveNpcMode::Stop);
            }
        }
        else {
            if loose_ge!(self.current_time, inst_act.anim_start.duration) {
                res.enter(ActionMoveNpcMode::Move);
            }
        }
        Ok(res)
    }

    fn prepare_move(&mut self, _ctxa: &mut ContextAction) -> XResult<()> {
        self.init_anim(ActionMoveNpcMode::Move);
        self.local_fade_in_weight = ifelse!(self.inst.anim_move.fade_in <= 0.0, 1.0, 0.0);
        self.root_motion.set_local_id(self.inst.anim_move.local_id, 0.0)?;
        Ok(())
    }

    fn update_move(&mut self, ctxa: &mut ContextAction) -> XResult<UpdateRes> {
        let inst_act = self.inst.clone();
        let chara_dir = ctxa.chara_phy.direction_xz();
        let mut res = UpdateRes::new(chara_dir);
        let move_dir = match ctxa.ai_thinking {
            Some(ai_thinking) => ai_thinking.move_dir,
            None => {
                log::warn!(
                    "LogicActionMoveNpc::update_move() missing ai_thinking, action_id={}, tmpl_action={}, chara_id={}, tmpl_character={}",
                    self.id,
                    self.inst.tmpl_id,
                    ctxa.chara_id,
                    ctxa.inst_chara.tmpl_character,
                );
                res.exit();
                return Ok(res);
            }
        };

        self.current_time += ctxa.time_step;
        self.handle_fade_in(&inst_act.anim_move, ctxa.time_step);

        self.root_motion
            .update(inst_act.anim_move.ratio_unsafe(self.current_time))?;
        let speed = self.root_motion.position_delta().xz().length() * ctxa.frac_1_time_step * self.inst.speed_ratio;
        res.set_dir_speed(chara_dir, speed);

        if loose_le!(move_dir.length(), 0.0) {
            if self.stop_anim_idx == u16::MAX {
                let anim_ratio = inst_act.anim_move.ratio_warpping(self.current_time);
                match self.find_stop_by_anim_ratio(&inst_act.anim_move, anim_ratio, true) {
                    Some((stop_idx, stop_time)) => self.set_prepare_stop(stop_idx, stop_time),
                    None => {
                        res.exit();
                        return Ok(res);
                    }
                }
            }
        }
        else {
            self.clear_prepare_stop();
            res.set_dir_speed(self.turn_towards(chara_dir, move_dir), speed);
        }

        // Check preparing stop
        if self.stop_anim_idx != u16::MAX {
            if loose_ge!(self.current_time, self.prepare_stop_time) {
                res.enter(ActionMoveNpcMode::Stop);
            }
        }
        Ok(res)
    }

    fn prepare_stop(&mut self, ctxa: &mut ContextAction) -> XResult<()> {
        self.init_anim(ActionMoveNpcMode::Stop);
        self.prepare_stop_time = 0.0;

        let stop = match self.inst.stops.get(self.stop_anim_idx as usize) {
            Some(stop) => stop,
            None => {
                log::warn!(
                    "LogicActionMoveNpc::prepare_stop() missing stop, action_id={}, tmpl_action={}, stop_anim_idx={}, chara_id={}, tmpl_character={}",
                    self.id,
                    self.inst.tmpl_id,
                    self.stop_anim_idx,
                    ctxa.chara_id,
                    ctxa.inst_chara.tmpl_character,
                );
                return Ok(());
            }
        };

        self.local_fade_in_weight = ifelse!(stop.anim.fade_in <= 0.0, 1.0, 0.0);
        self.root_motion.set_local_id(stop.anim.local_id, 0.0)?;
        Ok(())
    }

    fn update_stop(&mut self, ctxa: &mut ContextAction) -> XResult<UpdateRes> {
        let inst_act = self.inst.clone();
        let chara_dir = ctxa.chara_phy.direction_xz();
        let mut res = UpdateRes::new(chara_dir);

        let stop = match inst_act.stops.get(self.stop_anim_idx as usize) {
            Some(stop) => stop,
            None => {
                res.exit();
                return Ok(res);
            }
        };

        self.current_time += ctxa.time_step;
        self.handle_fade_in(&stop.anim, ctxa.time_step);

        self.root_motion.update(stop.anim.ratio_saturating(self.current_time))?;
        let speed = self.root_motion.position_delta().xz().length() * ctxa.frac_1_time_step * self.inst.speed_ratio;
        res.set_dir_speed(chara_dir, speed);

        if loose_ge!(self.current_time, stop.anim.duration) {
            res.exit();
        }
        Ok(res)
    }

    #[inline(always)]
    fn handle_fade_in(&mut self, anim: &InstAnimation, time_step: f32) {
        if self.local_fade_in_weight < 1.0 {
            self.local_fade_in_weight = anim.fade_in_weight(self.local_fade_in_weight, time_step);
        }

        if self.fade_in_weight < 1.0 {
            self.fade_in_weight = anim.fade_in_weight(self.fade_in_weight, time_step);
        }
    }

    #[inline(always)]
    fn turn_towards(&self, chara_dir: Vec2xz, move_dir: Vec2xz) -> Vec2xz {
        let diff_cos = chara_dir.dot(move_dir);
        if diff_cos >= self.turn_cos_step {
            return move_dir;
        }

        let mut diff_cross_sign = chara_dir.angle_to_sign(move_dir);
        if diff_cross_sign == 0.0 {
            diff_cross_sign = 1.0;
        }

        let mut step_vec = self.turn_angle_step;
        step_vec.z *= diff_cross_sign;
        step_vec.rotate(chara_dir)
    }

    fn find_stop_by_anim_ratio(
        &self,
        inst_anim: &InstAnimation,
        anim_ratio: f32,
        wrapping: bool,
    ) -> Option<(u16, f32)> {
        let (stop_idx, stop_ratio) = self
            .inst
            .find_stop_by_anim_ratio(inst_anim.files, anim_ratio, wrapping)?;

        let delta_ratio = if stop_ratio >= anim_ratio {
            stop_ratio - anim_ratio
        }
        else {
            debug_assert!(wrapping, "invalid non-wrapping stop ratio");
            1.0 - anim_ratio + stop_ratio
        };

        let prepare_stop_time = self.current_time + delta_ratio * inst_anim.duration;
        Some((stop_idx as u16, prepare_stop_time))
    }

    fn save_current_animation(&self) -> StateActionAnimation {
        let (anim, ratio) = match self.mode {
            ActionMoveNpcMode::Start => {
                let anim = &self.inst.anim_start;
                (anim, anim.ratio_saturating(self.current_time))
            }
            ActionMoveNpcMode::Move => {
                let anim = &self.inst.anim_move;
                (anim, anim.ratio_warpping(self.current_time))
            }
            ActionMoveNpcMode::Stop => {
                let anim = match self.inst.stops.get(self.stop_anim_idx as usize) {
                    Some(stop) => &stop.anim,
                    None => &self.inst.anim_move,
                };
                (anim, anim.ratio_saturating(self.current_time))
            }
        };

        StateActionAnimation::new_with_anim(anim, ratio, self.local_fade_in_weight)
    }
}
