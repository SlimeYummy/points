use cirtical_point_csgen::{CsEnum, CsOut};
use glam::Vec2;
use std::f32::consts::{FRAC_PI_2, PI};
use std::fmt::Debug;
use std::rc::Rc;

use crate::instance::InstActionMove;
use crate::logic::action::base::{
    continue_mode, impl_state_action, ActionUpdateReturn, ContextAction, LogicActionAny, LogicActionBase,
    StateActionAnimation, StateActionAny, StateActionBase, StateActionType,
};
use crate::logic::game::ContextUpdate;
use crate::template::TmplType;
use crate::utils::{dt_sign, extend, xresf, Castable, XResult, LEVEL_MOVE, LEVEL_UNBREAKABLE};

const ANIME_MOVE_ID: u32 = 1;
// const ANIME_TURN_LEFT_ID: u32 = 2;
// const ANIME_TURN_RIGHT_ID: u32 = 3;

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
    TurnLeft,
    TurnRight,
    Stop,
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
    pub switch_time: f32,
    pub current_time: f32,
}

extend!(StateActionMove, StateActionBase);
impl_state_action!(StateActionMove, ActionMove, Move, "Move");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionMove {
    _base: LogicActionBase,
    inst: Rc<InstActionMove>,

    yam_ang_vel: f32,
    turn_ang_vel: f32,
    turn_threshold_cos: f32,

    mode: ActionMoveMode,
    move_dir: Vec2,
    switch_time: f32,
    previous_time: f32,
    current_time: f32,
}

extend!(LogicActionMove, LogicActionBase);

impl LogicActionMove {
    pub fn new(ctx: &mut ContextUpdate<'_>, inst_act: Rc<InstActionMove>) -> XResult<LogicActionMove> {
        let yam_ang_vel = FRAC_PI_2 / (inst_act.yam_time as f32);
        let turn_ang_vel = PI / (inst_act.turn_time as f32);
        let turn_threshold_cos = match (&inst_act.anim_turn_left, &inst_act.anim_turn_right) {
            (Some(_), Some(_)) => 0.0,
            _ => -1.0,
        };

        Ok(LogicActionMove {
            _base: LogicActionBase {
                derive_level: inst_act.derive_level,
                poise_level: inst_act.poise_level,
                ..LogicActionBase::new(ctx.gene.gen_id(), inst_act.clone())
            },
            inst: inst_act,

            yam_ang_vel,
            turn_ang_vel,
            turn_threshold_cos,

            mode: ActionMoveMode::Start,
            move_dir: Vec2::ZERO,
            switch_time: 0.0,
            previous_time: 0.0,
            current_time: 0.0,
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
        self.switch_time = state.switch_time;
        self.current_time = state.current_time;
        Ok(())
    }

    fn start(&mut self, ctx: &mut ContextUpdate<'_>, ctxa: &mut ContextAction<'_>) -> XResult<()> {
        self._base.start(ctx, ctxa)?;
        self.current_time = -ctxa.time_step;
        self.mode = ActionMoveMode::Start;
        // In order to ensure the accuracy of the player's first attack direction (first attack in free state).
        // The turn of the ActionMoveMode::Start is unbreakable.
        self.derive_level = LEVEL_UNBREAKABLE;
        Ok(())
    }

    fn update(&mut self, ctx: &mut ContextUpdate<'_>, ctxa: &mut ContextAction<'_>) -> XResult<ActionUpdateReturn> {
        self._base.update(ctx, ctxa)?;

        let chara_dir = ctxa.chara_physics.direction();
        let input_move_dir = ctxa.input_vars.optimized_world_move().move_dir();
        let mut stop = false;
        if let Some(dir) = input_move_dir {
            self.move_dir = dir;
        } else {
            if self.mode != ActionMoveMode::Start {
                match &self.inst.anim_stop {
                    Some(_) => self.mode = ActionMoveMode::Stop,
                    None => stop = true,
                };
                self.move_dir = Vec2::ZERO;
            }
        }
        println!("derive_level: {:?}", self.derive_level);

        let diff_cos = chara_dir.dot(self.move_dir);
        let yam_step_angle_nosign = self.yam_ang_vel * ctxa.time_step;
        let yam_step_vec_nosign = Vec2::from_angle(yam_step_angle_nosign);
        let yam_step_cos = yam_step_vec_nosign.x;

        let mut new_chara_dir = chara_dir;
        let mut new_move_dir = Vec2::ZERO;
        loop {
            match self.mode {
                ActionMoveMode::Start => {
                    // diff_angle < step_angle
                    if diff_cos > yam_step_cos {
                        if input_move_dir.is_some() {
                            self.derive_level = LEVEL_MOVE;
                            continue_mode!(self.mode, ActionMoveMode::Move);
                        } else {
                            new_chara_dir = self.move_dir;
                            stop = true;
                        }
                    } else {
                        let mut step_vec = yam_step_vec_nosign;
                        step_vec.y *= dt_sign(chara_dir.perp_dot(self.move_dir));
                        new_chara_dir = step_vec.rotate(chara_dir);
                        println!(
                            "Start move_dir:{} chara_dir:{} new_chara_dir:{} step_vec:{} step_angle:{}",
                            self.move_dir,
                            chara_dir,
                            new_chara_dir,
                            step_vec,
                            step_vec.to_angle()
                        );
                    }
                }
                ActionMoveMode::Move => {
                    // diff_angle < step_angle
                    if diff_cos > yam_step_cos {
                        new_chara_dir = self.move_dir;
                    } else if diff_cos >= self.turn_threshold_cos {
                        let mut step_vec = yam_step_vec_nosign;
                        step_vec.y *= dt_sign(chara_dir.perp_dot(self.move_dir));
                        new_chara_dir = step_vec.rotate(chara_dir);
                        println!(
                            "chara_dir:{} new_chara_dir:{} step_vec:{} step_angle:{}",
                            chara_dir,
                            new_chara_dir,
                            step_vec,
                            step_vec.to_angle()
                        );
                    } else {
                        let sign = dt_sign(chara_dir.perp_dot(self.move_dir));
                        if sign < 0.0 {
                            continue_mode!(self.mode, ActionMoveMode::TurnLeft);
                        } else if sign > 0.0 {
                            continue_mode!(self.mode, ActionMoveMode::TurnRight);
                        }
                    }
                    new_move_dir = new_chara_dir * 1.0;
                }
                ActionMoveMode::TurnLeft => {}
                ActionMoveMode::TurnRight => {}
                ActionMoveMode::Stop => {}
            }
            break;
        }

        self.current_time += ctxa.time_step;

        if self.fade_in_weight < 1.0 {
            self.fade_in_weight += ctxa.time_step / self.inst.anim_move.fade_in;
            self.fade_in_weight = self.fade_in_weight.min(1.0);
        }

        if stop {
            self.stop(ctx, ctxa)?;
        }

        let mut ret = ActionUpdateReturn::new(self.save());
        ret.set_velocity_2d(new_move_dir * 5.0);
        ret.set_direction(new_chara_dir);
        Ok(ret)
    }

    fn save(&self) -> Box<dyn StateActionAny> {
        let mut state = Box::new(StateActionMove {
            _base: self._base.save(self.typ(), self.tmpl_typ()),
            mode: self.mode,
            switch_time: self.switch_time,
            current_time: self.current_time,
        });

        match self.mode {
            ActionMoveMode::Start | ActionMoveMode::Move => {
                state.animations[0] = StateActionAnimation {
                    animation_id: ANIME_MOVE_ID,
                    files: self.inst.anim_move.files.clone(),
                    ratio: self.inst.anim_move.ratio_warpping(self.current_time),
                    weight: 1.0,
                }
            }
            // ActionMoveMode::TurnLeft => {}
            // ActionMoveMode::TurnRight => {}
            // ActionMoveMode::Stop => {}
            _ => unreachable!("mode: {:?}", self.mode),
        };

        state.fade_in_weight = self.fade_in_weight;
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::{DEFAULT_TOWARD_DIR_3D, SPF};
    use crate::logic::action::base::LogicActionStatus;
    use crate::logic::action::test_utils::*;
    use crate::utils::tests::FrameTicker;
    use crate::utils::{id, s2f, sb};
    use approx::assert_ulps_eq;
    use glam::{Quat, Vec3, Vec3A, Vec3Swizzles};

    #[test]
    fn test_state_rkyv() {
        let mut raw_state = Box::new(StateActionMove {
            _base: StateActionBase::new(StateActionType::Move, TmplType::ActionMove),
            mode: ActionMoveMode::Move,
            switch_time: 5.0,
            current_time: 10.0,
        });
        raw_state.id = 123;
        raw_state.tmpl_id = id!("Action.Instance.Run/1A");
        raw_state.status = LogicActionStatus::Activing;
        raw_state.first_frame = 15;
        raw_state.last_frame = 99;
        raw_state.derive_level = 1;
        raw_state.poise_level = 2;
        raw_state.animations[0] = StateActionAnimation::new(sb!("move"), 1, 0.5, 0.5);

        let state = test_state_action_rkyv(raw_state, StateActionType::Move, TmplType::ActionMove).unwrap();
        let state = state.cast::<StateActionMove>().unwrap();

        assert_eq!(state.id, 123);
        assert_eq!(state.tmpl_id, id!("Action.Instance.Run/1A"));
        assert_eq!(state.status, LogicActionStatus::Activing);
        assert_eq!(state.first_frame, 15);
        assert_eq!(state.last_frame, 99);
        assert_eq!(state.derive_level, 1);
        assert_eq!(state.poise_level, 2);
        assert_eq!(state.animations[0], StateActionAnimation::new(sb!("move"), 1, 0.5, 0.5));
        assert_eq!(state.animations[1], StateActionAnimation::default());
        assert_eq!(state.animations[2], StateActionAnimation::default());
        assert_eq!(state.animations[3], StateActionAnimation::default());
        assert_eq!(state.mode, ActionMoveMode::Move);
        assert_eq!(state.switch_time, 5.0);
        assert_eq!(state.current_time, 10.0);
    }

    fn new_move(tenv: &mut TestEnv) -> (LogicActionMove, Rc<InstActionMove>) {
        let inst_act: Rc<InstActionMove> = tenv
            .inst_player
            .find_action_by_id(id!("Action.Instance.Run/1A"))
            .unwrap();
        let logic_act = LogicActionMove::new(&mut tenv.context_update(), inst_act.clone()).unwrap();
        (logic_act, inst_act)
    }

    static RUN_OZZ: &str = "girl_run";

    #[test]
    fn test_logic_new() {
        let mut tenv = TestEnv::new().unwrap();
        let logic_move = new_move(&mut tenv).0;

        assert_eq!(logic_move.tmpl_id(), id!("Action.Instance.Run/1A"));
        assert!(logic_move.is_starting());
        assert_eq!(logic_move.first_frame, 0);
        assert_eq!(logic_move.last_frame, u32::MAX);
        assert_eq!(logic_move.fade_in_weight, 0.0);
        assert_ulps_eq!(logic_move.yam_ang_vel, FRAC_PI_2 / 0.4);
        assert_ulps_eq!(logic_move.turn_ang_vel, PI / 1.0);
        assert_eq!(logic_move.turn_threshold_cos, -1.0);
        assert_eq!(logic_move.mode, ActionMoveMode::Start);
        assert_eq!(logic_move.switch_time, 0.0);
        assert_eq!(logic_move.current_time, 0.0);
    }

    #[test]
    fn test_logic_first_update() {
        let mut tenv = TestEnv::new().unwrap();

        {
            let (mut logic_move, inst_move) = new_move(&mut tenv);
            let (mut ctx, mut ctxa) = tenv.contexts(true);
            ctxa.input_vars.optimized_device_move.moving = true;
            ctxa.input_vars.optimized_device_move.direction = Vec2::NEG_Y;

            logic_move.start(&mut ctx, &mut ctxa).unwrap();
            let ret = logic_move.update(&mut ctx, &mut ctxa).unwrap();
            assert!(logic_move.is_activing());
            assert_eq!(logic_move.mode, ActionMoveMode::Move);
            assert_eq!(logic_move.current_time, 0.0);
            assert_eq!(ret.state.fade_in_weight, SPF / inst_move.anim_move.fade_in);
            assert_eq!(ret.state.animations[0].animation_id, ANIME_MOVE_ID);
            assert_eq!(ret.state.animations[0].files, RUN_OZZ);
            assert_eq!(ret.state.animations[0].ratio, 0.0);
            assert_eq!(ret.state.animations[0].weight, 1.0);
        }

        {
            let (mut logic_move, inst_move) = new_move(&mut tenv);
            let (mut ctx, mut ctxa) = tenv.contexts(true);
            ctxa.input_vars.optimized_device_move.moving = true;
            ctxa.input_vars.optimized_device_move.direction = Vec2::Y;

            logic_move.start(&mut ctx, &mut ctxa).unwrap();
            let ret = logic_move.update(&mut ctx, &mut ctxa).unwrap();
            assert!(logic_move.is_activing());
            assert_eq!(logic_move.mode, ActionMoveMode::Start);
            assert_eq!(logic_move.current_time, 0.0);
            assert_eq!(ret.state.fade_in_weight, SPF / inst_move.anim_move.fade_in);
            assert_eq!(ret.state.animations[0].animation_id, ANIME_MOVE_ID);
            assert_eq!(ret.state.animations[0].files, RUN_OZZ);
            assert_eq!(ret.state.animations[0].ratio, 0.0);
            assert_eq!(ret.state.animations[0].weight, 1.0);
        }
    }

    #[test]
    fn test_logic_start() {
        let mut tenv = TestEnv::new().unwrap();
        let (mut logic_move, inst_move) = new_move(&mut tenv);

        let (mut ctx, mut ctxa) = tenv.contexts(true);
        logic_move.start(&mut ctx, &mut ctxa).unwrap();
        for ft in FrameTicker::new(0..s2f(0.4)) {
            let (mut ctx, mut ctxa) = tenv.contexts(true);
            ctxa.input_vars.optimized_device_move.moving = true;
            ctxa.input_vars.optimized_device_move.direction = Vec2::X;

            let ret = logic_move.update(&mut ctx, &mut ctxa).unwrap();
            assert!(logic_move.is_activing());
            assert_eq!(logic_move.mode, ft.or_last(ActionMoveMode::Start, ActionMoveMode::Move));
            assert_eq!(logic_move.current_time, ft.time);

            assert_ulps_eq!(ret.state.fade_in_weight, inst_move.anim_move.fade_in_weight(ft.time(1)));
            assert_eq!(ret.state.animations[0].animation_id, ANIME_MOVE_ID);
            assert_eq!(ret.state.animations[0].files, RUN_OZZ);
            assert_ulps_eq!(ret.state.animations[0].ratio, ft.time / inst_move.anim_move.duration);
            assert_eq!(ret.state.animations[0].weight, 1.0);

            let rot = ft.or_last(
                Quat::from_rotation_y(logic_move.yam_ang_vel * ft.time(1)),
                Quat::from_rotation_y(FRAC_PI_2),
            );
            assert_ulps_eq!(ret.new_direction.unwrap(), (rot * Vec3::Z).xz(),);
            assert_ulps_eq!(
                ret.new_velocity.unwrap(),
                ft.or_last(Vec3A::ZERO, Vec3A::new(5.0, 0.0, 0.0))
            );
            tenv.chara_physics.set_direction(ret.new_direction.unwrap());
        }
    }

    #[test]
    fn test_logic_move_forward() {
        let mut tenv = TestEnv::new().unwrap();
        let (mut logic_move, inst_move) = new_move(&mut tenv);

        let (mut ctx, mut ctxa) = tenv.contexts(false);
        logic_move.start(&mut ctx, &mut ctxa).unwrap();
        for ft in FrameTicker::new(0..3) {
            let (mut ctx, mut ctxa) = tenv.contexts(false);
            ctxa.input_vars.optimized_device_move.moving = true;
            ctxa.input_vars.optimized_device_move.direction = Vec2::NEG_Y;

            let ret = logic_move.update(&mut ctx, &mut ctxa).unwrap();
            assert!(logic_move.is_activing());
            assert_eq!(logic_move.mode, ActionMoveMode::Move);
            assert_eq!(logic_move.current_time, ft.time);

            assert_eq!(ret.state.fade_in_weight, 1.0);
            assert_eq!(ret.state.animations[0].animation_id, ANIME_MOVE_ID);
            assert_eq!(ret.state.animations[0].files, RUN_OZZ);
            assert_eq!(ret.state.animations[0].ratio, ft.time / inst_move.anim_move.duration);
            assert_eq!(ret.state.animations[0].weight, 1.0);

            assert_ulps_eq!(ret.new_direction.unwrap(), Vec2::Y);
            assert_ulps_eq!(ret.new_velocity.unwrap(), Vec3A::new(0.0, 0.0, 5.0));
            tenv.chara_physics.set_direction(ret.new_direction.unwrap());
        }
    }

    #[test]
    fn test_logic_move_yam() {
        let mut tenv = TestEnv::new().unwrap();
        let (mut logic_move, inst_move) = new_move(&mut tenv);
        let (mut ctx, mut ctxa) = tenv.contexts(false);
        ctxa.input_vars.optimized_device_move.moving = true;
        ctxa.input_vars.optimized_device_move.direction = Vec2::NEG_Y;
        logic_move.start(&mut ctx, &mut ctxa).unwrap();
        logic_move.update(&mut ctx, &mut ctxa).unwrap();

        for ft in FrameTicker::new(0..s2f(0.4)) {
            let (mut ctx, mut ctxa) = tenv.contexts(false);
            ctxa.input_vars.optimized_device_move.moving = true;
            ctxa.input_vars.optimized_device_move.direction = Vec2::NEG_X;

            let ret = logic_move.update(&mut ctx, &mut ctxa).unwrap();
            assert!(logic_move.is_activing());
            assert_eq!(logic_move.mode, ActionMoveMode::Move);
            assert_eq!(logic_move.current_time, ft.time(1));

            assert_eq!(ret.state.fade_in_weight, 1.0);
            assert_eq!(ret.state.animations[0].animation_id, ANIME_MOVE_ID);
            assert_eq!(ret.state.animations[0].files, RUN_OZZ);
            assert_eq!(ret.state.animations[0].ratio, ft.time(1) / inst_move.anim_move.duration);
            assert_eq!(ret.state.animations[0].weight, 1.0);

            let rot = ft.or_last(
                Quat::from_axis_angle(Vec3::Y, -logic_move.yam_ang_vel * ft.time(1)),
                Quat::from_rotation_y(-FRAC_PI_2),
            );
            assert_ulps_eq!(ret.new_direction.unwrap(), (rot * Vec3::Z).xz(),);
            assert_ulps_eq!(ret.new_velocity.unwrap(), rot * Vec3A::new(0.0, 0.0, 5.0));
            tenv.chara_physics.set_direction(ret.new_direction.unwrap());
        }
    }

    #[test]
    fn test_logic_move_turn() {}

    #[test]
    fn test_logic_move_stop() {
        let mut tenv = TestEnv::new().unwrap();
        let mut logic_move = new_move(&mut tenv).0;
        let (mut ctx, mut ctxa) = tenv.contexts(false);
        logic_move.start(&mut ctx, &mut ctxa).unwrap();

        for ft in FrameTicker::new(0..10) {
            let (mut ctx, mut ctxa) = tenv.contexts(false);
            if !ft.last {
                ctxa.input_vars.optimized_device_move.moving = true;
                ctxa.input_vars.optimized_device_move.direction = Vec2::NEG_X;
            } else {
                ctxa.input_vars.optimized_device_move.moving = false;
                ctxa.input_vars.optimized_device_move.direction = Vec2::ZERO;
            }

            let ret = logic_move.update(&mut ctx, &mut ctxa).unwrap();
            if !ft.last {
                assert!(logic_move.is_activing());
            } else {
                assert!(logic_move.is_stopping());
            }
            assert!(ret.derive_keeping.is_none());
        }
    }
}
