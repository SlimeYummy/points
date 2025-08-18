use cirtical_point_csgen::{CsEnum, CsOut};
use glam::{Vec3A, Vec3Swizzles};
use glam_ext::Vec2xz;
use libm;
use log::warn;
use std::f32::consts::PI;
use std::fmt::Debug;
use std::rc::Rc;
use std::u16;

use crate::consts::{CFG_SPF, MAX_ACTION_ANIMATION};
use crate::instance::InstActionMove;
use crate::logic::action::base::{
    impl_state_action, ActionUpdateReturn, ContextAction, LogicActionAny, LogicActionBase, StateActionAnimation,
    StateActionAny, StateActionBase, StateActionType,
};
use crate::logic::action::root_motion::LogicMultiRootMotion;
use crate::logic::game::ContextUpdate;
use crate::logic::StateMultiRootMotion;
use crate::template::TmplType;
use crate::utils::{extend, s2ff_round, xres, xresf, Castable, XResult, LEVEL_IDLE, LEVEL_UNBREAKABLE};
use crate::{loose_ge, loose_le, strict_gt, strict_lt};

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
    pub current_time: f32,
    pub start_anim_idx: u16,
    pub turn_anim_idx: u16,
    pub stop_anim_idx: u16,

    pub root_motion: StateMultiRootMotion,
    pub start_turn_angle_step: Vec2xz,

    pub local_fade_in_weight: f32,
    pub anim_offset_time: f32,
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
    current_time: f32,
    start_anim_idx: u16,
    turn_anim_idx: u16,
    stop_anim_idx: u16,

    root_motion: LogicMultiRootMotion,
    start_turn_angle_step: Vec2xz,

    local_fade_in_weight: f32,
    anim_offset_time: f32,
    anim_queue: Vec<StateActionAnimation>,
}

extend!(LogicActionMove, LogicActionBase);

impl LogicActionMove {
    pub fn new(ctx: &mut ContextUpdate<'_>, inst_act: Rc<InstActionMove>) -> XResult<LogicActionMove> {
        let root_motion =
            LogicMultiRootMotion::new_with_capacity(ctx, inst_act.animations(), inst_act.animations_size())?;
        let move_track = root_motion.track(inst_act.anim_move.local_id);
        let speed_ratio = inst_act.move_speed / move_track.whole_position().xz().length();

        let turn_angle_step = Vec2xz::from_angle(PI / s2ff_round(inst_act.turn_time));
        let turn_cos_step = libm::cosf(PI / s2ff_round(inst_act.turn_time));
        println!("{:?}", (inst_act.turn_time, turn_angle_step, turn_cos_step));

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
            current_time: 0.0,
            start_anim_idx: u16::MAX,
            turn_anim_idx: u16::MAX,
            stop_anim_idx: u16::MAX,

            root_motion,
            start_turn_angle_step: Vec2xz::ZERO,

            local_fade_in_weight: 1.0,
            anim_offset_time: 0.0,
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
        self.current_time = state.current_time;
        self.start_anim_idx = state.start_anim_idx;
        self.turn_anim_idx = state.turn_anim_idx;
        self.stop_anim_idx = state.stop_anim_idx;

        self.root_motion.restore(&state.root_motion);
        self.start_turn_angle_step = state.start_turn_angle_step;

        self.local_fade_in_weight = state.local_fade_in_weight;
        self.anim_offset_time = state.anim_offset_time;

        self.anim_queue.clear();
        for anim in &state.animations {
            if !anim.is_empty() {
                self.anim_queue.push(anim.clone());
            }
        }
        Ok(())
    }

    fn start(&mut self, ctx: &mut ContextUpdate<'_>, ctxa: &mut ContextAction<'_>) -> XResult<()> {
        self._base.start(ctx, ctxa)?;
        self.prepare_start(ctxa)?;
        Ok(())
    }

    fn update(&mut self, ctx: &mut ContextUpdate<'_>, ctxa: &mut ContextAction<'_>) -> XResult<ActionUpdateReturn> {
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
        ret.set_velocity(res.new_velocity);
        // println!("{:?} {:?} {:?}", self.mode, ret.new_velocity.unwrap().length(), ret.new_direction.unwrap());
        Ok(ret)
    }

    fn save(&self) -> Box<dyn StateActionAny> {
        let mut state = Box::new(StateActionMove {
            _base: self._base.save(self.typ(), self.tmpl_typ()),
            mode: self.mode,
            current_time: self.current_time,
            start_anim_idx: self.start_anim_idx,
            turn_anim_idx: self.turn_anim_idx,
            stop_anim_idx: self.stop_anim_idx,

            root_motion: self.root_motion.save(),
            start_turn_angle_step: self.start_turn_angle_step,

            local_fade_in_weight: self.local_fade_in_weight,
            anim_offset_time: self.anim_offset_time,
        });
        state.fade_in_weight = self.fade_in_weight;

        debug_assert!(self.anim_queue.len() <= state.animations.len());
        state.animations[0..self.anim_queue.len()].clone_from_slice(&self.anim_queue);
        state.animations[self.anim_queue.len()] = self.save_current_animation();
        state
    }
}

#[derive(Debug)]
struct UpdateRes {
    new_direction: Vec2xz,
    new_velocity: Vec3A,
    operation: Operation,
}

impl UpdateRes {
    #[inline]
    fn new(new_direction: Vec2xz) -> Self {
        Self {
            new_direction,
            new_velocity: Vec3A::ZERO,
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
        self.new_velocity = direction.as_vec3a() * speed;
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
    fn prepare_start(&mut self, ctxa: &mut ContextAction<'_>) -> XResult<()> {
        self.current_time = 0.0;
        self.local_fade_in_weight = 1.0;
        self.anim_offset_time = 0.0;

        // In order to ensure the accuracy of the player's first attack direction (first attack in free state).
        // The turn of the ActionMoveMode::Start is unbreakable.
        self.derive_level = LEVEL_UNBREAKABLE.max(self.inst.derive_level);

        let chara_dir = ctxa.chara_physics.direction();
        let world_move = ctxa.input_vars.optimized_world_move();

        let move_dir = world_move.move_dir().unwrap_or(chara_dir);
        let angle = chara_dir.angle_to(move_dir);

        if let Some((start_idx, start)) = self.inst.find_start_by_angle(angle) {
            self.mode = ActionMoveMode::Start;
            self.root_motion.set_track(start.anim.local_id, 0.0)?;

            self.start_anim_idx = start_idx as u16;
            self.start_turn_angle_step = Vec2xz::from_angle(angle / s2ff_round(start.turn_in_place_end + CFG_SPF));

            println!(
                "!!!!!!!!!!! {:?} {:?} {:?} {} {:?}",
                chara_dir,
                move_dir,
                self.start_turn_angle_step,
                self.start_turn_angle_step.to_angle().to_degrees(),
                start
            );
            self.fade_in_weight = start.anim.fade_in_weight(self.fade_in_weight, ctxa.time_step);
        }
        else {
            warn!("Angle: {}", angle);
            self.mode = ActionMoveMode::StartNoAnim;
            self.root_motion.set_track(self.inst.anim_move.local_id, 0.0)?;

            self.fade_in_weight = self.inst.anim_move.fade_in_weight(self.fade_in_weight, ctxa.time_step);
        }
        Ok(())
    }

    fn update_start(&mut self, ctxa: &mut ContextAction<'_>) -> XResult<UpdateRes> {
        let inst_act = self.inst.clone();
        let chara_dir = ctxa.chara_physics.direction();
        let world_move = ctxa.input_vars.optimized_world_move();
        let start = &inst_act.starts[self.start_anim_idx as usize];
        let mut res = UpdateRes::new(chara_dir);

        self.current_time += ctxa.time_step;
        debug_assert_eq!(self.anim_offset_time, 0.0);
        let adjusted_time = self.current_time; // anim_offset_time must be 0.0

        if strict_gt!(adjusted_time, start.turn_in_place_end) {
            self.derive_level = inst_act.derive_level;
        }

        if self.fade_in_weight < 1.0 {
            self.fade_in_weight = start.anim.fade_in_weight(self.fade_in_weight, ctxa.time_step);
        }

        self.root_motion.update(start.anim.ratio_saturating(adjusted_time))?;
        let speed = self.root_motion.position_delta().xz().length() / ctxa.time_step * self.speed_ratio;
        res.set_dir_speed(chara_dir, speed); // Setup default values

        'X: {
            // Turn inplace
            if loose_le!(adjusted_time, start.turn_in_place_end + CFG_SPF) {
                let chara_dir = ctxa.chara_physics.direction();
                let new_direction = self.start_turn_angle_step.rotate(chara_dir);
                res.set_dir_speed(new_direction, speed);
                println!(
                    "update_start(trun) => adjusted_time:{} chara_dir:{:?} new_direction:{:?}",
                    adjusted_time, chara_dir, new_direction
                );
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

    fn update_start_no_anim(&mut self, _ctxa: &mut ContextAction<'_>) -> XResult<UpdateRes> {
        unimplemented!() // crach here
    }

    fn prepare_move(&mut self, _ctxa: &mut ContextAction<'_>, _change: &OptEnter) -> XResult<()> {
        let prev_using_move = self.mode.using_move_anim();

        self.mode = ActionMoveMode::Move;
        self.fade_in_weight = 1.0;

        if !prev_using_move {
            self.current_time = 0.0;

            self.local_fade_in_weight = if self.inst.anim_move.fade_in <= 0.0 { 1.0 } else { 0.0 };
            self.anim_offset_time = 0.0;

            self.root_motion.set_track(self.inst.anim_move.local_id, 0.0)?;
        }
        Ok(())
    }

    fn update_move(&mut self, ctxa: &mut ContextAction<'_>) -> XResult<UpdateRes> {
        let inst_act = self.inst.clone();
        let chara_dir = ctxa.chara_physics.direction();
        let world_move = ctxa.input_vars.optimized_world_move();
        let mut res = UpdateRes::new(chara_dir);

        self.current_time += ctxa.time_step;
        let adjusted_time = self.current_time + self.anim_offset_time;

        self.derive_level = inst_act.derive_level;

        if self.local_fade_in_weight < 1.0 {
            self.local_fade_in_weight = self
                .inst
                .anim_move
                .fade_in_weight(self.local_fade_in_weight, ctxa.time_step);
        }

        self.root_motion.update(self.inst.anim_move.ratio_safe(adjusted_time))?;
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
                let phase = inst_act.anim_move.ratio_saturating(adjusted_time);
                match inst_act.find_stop_by_phase(phase) {
                    Some((stop_idx, _, offset)) => {
                        res.enter2(ActionMoveMode::Stop, stop_idx as u16, offset);
                    }
                    None => res.exit(),
                };
            }
        }

        return Ok(res);
    }

    fn prepare_turn(&mut self, _ctxa: &mut ContextAction<'_>, _goto: &OptEnter) -> XResult<()> {
        unimplemented!()
    }

    fn update_turn(&mut self, _ctxa: &mut ContextAction<'_>) -> XResult<UpdateRes> {
        unimplemented!()
    }

    fn prepare_stop(&mut self, _ctxa: &mut ContextAction<'_>, enter: &OptEnter) -> XResult<()> {
        let prev_using_move = self.mode.using_move_anim();

        if enter.new_mode == ActionMoveMode::Stop {
            let stop = &self.inst.stops[enter.anim_idx as usize];

            self.mode = ActionMoveMode::Stop;
            self.current_time = 0.0;
            self._base.fade_in_weight = 1.0;
            self.stop_anim_idx = enter.anim_idx;

            self.anim_offset_time = enter.anim_offset_time;
            self.local_fade_in_weight = if stop.anim.fade_in <= 0.0 { 1.0 } else { 0.0 };

            self.root_motion
                .set_track(stop.anim.local_id, stop.anim.ratio_saturating(self.anim_offset_time))?;
        }
        else {
            // ActionMoveMode::StopNoAnim
            self.mode = ActionMoveMode::StopNoAnim;
            self.fade_in_weight = 1.0;

            if !prev_using_move {
                self.current_time = 0.0;

                self.anim_offset_time = 0.0;
                self.fade_in_weight = if self.inst.anim_move.fade_in <= 0.0 { 1.0 } else { 0.0 };

                self.root_motion.set_track(self.inst.anim_move.local_id, 0.0)?;
            }
        }
        Ok(())
    }

    fn update_stop(&mut self, ctxa: &mut ContextAction<'_>) -> XResult<UpdateRes> {
        let chara_dir = ctxa.chara_physics.direction();
        let inst_act = self.inst.clone();
        let stop = &inst_act.stops[self.stop_anim_idx as usize];
        let mut res = UpdateRes::new(chara_dir);

        self.current_time += ctxa.time_step;
        let adjusted_time = self.current_time + self.anim_offset_time;

        if loose_le!(adjusted_time, stop.speed_down_end) {
            self.derive_level = inst_act.derive_level;
        }
        else {
            self.derive_level = LEVEL_IDLE;
        }

        if self.local_fade_in_weight < 1.0 {
            self.local_fade_in_weight = stop.anim.fade_in_weight(self.local_fade_in_weight, ctxa.time_step);
        }

        self.root_motion.update(stop.anim.ratio_saturating(adjusted_time))?;
        let speed = self.root_motion.position_delta().xz().length() / ctxa.time_step * self.speed_ratio;
        res.set_dir_speed(chara_dir, speed);

        if loose_ge!(adjusted_time, stop.anim.duration) {
            res.exit();
        }

        return Ok(res);
    }

    fn update_stop_no_anim(&mut self, _ctxa: &mut ContextAction<'_>) -> XResult<UpdateRes> {
        unimplemented!()
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

        StateActionAnimation {
            animation_id: anim.local_id,
            files: anim.files.clone(),
            ratio,
            weight: self.local_fade_in_weight,
        }
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

//     static RUN_OZZ: &str = "girl_run";

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
