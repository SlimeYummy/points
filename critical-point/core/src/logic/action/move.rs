use critical_point_csgen::{CsEnum, CsOut};
use glam::Vec3Swizzles;
use glam_ext::Vec2xz;
use libm;
use std::f32::consts::PI;
use std::fmt::Debug;
use std::rc::Rc;
use std::u16;

use crate::animation::RootTrackName;
use crate::consts::{CFG_SPF, MAX_ACTION_ANIMATION};
use crate::instance::{InstActionMove, InstAnimation};
use crate::logic::action::base::{
    impl_state_action, ActionStartReturn, ActionUpdateReturn, ContextAction, LogicActionAny, LogicActionBase,
    StateActionAnimation, StateActionAny, StateActionBase, StateActionType,
};
use crate::logic::action::root_motion::LogicMultiRootMotion;
use crate::logic::game::ContextUpdate;
use crate::logic::StateMultiRootMotion;
use crate::template::TmplType;
use crate::utils::{
    calc_fade_in, extend, lerp, loose_ge, loose_le, ratio_warpping, s2ff_round, strict_gt, xres, xresf, Castable,
    XResult, ifelse
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
pub enum ActionMoveMode {
    Start,
    Move,
    Turn,
    Stop,
    StartNoAnim,
    StopNoAnim,
}

impl ActionMoveMode {
    #[inline]
    fn using_move_anim(&self) -> bool {
        matches!(
            self,
            ActionMoveMode::Move | ActionMoveMode::StartNoAnim | ActionMoveMode::StopNoAnim
        )
    }
}

#[repr(C)]
#[derive(
    Debug, PartialEq, rkyv::Archive, serde::Serialize, serde::Deserialize, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateActionMove {
    pub _base: StateActionBase,

    pub mode: ActionMoveMode,
    pub smooth_move_switch: bool,
    pub start_anim_idx: u16,
    pub turn_anim_idx: u16,
    pub stop_anim_idx: u16,

    pub current_time: f32,
    pub anim_offset_time: f32,
    pub local_fade_in_weight: f32,

    pub start_turn_angle_step: Vec2xz,
    pub smooth_move_start_speed: f32,
    pub root_motion: StateMultiRootMotion,
}

extend!(StateActionMove, StateActionBase);
impl_state_action!(StateActionMove, ActionMove, Move, "Move");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionMove {
    _base: LogicActionBase,
    inst: Rc<InstActionMove>,
    speed_ratio: f32,
    turn_angle_step: Vec2xz,
    turn_cos_step: f32,

    mode: ActionMoveMode,
    smooth_move_switch: bool,
    start_anim_idx: u16,
    turn_anim_idx: u16,
    stop_anim_idx: u16,

    current_time: f32,
    anim_offset_time: f32,
    local_fade_in_weight: f32,

    start_turn_angle_step: Vec2xz,
    smooth_move_start_speed: f32,
    root_motion: LogicMultiRootMotion,
    anim_queue: Vec<StateActionAnimation>,
}

extend!(LogicActionMove, LogicActionBase);

impl LogicActionMove {
    pub fn new(ctx: &mut ContextUpdate, inst_act: Rc<InstActionMove>) -> XResult<LogicActionMove> {
        let root_motion =
            LogicMultiRootMotion::new_with_capacity(ctx, inst_act.animations(), inst_act.animations_count())?;
        let move_track = root_motion.track(inst_act.anim_move.local_id);
        let speed_ratio = inst_act.move_speed / move_track.whole_position(RootTrackName::Default).xz().length();

        let turn_angle_step = Vec2xz::from_angle(PI / s2ff_round(inst_act.turn_time));
        let turn_cos_step = libm::cosf(PI / s2ff_round(inst_act.turn_time));
        // println!("{:?}", (inst_act.turn_time, turn_angle_step, turn_cos_step));

        Ok(LogicActionMove {
            _base: LogicActionBase {
                derive_level: inst_act.derive_level,
                poise_level: inst_act.poise_level,
                ..LogicActionBase::new(ctx.gene.gen_id(), inst_act.clone())
            },
            inst: inst_act.clone(),
            speed_ratio,
            turn_angle_step,
            turn_cos_step,

            mode: ActionMoveMode::Move,
            smooth_move_switch: false,
            start_anim_idx: u16::MAX,
            turn_anim_idx: u16::MAX,
            stop_anim_idx: u16::MAX,

            current_time: 0.0,
            anim_offset_time: 0.0,
            local_fade_in_weight: 1.0,

            start_turn_angle_step: Vec2xz::ZERO,
            smooth_move_start_speed: 0.0,
            root_motion,
            anim_queue: Vec::new(),
        })
    }
}

unsafe impl LogicActionAny for LogicActionMove {
    #[inline]
    fn typ(&self) -> StateActionType {
        StateActionType::Move
    }

    #[inline]
    fn tmpl_typ(&self) -> TmplType {
        TmplType::ActionMove
    }

    fn restore(&mut self, state: &(dyn StateActionAny + 'static)) -> XResult<()> {
        if state.id != self._base.id {
            return xresf!(LogicIDMismatch; "state.id={}, self.id={}", state.id, self._base.id);
        }
        let state = state.cast::<StateActionMove>()?;

        self._base.restore(&state._base);
        self.mode = state.mode;
        self.smooth_move_switch = state.smooth_move_switch;
        self.start_anim_idx = state.start_anim_idx;
        self.turn_anim_idx = state.turn_anim_idx;
        self.stop_anim_idx = state.stop_anim_idx;

        self.current_time = state.current_time;
        self.anim_offset_time = state.anim_offset_time;
        self.local_fade_in_weight = state.local_fade_in_weight;

        self.start_turn_angle_step = state.start_turn_angle_step;
        self.smooth_move_start_speed = state.smooth_move_start_speed;
        self.root_motion.restore(&state.root_motion);

        self.anim_queue.clear();
        for anim in &state.animations {
            if !anim.is_empty() {
                self.anim_queue.push(anim.clone());
            }
        }
        Ok(())
    }

    fn save(&self) -> Box<dyn StateActionAny> {
        let mut state = Box::new(StateActionMove {
            _base: self._base.save(self.typ(), self.tmpl_typ()),
            mode: self.mode,
            smooth_move_switch: self.smooth_move_switch,
            start_anim_idx: self.start_anim_idx,
            turn_anim_idx: self.turn_anim_idx,
            stop_anim_idx: self.stop_anim_idx,

            current_time: self.current_time,
            anim_offset_time: self.anim_offset_time,
            local_fade_in_weight: self.local_fade_in_weight,

            start_turn_angle_step: self.start_turn_angle_step,
            smooth_move_start_speed: self.smooth_move_start_speed,
            root_motion: self.root_motion.save(),
        });

        debug_assert!(self.anim_queue.len() <= state.animations.len());
        state.animations[0..self.anim_queue.len()].clone_from_slice(&self.anim_queue);
        state.animations[self.anim_queue.len()] = self.save_current_animation();
        state
    }

    fn start(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<ActionStartReturn> {
        self._base.start(ctx, ctxa)?;

        let mut ret = ActionStartReturn::new();
        if self.try_enter_smooth(ctxa)? {
            ret.prev_fade_update = true;
        }
        else {
            self.prepare_start(ctxa)?;
        }
        Ok(ret)
    }

    fn update(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<ActionUpdateReturn> {
        self._base.update(ctx, ctxa)?;

        let res = match self.mode {
            ActionMoveMode::Start => self.update_start(ctxa)?,
            ActionMoveMode::Move => self.update_move(ctxa)?,
            ActionMoveMode::Turn => self.update_turn(ctxa)?,
            ActionMoveMode::Stop => self.update_stop(ctxa)?,
            ActionMoveMode::StartNoAnim => self.update_start_no_anim(ctxa)?,
            ActionMoveMode::StopNoAnim => self.update_stop_no_anim(ctxa)?,
        };

        if let Operation::Enter(enter) = res.operation {
            // Save previous animation
            self.anim_queue.push(self.save_current_animation());
            while self.anim_queue.len() >= MAX_ACTION_ANIMATION {
                self.anim_queue.remove(0);
            }

            match enter.new_mode {
                ActionMoveMode::Move | ActionMoveMode::StartNoAnim => self.prepare_move(ctxa, &enter)?,
                ActionMoveMode::Turn => self.prepare_turn(ctxa, &enter)?,
                ActionMoveMode::Stop => self.prepare_stop(ctxa, &enter)?,
                ActionMoveMode::Start | ActionMoveMode::StopNoAnim => return xres!(Unexpected; "unreachable start")?,
            }
        }
        else if matches!(res.operation, Operation::Exit) {
            self.stop(ctx, ctxa)?;
        }

        // Clear saved animation, if fade in is complete
        if self.local_fade_in_weight >= 1.0 {
            self.anim_queue.clear();
        }

        let mut ret = ActionUpdateReturn::new();
        ret.set_direction(res.new_direction);
        if self.smooth_move_switch && self.fade_in_weight < 1.0 {
            ret.set_velocity(
                res.new_direction.as_vec3a() * lerp(self.smooth_move_start_speed, res.new_speed, self.fade_in_weight),
            );
        }
        else {
            ret.set_velocity(res.new_direction.as_vec3a() * res.new_speed);
        }
        // println!("{:?} {:?} {:?}", self.mode, ret.new_velocity.unwrap().length(), ret.new_direction.unwrap());
        Ok(ret)
    }

    fn fade_start(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<bool> {
        self._base.fade_start(ctx, ctxa)?;
        Ok(self.mode == ActionMoveMode::Move)
    }

    fn fade_update(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<()> {
        self._base.fade_update(ctx, ctxa)?;
        self.fade_update_impl(ctxa)
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

    #[inline]
    fn is_keep(&self) -> bool {
        matches!(self.operation, Operation::Keep)
    }

    #[inline]
    fn set_dir_speed(&mut self, direction: Vec2xz, speed: f32) {
        self.new_direction = direction;
        self.new_speed = speed;
    }

    #[inline]
    fn enter(&mut self, new_mode: ActionMoveMode, anim_idx: u16) {
        self.operation = Operation::Enter(OptEnter {
            new_mode,
            anim_idx,
            anim_offset_time: 0.0,
        });
    }

    #[inline]
    fn enter2(&mut self, new_mode: ActionMoveMode, anim_idx: u16, anim_offset_time: f32) {
        self.operation = Operation::Enter(OptEnter {
            new_mode,
            anim_idx,
            anim_offset_time,
        });
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
    Enter(OptEnter),
    Exit,
}

#[derive(Debug)]
struct OptEnter {
    new_mode: ActionMoveMode,
    anim_idx: u16,
    anim_offset_time: f32,
}

impl LogicActionMove {
    fn prepare_start(&mut self, ctxa: &mut ContextAction) -> XResult<ActionStartReturn> {
        let inst_act = self.inst.clone();
        let chara_dir = ctxa.chara_physics.direction();
        let world_move = ctxa.input_vars.optimized_world_move();
        let move_dir = world_move.move_dir().unwrap_or(chara_dir);
        let angle = chara_dir.angle_to(move_dir);

        if let Some((start_idx, start)) = inst_act.find_start_by_angle(angle) {
            self.mode = ActionMoveMode::Start;
            self.start_anim_idx = start_idx as u16;

            self.current_time = 0.0;
            self.anim_offset_time = 0.0;
            self.local_fade_in_weight = 1.0;

            self.start_turn_angle_step = Vec2xz::from_angle(angle / s2ff_round(start.turn_in_place_end + CFG_SPF));
            self.root_motion.set_local_id(start.anim.local_id, 0.0)?;

            self.derive_level = inst_act.special_derive_level;
            self.fade_in_weight = start.anim.fade_in_weight(0.0, ctxa.time_step);
        }
        else {
            log::warn!("Angle: {}", angle);
            self.mode = ActionMoveMode::StartNoAnim;

            self.current_time = 0.0;
            self.anim_offset_time = 0.0;
            self.local_fade_in_weight = 1.0;

            self.derive_level = inst_act.derive_level;
            self.fade_in_weight = inst_act.anim_move.fade_in_weight(0.0, ctxa.time_step);

            self.root_motion.set_local_id(inst_act.anim_move.local_id, 0.0)?;
        }
        Ok(ActionStartReturn::new())
    }

    fn update_start(&mut self, ctxa: &mut ContextAction) -> XResult<UpdateRes> {
        let inst_act = self.inst.clone();
        let chara_dir = ctxa.chara_physics.direction();
        let world_move = ctxa.input_vars.optimized_world_move();
        let start = &inst_act.starts[self.start_anim_idx as usize];
        let mut res = UpdateRes::new(chara_dir);

        self.current_time += ctxa.time_step;
        let adjusted_time = self.current_time + self.anim_offset_time;

        if strict_gt!(adjusted_time, start.turn_in_place_end) {
            self.derive_level = inst_act.derive_level;
        }

        self.handle_fade_in(&start.anim, ctxa.time_step);

        self.root_motion.update(start.anim.ratio_saturating(adjusted_time))?;
        let speed = self.root_motion.position_delta().xz().length() / ctxa.time_step * self.speed_ratio;
        res.set_dir_speed(chara_dir, speed); // Setup default values

        'X: {
            // Turn inplace
            if loose_le!(adjusted_time, start.turn_in_place_end + CFG_SPF) {
                let chara_dir = ctxa.chara_physics.direction();
                let new_direction = self.start_turn_angle_step.rotate(chara_dir);
                res.set_dir_speed(new_direction, speed);
                // println!(
                //     "update_start(trun) => adjusted_time:{} chara_dir:{:?} new_direction:{:?}",
                //     adjusted_time, chara_dir, new_direction
                // );
            }
            else {
                // Move
                if let Some(move_dir) = world_move.move_dir() {
                    let diff_cos = chara_dir.dot(move_dir);
                    let diff_cross_sign = chara_dir.angle_to_sign(move_dir);

                    // Enter turn animation
                    if !inst_act.check_direct_turn_by_cos(diff_cos, diff_cross_sign) {
                        let angle = chara_dir.angle_to(move_dir);
                        if let Some((turn_idx, _)) = inst_act.find_turn_by_angle(angle) {
                            // TODO: ...
                            res.enter(ActionMoveMode::Turn, turn_idx as u16);
                            break 'X;
                        }
                    }

                    // Direct turn
                    let new_direction = if diff_cos >= self.turn_cos_step {
                        move_dir
                    }
                    else {
                        let mut step_vec = self.turn_angle_step;
                        step_vec.z *= diff_cross_sign;
                        step_vec.rotate(chara_dir)
                    };
                    res.set_dir_speed(new_direction, speed);
                }
                // Stop
                else {
                    if loose_le!(adjusted_time, start.quick_stop_end) {
                        res.exit();
                    }
                    else {
                        let phase = inst_act
                            .anim_move
                            .ratio_saturating(inst_act.anim_move.duration - (start.anim.duration - adjusted_time));
                        match inst_act.find_stop_by_phase(phase) {
                            Some((stop_idx, _, offset)) => {
                                res.enter2(ActionMoveMode::Stop, stop_idx as u16, offset);
                            }
                            None => res.exit(),
                        };
                    }
                }
            }
        }

        if res.is_keep() && loose_ge!(adjusted_time, start.anim.duration) {
            res.enter(ActionMoveMode::Move, u16::MAX);
        }

        // println!("update_start => ratio:{:?} speed:{:?} res:{:?}", start.anim.ratio_saturating(adjusted_time), speed, res);
        return Ok(res);
    }

    fn update_start_no_anim(&mut self, _ctxa: &mut ContextAction) -> XResult<UpdateRes> {
        unimplemented!() // crach here
    }

    fn prepare_move(&mut self, _ctxa: &mut ContextAction, _change: &OptEnter) -> XResult<()> {
        let prev_mode = self.mode;
        self.mode = ActionMoveMode::Move;

        if !prev_mode.using_move_anim() {
            self.current_time = 0.0;
            self.anim_offset_time = 0.0;
            self.local_fade_in_weight = ifelse!(self.inst.anim_move.fade_in <= 0.0, 1.0, 0.0);

            self.root_motion.set_local_id(self.inst.anim_move.local_id, 0.0)?;
        }

        self._base.derive_level = self.inst.derive_level;
        Ok(())
    }

    fn update_move(&mut self, ctxa: &mut ContextAction) -> XResult<UpdateRes> {
        let inst_act = self.inst.clone();
        let chara_dir = ctxa.chara_physics.direction();
        let world_move = ctxa.input_vars.optimized_world_move();
        let mut res = UpdateRes::new(chara_dir);

        self.current_time += ctxa.time_step;
        let adjusted_time = self.current_time + self.anim_offset_time;

        self.derive_level = inst_act.derive_level;

        self.handle_fade_in(&inst_act.anim_move, ctxa.time_step);

        self.root_motion
            .update(self.inst.anim_move.ratio_unsafe(adjusted_time))?;
        let speed = self.root_motion.position_delta().xz().length() / ctxa.time_step * self.speed_ratio;
        res.set_dir_speed(chara_dir, speed); // Setup default values

        'X: {
            // Move
            if let Some(move_dir) = world_move.move_dir() {
                let diff_cos = chara_dir.dot(move_dir);
                let diff_cross_sign = chara_dir.angle_to_sign(move_dir);

                // Enter turn animation
                if !inst_act.check_direct_turn_by_cos(diff_cos, diff_cross_sign) {
                    let angle = chara_dir.angle_to(move_dir);
                    if let Some((turn_idx, _)) = inst_act.find_turn_by_angle(angle) {
                        // TODO: ...
                        res.enter(ActionMoveMode::Turn, turn_idx as u16);
                        break 'X;
                    }
                }

                // Direct turn
                let new_direction = if diff_cos >= self.turn_cos_step {
                    move_dir
                }
                else {
                    let mut step_vec = self.turn_angle_step;
                    step_vec.z *= diff_cross_sign;
                    step_vec.rotate(chara_dir)
                };
                res.set_dir_speed(new_direction, speed);
            }
            // Stop
            else {
                let phase = inst_act.anim_move.ratio_warpping(adjusted_time);
                match inst_act.find_stop_by_phase(phase) {
                    Some((stop_idx, _, offset)) => {
                        log::info!(
                            "find_stop_by_phase phase={} stop_idx={} offset={}",
                            phase,
                            stop_idx,
                            offset
                        );
                        res.enter2(ActionMoveMode::Stop, stop_idx as u16, offset);
                    }
                    None => res.exit(),
                };
            }
        }

        return Ok(res);
    }

    fn prepare_turn(&mut self, _ctxa: &mut ContextAction, _goto: &OptEnter) -> XResult<()> {
        unimplemented!()
    }

    fn update_turn(&mut self, _ctxa: &mut ContextAction) -> XResult<UpdateRes> {
        unimplemented!()
    }

    fn prepare_stop(&mut self, _ctxa: &mut ContextAction, enter: &OptEnter) -> XResult<()> {
        if enter.new_mode == ActionMoveMode::Stop {
            let stop = &self.inst.stops[enter.anim_idx as usize];
            self.stop_anim_idx = enter.anim_idx;

            self.mode = ActionMoveMode::Stop;

            self.current_time = 0.0;
            self.anim_offset_time = enter.anim_offset_time;
            self.local_fade_in_weight = ifelse!(stop.anim.fade_in <= 0.0, 1.0, 0.0);

            self.root_motion
                .set_local_id(stop.anim.local_id, stop.anim.ratio_saturating(self.anim_offset_time))?;

            self._base.derive_level = self.inst.derive_level;
            self.set_derive_self(true);
        }
        else {
            let prev_mode = self.mode;
            self.mode = ActionMoveMode::StopNoAnim;

            if !prev_mode.using_move_anim() {
                self.current_time = 0.0;
                self.anim_offset_time = 0.0;
                self.local_fade_in_weight = ifelse!(self.inst.stop_time <= 0.0, 1.0, 0.0);

                self.root_motion.set_local_id(self.inst.anim_move.local_id, 0.0)?;
            }

            self._base.derive_level = self.inst.derive_level;
            self.set_derive_self(true);
        }
        Ok(())
    }

    fn update_stop(&mut self, ctxa: &mut ContextAction) -> XResult<UpdateRes> {
        let chara_dir = ctxa.chara_physics.direction();
        let inst_act = self.inst.clone();
        let stop = &inst_act.stops[self.stop_anim_idx as usize];
        let mut res = UpdateRes::new(chara_dir);

        self.current_time += ctxa.time_step;
        let adjusted_time = self.current_time + self.anim_offset_time;

        // self.derive_level = match loose_le!(adjusted_time, stop.speed_down_end) {
        //     true => self.inst.special_derive_level,
        //     false => self.inst.derive_level,
        // };

        self.handle_fade_in(&stop.anim, ctxa.time_step);

        self.root_motion.update(stop.anim.ratio_saturating(adjusted_time))?;
        let speed: f32 = self.root_motion.position_delta().xz().length() / ctxa.time_step * self.speed_ratio;
        res.set_dir_speed(chara_dir, speed);

        if loose_ge!(adjusted_time, stop.anim.duration) {
            res.exit();
        }

        return Ok(res);
    }

    fn update_stop_no_anim(&mut self, _ctxa: &mut ContextAction) -> XResult<UpdateRes> {
        unimplemented!()
    }

    #[inline(always)]
    fn handle_fade_in(&mut self, anim: &InstAnimation, time_step: f32) {
        if self.local_fade_in_weight < 1.0 {
            self.local_fade_in_weight = anim.fade_in_weight(self.local_fade_in_weight, time_step);
        }

        if self.fade_in_weight < 1.0 {
            if !self.smooth_move_switch {
                self.fade_in_weight = anim.fade_in_weight(self.fade_in_weight, time_step);
            }
            else {
                self.fade_in_weight = calc_fade_in(self.fade_in_weight, time_step, self.inst.smooth_move_duration);
                if self.fade_in_weight >= 1.0 {
                    self.smooth_move_switch = false;
                }
            }
        }
    }

    fn try_enter_smooth(&mut self, ctxa: &mut ContextAction) -> XResult<bool> {
        let Some(prev_act) = ctxa.prev_action
        else {
            return Ok(false);
        };

        if let Ok(prev_mov) = prev_act.cast::<LogicActionMove>() {
            if prev_mov.mode == ActionMoveMode::Move {
                let prev_adjusted_time = prev_mov.current_time + prev_mov.anim_offset_time;
                let prev_ratio = prev_mov.inst.anim_move.ratio_warpping(prev_adjusted_time);

                self.mode = ActionMoveMode::Move;
                self.smooth_move_switch = true;

                self.current_time = 0.0;
                self.anim_offset_time = self.inst.anim_move.duration * prev_ratio;
                self.local_fade_in_weight = 1.0;

                self.smooth_move_start_speed = prev_mov.root_motion.position_delta().xz().length() / ctxa.time_step;
                self.root_motion
                    .set_local_id(self.inst.anim_move.local_id, prev_ratio)?;

                self._base.fade_in_weight = 0.0;
                self._base.derive_level = self.inst.derive_level;
                return Ok(true);
            }
            else if prev_mov.mode == ActionMoveMode::Start {
                let prev_start = &prev_mov.inst.starts[prev_mov.start_anim_idx as usize];
                let prev_adjusted_time = prev_mov.current_time + prev_mov.anim_offset_time;
                let prev_x_ratio = ratio_warpping(
                    prev_adjusted_time - prev_start.turn_in_place_end,
                    prev_start.anim.duration - prev_start.turn_in_place_end,
                );

                if let Some((start_idx, start)) = self.inst.find_start_by_angle(0.0) {
                    self.mode = ActionMoveMode::Start;
                    self.smooth_move_switch = true;
                    self.start_anim_idx = start_idx as u16;

                    self.current_time = 0.0;
                    self.anim_offset_time =
                        start.turn_in_place_end + prev_x_ratio * (start.anim.duration - prev_start.turn_in_place_end);
                    self.local_fade_in_weight = 1.0;

                    self.smooth_move_start_speed = prev_mov.root_motion.position_delta().xz().length() / ctxa.time_step;
                    self.root_motion.set_local_id(
                        self.inst.anim_move.local_id,
                        start.anim.ratio_warpping(self.anim_offset_time),
                    )?;

                    self._base.fade_in_weight = 0.0;
                    self._base.derive_level = self.inst.derive_level;
                    return Ok(true);
                }
            }
            else if prev_mov.mode == ActionMoveMode::Stop {
                let prev_adjusted_time = prev_mov.current_time + prev_mov.anim_offset_time;
                let prev_ratio = prev_mov
                    .inst
                    .calc_stop_phase(prev_mov.stop_anim_idx as usize, prev_adjusted_time);
                log::info!(
                    "calc_stop_phase stop_idx={} time={} phase={:?}",
                    prev_mov.stop_anim_idx,
                    prev_adjusted_time,
                    prev_ratio
                );
                println!(
                    "{:?}",
                    prev_mov.inst.stops[prev_mov.stop_anim_idx as usize].leave_phase_table
                );
                let Some(prev_ratio) = prev_ratio
                else {
                    return Ok(false);
                };

                self.mode = ActionMoveMode::Move;
                self.smooth_move_switch = true;

                self.current_time = 0.0;
                self.anim_offset_time = self.inst.anim_move.duration * prev_ratio;
                self.local_fade_in_weight = 1.0;

                self.smooth_move_start_speed = prev_mov.root_motion.position_delta().xz().length() / ctxa.time_step;
                self.root_motion
                    .set_local_id(self.inst.anim_move.local_id, prev_ratio)?;

                self._base.fade_in_weight = 0.0;
                self._base.derive_level = self.inst.derive_level;
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn fade_update_impl(&mut self, ctxa: &mut ContextAction) -> XResult<()> {
        match self.mode {
            ActionMoveMode::Move => {
                self.current_time += ctxa.time_step;
                if self.local_fade_in_weight < 1.0 {
                    self.local_fade_in_weight = self
                        .inst
                        .anim_move
                        .fade_in_weight(self.local_fade_in_weight, ctxa.time_step);
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn save_current_animation(&self) -> StateActionAnimation {
        let adjusted_time = self.current_time + self.anim_offset_time;
        let anim;
        let ratio;
        match self.mode {
            ActionMoveMode::Start => match self.inst.starts.get(self.start_anim_idx as usize) {
                Some(start) => {
                    anim = &start.anim;
                    ratio = anim.ratio_saturating(adjusted_time);
                }
                None => {
                    anim = &self.inst.anim_move;
                    ratio = anim.ratio_warpping(adjusted_time);
                }
            },
            ActionMoveMode::Move | ActionMoveMode::StartNoAnim | ActionMoveMode::StopNoAnim => {
                anim = &self.inst.anim_move;
                ratio = anim.ratio_warpping(adjusted_time);
            }
            ActionMoveMode::Turn => match self.inst.turns.get(self.turn_anim_idx as usize) {
                Some(turn) => {
                    anim = &turn.anim;
                    ratio = anim.ratio_saturating(adjusted_time);
                }
                None => {
                    anim = &self.inst.anim_move;
                    ratio = anim.ratio_warpping(adjusted_time);
                }
            },
            ActionMoveMode::Stop => match self.inst.stops.get(self.stop_anim_idx as usize) {
                Some(stop) => {
                    anim = &stop.anim;
                    ratio = anim.ratio_saturating(adjusted_time);
                }
                None => {
                    anim = &self.inst.anim_move;
                    ratio = anim.ratio_warpping(adjusted_time);
                }
            },
        };

        StateActionAnimation::new_with_anim(&anim, ratio, self.local_fade_in_weight)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::consts::{DEFAULT_TOWARD_DIR_3D, SPF};
//     use crate::logic::action::base::LogicActionStatus;
//     use crate::logic::action::test_utils::*;
//     use crate::utils::tests::FrameTicker;
//     use crate::utils::{id, s2f, sb};
//     use approx::assert_ulps_eq;
//     use glam::{Quat, Vec3, Vec3A, Vec3Swizzles};

//     #[test]
//     fn test_state_rkyv() {
//         let mut raw_state = Box::new(StateActionMove {
//             _base: StateActionBase::new(StateActionType::Move, TmplType::ActionMove),
//             mode: ActionMoveMode::Move,
//             switch_time: 5.0,
//             current_time: 10.0,
//         });
//         raw_state.id = 123;
//         raw_state.tmpl_id = id!("Action.Instance.Run/1A");
//         raw_state.status = LogicActionStatus::Activing;
//         raw_state.first_frame = 15;
//         raw_state.last_frame = 99;
//         raw_state.derive_level = 1;
//         raw_state.poise_level = 2;
//         raw_state.animations[0] = StateActionAnimation::new(sb!("move"), 1, 0.5, 0.5);

//         let state = test_state_action_rkyv(raw_state, StateActionType::Move, TmplType::ActionMove).unwrap();
//         let state = state.cast::<StateActionMove>().unwrap();

//         assert_eq!(state.id, 123);
//         assert_eq!(state.tmpl_id, id!("Action.Instance.Run/1A"));
//         assert_eq!(state.status, LogicActionStatus::Activing);
//         assert_eq!(state.first_frame, 15);
//         assert_eq!(state.last_frame, 99);
//         assert_eq!(state.derive_level, 1);
//         assert_eq!(state.poise_level, 2);
//         assert_eq!(state.animations[0], StateActionAnimation::new(sb!("move"), 1, 0.5, 0.5));
//         assert_eq!(state.animations[1], StateActionAnimation::default());
//         assert_eq!(state.animations[2], StateActionAnimation::default());
//         assert_eq!(state.animations[3], StateActionAnimation::default());
//         assert_eq!(state.mode, ActionMoveMode::Move);
//         assert_eq!(state.switch_time, 5.0);
//         assert_eq!(state.current_time, 10.0);
//     }

//     fn new_move(tenv: &mut TestEnv) -> (LogicActionMove, Rc<InstActionMove>) {
//         let inst_act: Rc<InstActionMove> = tenv
//             .inst_player
//             .find_action_by_id(id!("Action.Instance.Run/1A"))
//             .unwrap();
//         let logic_act = LogicActionMove::new(&mut tenv.context_update(), inst_act.clone()).unwrap();
//         (logic_act, inst_act)
//     }

//     static RUN_OZZ: &str = "Girl_Run_Empty";

//     #[test]
//     fn test_logic_new() {
//         let mut tenv = TestEnv::new().unwrap();
//         let logic_move = new_move(&mut tenv).0;

//         assert_eq!(logic_move.tmpl_id(), id!("Action.Instance.Run/1A"));
//         assert!(logic_move.is_starting());
//         assert_eq!(logic_move.first_frame, 0);
//         assert_eq!(logic_move.last_frame, u32::MAX);
//         assert_eq!(logic_move.fade_in_weight, 0.0);
//         assert_ulps_eq!(logic_move.yam_ang_vel, FRAC_PI_2 / 0.4);
//         assert_ulps_eq!(logic_move.turn_ang_vel, PI / 1.0);
//         assert_eq!(logic_move.turn_threshold_cos, -1.0);
//         assert_eq!(logic_move.mode, ActionMoveMode::Start);
//         assert_eq!(logic_move.switch_time, 0.0);
//         assert_eq!(logic_move.current_time, 0.0);
//     }

//     #[test]
//     fn test_logic_first_update() {
//         let mut tenv = TestEnv::new().unwrap();

//         {
//             let (mut logic_move, inst_move) = new_move(&mut tenv);
//             let (mut ctx, mut ctxa) = tenv.contexts(true);
//             ctxa.input_vars.optimized_device_move.moving = true;
//             ctxa.input_vars.optimized_device_move.direction = Vec2::NEG_Y;

//             logic_move.start(&mut ctx, &mut ctxa).unwrap();
//             let ret = logic_move.update(&mut ctx, &mut ctxa).unwrap();
//             assert!(logic_move.is_activing());
//             assert_eq!(logic_move.mode, ActionMoveMode::Move);
//             assert_eq!(logic_move.current_time, 0.0);
//             assert_eq!(ret.state.fade_in_weight, SPF / inst_move.anim_move.fade_in);
//             assert_eq!(ret.state.animations[0].animation_id, ANIME_MOVE_ID);
//             assert_eq!(ret.state.animations[0].files, RUN_OZZ);
//             assert_eq!(ret.state.animations[0].ratio, 0.0);
//             assert_eq!(ret.state.animations[0].weight, 1.0);
//         }

//         {
//             let (mut logic_move, inst_move) = new_move(&mut tenv);
//             let (mut ctx, mut ctxa) = tenv.contexts(true);
//             ctxa.input_vars.optimized_device_move.moving = true;
//             ctxa.input_vars.optimized_device_move.direction = Vec2::Y;

//             logic_move.start(&mut ctx, &mut ctxa).unwrap();
//             let ret = logic_move.update(&mut ctx, &mut ctxa).unwrap();
//             assert!(logic_move.is_activing());
//             assert_eq!(logic_move.mode, ActionMoveMode::Start);
//             assert_eq!(logic_move.current_time, 0.0);
//             assert_eq!(ret.state.fade_in_weight, SPF / inst_move.anim_move.fade_in);
//             assert_eq!(ret.state.animations[0].animation_id, ANIME_MOVE_ID);
//             assert_eq!(ret.state.animations[0].files, RUN_OZZ);
//             assert_eq!(ret.state.animations[0].ratio, 0.0);
//             assert_eq!(ret.state.animations[0].weight, 1.0);
//         }
//     }

//     #[test]
//     fn test_logic_start() {
//         let mut tenv = TestEnv::new().unwrap();
//         let (mut logic_move, inst_move) = new_move(&mut tenv);

//         let (mut ctx, mut ctxa) = tenv.contexts(true);
//         logic_move.start(&mut ctx, &mut ctxa).unwrap();
//         for ft in FrameTicker::new(0..s2f(0.4)) {
//             let (mut ctx, mut ctxa) = tenv.contexts(true);
//             ctxa.input_vars.optimized_device_move.moving = true;
//             ctxa.input_vars.optimized_device_move.direction = Vec2::X;

//             let ret = logic_move.update(&mut ctx, &mut ctxa).unwrap();
//             assert!(logic_move.is_activing());
//             assert_eq!(logic_move.mode, ft.or_last(ActionMoveMode::Start, ActionMoveMode::Move));
//             assert_eq!(logic_move.current_time, ft.time);

//             assert_ulps_eq!(ret.state.fade_in_weight, inst_move.anim_move.fade_in_weight(ft.time(1)));
//             assert_eq!(ret.state.animations[0].animation_id, ANIME_MOVE_ID);
//             assert_eq!(ret.state.animations[0].files, RUN_OZZ);
//             assert_ulps_eq!(ret.state.animations[0].ratio, ft.time / inst_move.anim_move.duration);
//             assert_eq!(ret.state.animations[0].weight, 1.0);

//             let rot = ft.or_last(
//                 Quat::from_rotation_y(logic_move.yam_ang_vel * ft.time(1)),
//                 Quat::from_rotation_y(FRAC_PI_2),
//             );
//             assert_ulps_eq!(ret.new_direction.unwrap(), (rot * Vec3::Z).xz(),);
//             assert_ulps_eq!(
//                 ret.new_velocity.unwrap(),
//                 ft.or_last(Vec3A::ZERO, Vec3A::new(5.0, 0.0, 0.0))
//             );
//             tenv.chara_physics.set_direction(ret.new_direction.unwrap());
//         }
//     }

//     #[test]
//     fn test_logic_move_forward() {
//         let mut tenv = TestEnv::new().unwrap();
//         let (mut logic_move, inst_move) = new_move(&mut tenv);

//         let (mut ctx, mut ctxa) = tenv.contexts(false);
//         logic_move.start(&mut ctx, &mut ctxa).unwrap();
//         for ft in FrameTicker::new(0..3) {
//             let (mut ctx, mut ctxa) = tenv.contexts(false);
//             ctxa.input_vars.optimized_device_move.moving = true;
//             ctxa.input_vars.optimized_device_move.direction = Vec2::NEG_Y;

//             let ret = logic_move.update(&mut ctx, &mut ctxa).unwrap();
//             assert!(logic_move.is_activing());
//             assert_eq!(logic_move.mode, ActionMoveMode::Move);
//             assert_eq!(logic_move.current_time, ft.time);

//             assert_eq!(ret.state.fade_in_weight, 1.0);
//             assert_eq!(ret.state.animations[0].animation_id, ANIME_MOVE_ID);
//             assert_eq!(ret.state.animations[0].files, RUN_OZZ);
//             assert_eq!(ret.state.animations[0].ratio, ft.time / inst_move.anim_move.duration);
//             assert_eq!(ret.state.animations[0].weight, 1.0);

//             assert_ulps_eq!(ret.new_direction.unwrap(), Vec2::Y);
//             assert_ulps_eq!(ret.new_velocity.unwrap(), Vec3A::new(0.0, 0.0, 5.0));
//             tenv.chara_physics.set_direction(ret.new_direction.unwrap());
//         }
//     }

//     #[test]
//     fn test_logic_move_yam() {
//         let mut tenv = TestEnv::new().unwrap();
//         let (mut logic_move, inst_move) = new_move(&mut tenv);
//         let (mut ctx, mut ctxa) = tenv.contexts(false);
//         ctxa.input_vars.optimized_device_move.moving = true;
//         ctxa.input_vars.optimized_device_move.direction = Vec2::NEG_Y;
//         logic_move.start(&mut ctx, &mut ctxa).unwrap();
//         logic_move.update(&mut ctx, &mut ctxa).unwrap();

//         for ft in FrameTicker::new(0..s2f(0.4)) {
//             let (mut ctx, mut ctxa) = tenv.contexts(false);
//             ctxa.input_vars.optimized_device_move.moving = true;
//             ctxa.input_vars.optimized_device_move.direction = Vec2::NEG_X;

//             let ret = logic_move.update(&mut ctx, &mut ctxa).unwrap();
//             assert!(logic_move.is_activing());
//             assert_eq!(logic_move.mode, ActionMoveMode::Move);
//             assert_eq!(logic_move.current_time, ft.time(1));

//             assert_eq!(ret.state.fade_in_weight, 1.0);
//             assert_eq!(ret.state.animations[0].animation_id, ANIME_MOVE_ID);
//             assert_eq!(ret.state.animations[0].files, RUN_OZZ);
//             assert_eq!(ret.state.animations[0].ratio, ft.time(1) / inst_move.anim_move.duration);
//             assert_eq!(ret.state.animations[0].weight, 1.0);

//             let rot = ft.or_last(
//                 Quat::from_axis_angle(Vec3::Y, -logic_move.yam_ang_vel * ft.time(1)),
//                 Quat::from_rotation_y(-FRAC_PI_2),
//             );
//             assert_ulps_eq!(ret.new_direction.unwrap(), (rot * Vec3::Z).xz(),);
//             assert_ulps_eq!(ret.new_velocity.unwrap(), rot * Vec3A::new(0.0, 0.0, 5.0));
//             tenv.chara_physics.set_direction(ret.new_direction.unwrap());
//         }
//     }

//     #[test]
//     fn test_logic_move_turn() {}

//     #[test]
//     fn test_logic_move_stop() {
//         let mut tenv = TestEnv::new().unwrap();
//         let mut logic_move = new_move(&mut tenv).0;
//         let (mut ctx, mut ctxa) = tenv.contexts(false);
//         logic_move.start(&mut ctx, &mut ctxa).unwrap();

//         for ft in FrameTicker::new(0..10) {
//             let (mut ctx, mut ctxa) = tenv.contexts(false);
//             if !ft.last {
//                 ctxa.input_vars.optimized_device_move.moving = true;
//                 ctxa.input_vars.optimized_device_move.direction = Vec2::NEG_X;
//             } else {
//                 ctxa.input_vars.optimized_device_move.moving = false;
//                 ctxa.input_vars.optimized_device_move.direction = Vec2::ZERO;
//             }

//             let ret = logic_move.update(&mut ctx, &mut ctxa).unwrap();
//             if !ft.last {
//                 assert!(logic_move.is_activing());
//             } else {
//                 assert!(logic_move.is_stopping());
//             }
//             assert!(ret.derive_keeping.is_none());
//         }
//     }
// }
