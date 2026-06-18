use critical_point_macros::{csharp_enum, csharp_out};
use glam::Vec3Swizzles;
use glam_ext::Vec2xz;
use libm;
use std::f32::consts::PI;
use std::fmt::Debug;
use std::rc::Rc;
use std::u16;

use crate::animation::RootTrackName;
use crate::consts::{CFG_SPF, MAX_ACTION_ANIMATION};
use crate::input::{RefInputEventQueue, WorldMoveState};
use crate::instance::{InstActionMove, InstAnimation};
use crate::logic::action::base::{
    ActionStartArgs, ActionStartReturn, ActionUpdateReturn, ContextAction, LogicActionAny, LogicActionBase,
    StateActionAnimation, StateActionAny, StateActionBase, impl_state_action,
};
use crate::logic::action::root_motion::{LogicMultiRootMotion, StateMultiRootMotion};
use crate::logic::game::ContextUpdate;
use crate::utils::{
    ActionType, Castable, XResult, calc_fade_in, extend, ifelse, lerp, loose_ge, loose_le, ok_or, ratio_warpping,
    s2ff_round, strict_gt, xres, xresf,
};

#[csharp_enum]
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
)]
#[rkyv(derive(Debug))]
pub enum ActionMoveMode {
    Start,
    Move,
    Turn,
    Stop,
}

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, PartialEq, rkyv::Archive, serde::Serialize, serde::Deserialize, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
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
impl_state_action!(StateActionMove, Move, "Move");

///
/// Move action designs specifically for player-controlled characters.
///
/// In order to respond to player input more quickly, the start, loop, turn, and stop phases of
/// this action all need to consider rapid stopping and switching.
///
#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionMove {
    _base: LogicActionBase,
    inst: Rc<InstActionMove>,
    player_inputs: Option<RefInputEventQueue>,
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
    prev_anim_queue: Vec<StateActionAnimation>,
}

extend!(LogicActionMove, LogicActionBase);

impl LogicActionMove {
    pub fn new(ctx: &mut ContextUpdate, inst_act: Rc<InstActionMove>) -> XResult<LogicActionMove> {
        let root_motion =
            LogicMultiRootMotion::new_with_capacity(ctx, inst_act.animations(), inst_act.animations_count())?;

        let turn_angle_step = Vec2xz::from_angle(PI / s2ff_round(inst_act.turn_time).max(1.0));
        let turn_cos_step = libm::cosf(PI / s2ff_round(inst_act.turn_time).max(1.0));

        Ok(LogicActionMove {
            _base: LogicActionBase {
                keep_level: inst_act.keep_level,
                poise_level: inst_act.poise_level,
                ..LogicActionBase::new(ctx.identity.gen_action_id(), inst_act.clone())
            },
            inst: inst_act.clone(),
            player_inputs: None,
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
            prev_anim_queue: Vec::new(),
        })
    }
}

unsafe impl LogicActionAny for LogicActionMove {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::Move
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

        self.prev_anim_queue.clear();
        let prev_len = state.animations.len().saturating_sub(1);
        for anim in state.animations.iter().take(prev_len) {
            self.prev_anim_queue.push(anim.clone());
        }
        Ok(())
    }

    fn save(&self) -> Box<dyn StateActionAny> {
        let mut state = Box::new(StateActionMove {
            _base: self._base.save(self.typ()),
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

        if ctxa.inst_chara.is_player {
            self.player_inputs = Some(ctx.input.player_inputs(ctxa.chara_id)?);
        }
        else {
            log::warn!(
                "LogicActionMove::start: non-player character, action_id={}, tmpl_action={}, chara_id={}, tmpl_character={}",
                self.id,
                self.inst.tmpl_id,
                ctxa.chara_id,
                ctxa.inst_chara.tmpl_character,
            );
        }

        let mut ret = ActionStartReturn::new();
        if self.try_enter_smooth(ctxa, args)? {
            ret.prev_fade_update = true;
        }
        else {
            let world_move = match &self.player_inputs {
                Some(player_inputs) => player_inputs.borrow().last_variables()?.world_move(),
                None => WorldMoveState::default(),
            };
            self.prepare_start(ctxa, world_move)?;
        }
        Ok(ret)
    }

    fn update(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<ActionUpdateReturn> {
        self._base.update(ctx, ctxa)?;

        let world_move = match &self.player_inputs {
            Some(player_inputs) => player_inputs.borrow().last_variables()?.world_move(),
            None => WorldMoveState::default(),
        };

        let res = match self.mode {
            ActionMoveMode::Start => self.update_start(ctxa, world_move)?,
            ActionMoveMode::Move => self.update_move(ctxa, world_move)?,
            ActionMoveMode::Turn => self.update_turn(ctxa)?,
            ActionMoveMode::Stop => self.update_stop(ctxa)?,
        };

        if let Operation::Enter(enter) = res.operation {
            // Save previous animation
            self.prev_anim_queue.push(self.save_current_animation());
            while self.prev_anim_queue.len() >= MAX_ACTION_ANIMATION {
                self.prev_anim_queue.remove(0);
            }

            match enter.new_mode {
                ActionMoveMode::Move => self.prepare_move(ctxa, &enter)?,
                ActionMoveMode::Turn => self.prepare_turn(ctxa, &enter)?,
                ActionMoveMode::Stop => self.prepare_stop(ctxa, &enter)?,
                ActionMoveMode::Start => {
                    return xres!(Unexpected; "unreachable start")?;
                }
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
        if self.smooth_move_switch && self.fade_in_weight < 1.0 {
            ret.set_velocity(
                res.new_direction.as_vec3a() * lerp(self.smooth_move_start_speed, res.new_speed, self.fade_in_weight),
            );
        }
        else {
            ret.set_velocity(res.new_direction.as_vec3a() * res.new_speed);
        }
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
    #[inline]
    fn init_anim(&mut self, mode: ActionMoveMode, anim_idx: u16) {
        self.mode = mode;
        match mode {
            ActionMoveMode::Start => self.start_anim_idx = anim_idx,
            ActionMoveMode::Turn => self.turn_anim_idx = anim_idx,
            ActionMoveMode::Stop => self.stop_anim_idx = anim_idx,
            ActionMoveMode::Move => {}
        }

        self.current_time = 0.0;
        self.anim_offset_time = 0.0;
        self.local_fade_in_weight = 1.0;
        self.keep_level = self.inst.keep_level;
    }

    fn prepare_start(&mut self, ctxa: &mut ContextAction, world_move: WorldMoveState) -> XResult<ActionStartReturn> {
        let inst_act = self.inst.clone();
        let chara_dir = ctxa.chara_phy.direction_xz();
        let move_dir = world_move.move_dir().unwrap_or(chara_dir);
        let angle = chara_dir.angle_to(move_dir);

        if let Some((start_idx, start)) = inst_act.find_start_by_angle(angle) {
            self.init_anim(ActionMoveMode::Start, start_idx as u16);
            self.keep_level = inst_act.keep_level_special;
            self.start_turn_angle_step = Vec2xz::from_angle(angle / s2ff_round(start.turn_in_place_end + CFG_SPF));
            self.root_motion.set_local_id(start.anim.local_id, 0.0)?;
        }
        else {
            self.init_anim(ActionMoveMode::Move, 0);
            self.root_motion.set_local_id(inst_act.anim_move.local_id, 0.0)?;
        }
        Ok(ActionStartReturn::new())
    }

    fn update_start(&mut self, ctxa: &mut ContextAction, world_move: WorldMoveState) -> XResult<UpdateRes> {
        let inst_act = self.inst.clone();
        let chara_dir = ctxa.chara_phy.direction_xz();
        let start = &inst_act.starts[self.start_anim_idx as usize];
        let mut res = UpdateRes::new(chara_dir);

        self.current_time += ctxa.time_step;
        let adjusted_time = self.current_time + self.anim_offset_time;

        if strict_gt!(adjusted_time, start.turn_in_place_end) {
            self.keep_level = inst_act.keep_level;
        }

        self.handle_fade_in(&start.anim, ctxa.time_step);

        self.root_motion.update(start.anim.ratio_saturating(adjusted_time))?;
        let speed = self.root_motion.position_delta().xz().length() * ctxa.frac_1_time_step * self.inst.speed_ratio;
        res.set_dir_speed(chara_dir, speed); // Setup default values

        'X: {
            // Turn inplace
            if loose_le!(adjusted_time, start.turn_in_place_end + CFG_SPF) {
                let chara_dir = ctxa.chara_phy.direction_xz();
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
        return Ok(res);
    }

    fn prepare_move(&mut self, _ctxa: &mut ContextAction, _change: &OptEnter) -> XResult<()> {
        if self.mode != ActionMoveMode::Move {
            self.init_anim(ActionMoveMode::Move, 0);
            self.local_fade_in_weight = ifelse!(self.inst.anim_move.fade_in <= 0.0, 1.0, 0.0);
            self.root_motion.set_local_id(self.inst.anim_move.local_id, 0.0)?;
        }
        Ok(())
    }

    fn update_move(&mut self, ctxa: &mut ContextAction, world_move: WorldMoveState) -> XResult<UpdateRes> {
        let inst_act = self.inst.clone();
        let chara_dir = ctxa.chara_phy.direction_xz();
        let mut res = UpdateRes::new(chara_dir);

        self.current_time += ctxa.time_step;
        let adjusted_time = self.current_time + self.anim_offset_time;

        self.handle_fade_in(&inst_act.anim_move, ctxa.time_step);

        self.root_motion
            .update(self.inst.anim_move.ratio_unsafe(adjusted_time))?;
        let speed = self.root_motion.position_delta().xz().length() * ctxa.frac_1_time_step * self.inst.speed_ratio;
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
        self.init_anim(ActionMoveMode::Stop, enter.anim_idx);
        self.anim_offset_time = enter.anim_offset_time;

        let stop = &self.inst.stops[enter.anim_idx as usize];
        self.local_fade_in_weight = ifelse!(stop.anim.fade_in <= 0.0, 1.0, 0.0);
        self.root_motion
            .set_local_id(stop.anim.local_id, stop.anim.ratio_saturating(self.anim_offset_time))?;

        self.set_derive_self(true);
        Ok(())
    }

    fn update_stop(&mut self, ctxa: &mut ContextAction) -> XResult<UpdateRes> {
        let chara_dir = ctxa.chara_phy.direction_xz();
        let inst_act = self.inst.clone();
        let stop = &inst_act.stops[self.stop_anim_idx as usize];
        let mut res = UpdateRes::new(chara_dir);

        self.current_time += ctxa.time_step;
        let adjusted_time = self.current_time + self.anim_offset_time;

        // self.keep_level = match loose_le!(adjusted_time, stop.speed_down_end) {
        //     true => self.inst.keep_level_special,
        //     false => self.inst.keep_level,
        // };

        self.handle_fade_in(&stop.anim, ctxa.time_step);

        self.root_motion.update(stop.anim.ratio_saturating(adjusted_time))?;
        let speed: f32 =
            self.root_motion.position_delta().xz().length() * ctxa.frac_1_time_step * self.inst.speed_ratio;
        res.set_dir_speed(chara_dir, speed);

        if loose_ge!(adjusted_time, stop.anim.duration) {
            res.exit();
        }
        return Ok(res);
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

    fn try_enter_smooth(&mut self, ctxa: &mut ContextAction, args: &ActionStartArgs) -> XResult<bool> {
        fn init_smooth(zelf: &mut LogicActionMove, prev_mov: &LogicActionMove, mode: ActionMoveMode, anim_idx: u16) {
            zelf.init_anim(mode, anim_idx);
            zelf.smooth_move_switch = true;
            zelf.fade_in_weight = 0.0;
            zelf.keep_level = prev_mov.keep_level;
        }

        let prev_act = ok_or!(args.prev_action; return Ok(false));
        let inst_act = self.inst.clone();

        if !inst_act.smooth_move_froms.contains(&prev_act.tmpl_id()) {
            return Ok(false);
        }

        if let Ok(prev_mov) = prev_act.cast::<LogicActionMove>() {
            if prev_mov.mode == ActionMoveMode::Move {
                let prev_adjusted_time = prev_mov.current_time + prev_mov.anim_offset_time;
                let prev_ratio = prev_mov.inst.anim_move.ratio_warpping(prev_adjusted_time);

                init_smooth(self, &prev_mov, ActionMoveMode::Move, 0);
                self.anim_offset_time = inst_act.anim_move.duration * prev_ratio;

                self.smooth_move_start_speed =
                    prev_mov.root_motion.position_delta().xz().length() * ctxa.frac_1_time_step;
                self.root_motion.set_local_id(inst_act.anim_move.local_id, prev_ratio)?;
                return Ok(true);
            }
            else if prev_mov.mode == ActionMoveMode::Start {
                let prev_start = &prev_mov.inst.starts[prev_mov.start_anim_idx as usize];
                let prev_adjusted_time = prev_mov.current_time + prev_mov.anim_offset_time;
                let prev_x_ratio = ratio_warpping(
                    prev_adjusted_time - prev_start.turn_in_place_end,
                    prev_start.anim.duration - prev_start.turn_in_place_end,
                );

                if let Some((start_idx, start)) = inst_act.find_start_by_angle(0.0) {
                    init_smooth(self, &prev_mov, ActionMoveMode::Start, start_idx as u16);
                    self.anim_offset_time =
                        start.turn_in_place_end + prev_x_ratio * (start.anim.duration - start.turn_in_place_end);

                    self.smooth_move_start_speed =
                        prev_mov.root_motion.position_delta().xz().length() * ctxa.frac_1_time_step;
                    self.root_motion
                        .set_local_id(start.anim.local_id, start.anim.ratio_warpping(self.anim_offset_time))?;
                    return Ok(true);
                }
            }
            else if prev_mov.mode == ActionMoveMode::Stop {
                let prev_adjusted_time = prev_mov.current_time + prev_mov.anim_offset_time;
                let prev_ratio = prev_mov
                    .inst
                    .calc_stop_phase(prev_mov.stop_anim_idx as usize, prev_adjusted_time);
                // log::info!(
                //     "calc_stop_phase stop_idx={} time={} phase={:?}",
                //     prev_mov.stop_anim_idx,
                //     prev_adjusted_time,
                //     prev_ratio
                // );
                let prev_ratio = ok_or!(prev_ratio; return Ok(false));

                init_smooth(self, &prev_mov, ActionMoveMode::Move, 0);
                self.anim_offset_time = inst_act.anim_move.duration * prev_ratio;

                self.smooth_move_start_speed =
                    prev_mov.root_motion.position_delta().xz().length() * ctxa.frac_1_time_step;
                self.root_motion.set_local_id(inst_act.anim_move.local_id, prev_ratio)?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    // TODO: remove or fix this
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
            ActionMoveMode::Start => {
                anim = match self.inst.starts.get(self.start_anim_idx as usize) {
                    Some(start) => &start.anim,
                    None => &self.inst.anim_move,
                };
                ratio = anim.ratio_saturating(adjusted_time);
            }
            ActionMoveMode::Move => {
                anim = &self.inst.anim_move;
                ratio = anim.ratio_warpping(adjusted_time);
            }
            ActionMoveMode::Turn => {
                anim = match self.inst.turns.get(self.turn_anim_idx as usize) {
                    Some(turn) => &turn.anim,
                    None => &self.inst.anim_move,
                };
                ratio = anim.ratio_saturating(adjusted_time);
            }
            ActionMoveMode::Stop => {
                anim = match self.inst.stops.get(self.stop_anim_idx as usize) {
                    Some(stop) => &stop.anim,
                    None => &self.inst.anim_move,
                };
                ratio = anim.ratio_saturating(adjusted_time);
            }
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
//             _base: StateActionBase::new(ActionType::Move, TmplType::ActionMove),
//             mode: ActionMoveMode::Move,
//             switch_time: 5.0,
//             current_time: 10.0,
//         });
//         raw_state.id = 123;
//         raw_state.tmpl_id = id!("Action.Instance.Run/1A");
//         raw_state.status = LogicActionStatus::Activing;
//         raw_state.first_frame = 15;
//         raw_state.last_frame = 99;
//         raw_state.keep_level = 1;
//         raw_state.poise_level = 2;
//         raw_state.animations[0] = StateActionAnimation::new(sb!("move"), 1, 0.5, 0.5);

//         let state = test_state_action_rkyv(raw_state, ActionType::Move, TmplType::ActionMove).unwrap();
//         let state = state.cast::<StateActionMove>().unwrap();

//         assert_eq!(state.id, 123);
//         assert_eq!(state.tmpl_id, id!("Action.Instance.Run/1A"));
//         assert_eq!(state.status, LogicActionStatus::Activing);
//         assert_eq!(state.first_frame, 15);
//         assert_eq!(state.last_frame, 99);
//         assert_eq!(state.keep_level, 1);
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
//             assert!(logic_move.is_running());
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
//             assert!(logic_move.is_running());
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
//             assert!(logic_move.is_running());
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
//             tenv.chara_phy.set_direction(ret.new_direction.unwrap());
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
//             assert!(logic_move.is_running());
//             assert_eq!(logic_move.mode, ActionMoveMode::Move);
//             assert_eq!(logic_move.current_time, ft.time);

//             assert_eq!(ret.state.fade_in_weight, 1.0);
//             assert_eq!(ret.state.animations[0].animation_id, ANIME_MOVE_ID);
//             assert_eq!(ret.state.animations[0].files, RUN_OZZ);
//             assert_eq!(ret.state.animations[0].ratio, ft.time / inst_move.anim_move.duration);
//             assert_eq!(ret.state.animations[0].weight, 1.0);

//             assert_ulps_eq!(ret.new_direction.unwrap(), Vec2::Y);
//             assert_ulps_eq!(ret.new_velocity.unwrap(), Vec3A::new(0.0, 0.0, 5.0));
//             tenv.chara_phy.set_direction(ret.new_direction.unwrap());
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
//             assert!(logic_move.is_running());
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
//             tenv.chara_phy.set_direction(ret.new_direction.unwrap());
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
//                 assert!(logic_move.is_running());
//             } else {
//                 assert!(logic_move.is_stopping());
//             }
//             assert!(ret.derive_keeping.is_none());
//         }
//     }
// }
