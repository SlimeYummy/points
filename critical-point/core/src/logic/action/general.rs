use approx::abs_diff_ne;
use core::f32;
use critical_point_csgen::CsOut;
use glam::Vec3A;
use glam_ext::Vec2xz;
use std::fmt::Debug;
use std::rc::Rc;

use crate::animation::RootTrackName;
use crate::instance::{InstActionGeneral, InstActionGeneralMovement};
use crate::logic::InputMoveSpeed;
use crate::logic::action::base::{
    impl_state_action, ActionStartReturn, ActionUpdateReturn, ContextAction, LogicActionAny, LogicActionBase,
    StateActionAnimation, StateActionAny, StateActionBase, StateActionType,
};
use crate::logic::action::root_motion::{LogicRootMotion, StateRootMotion};
use crate::logic::action::DeriveKeeping;
use crate::logic::game::ContextUpdate;
use crate::template::TmplType;
use crate::utils::{
    ease_in_out_quad, extend, lerp_with, quat_from_dir_xz, strict_lt, xresf, Castable, CustomEvent, TimeRange, XResult,
    LEVEL_IDLE,
};

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateActionGeneral {
    pub _base: StateActionBase,
    pub current_time: f32,
    pub from_rotation: f32,
    pub to_rotation: f32,
    pub current_rotation: f32,
    pub rotation_time: TimeRange,
    pub root_motion: StateRootMotion,
}

extend!(StateActionGeneral, StateActionBase);
impl_state_action!(StateActionGeneral, ActionGeneral, General, "General");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionGeneral {
    _base: LogicActionBase,
    inst: Rc<InstActionGeneral>,
    current_time: f32,
    from_rotation: f32,
    to_rotation: f32,
    current_rotation: f32,
    rotation_time: TimeRange,
    root_motion: LogicRootMotion,
}

extend!(LogicActionGeneral, LogicActionBase);

impl LogicActionGeneral {
    pub fn new(ctx: &mut ContextUpdate, inst_act: Rc<InstActionGeneral>) -> XResult<LogicActionGeneral> {
        Ok(LogicActionGeneral {
            _base: LogicActionBase {
                derive_level: *inst_act.derive_levels.find_value(0.0).unwrap_or(&LEVEL_IDLE),
                poise_level: match inst_act.attributes.find_value(0.0) {
                    Some(v) => v.poise_level,
                    None => 0,
                },
                ..LogicActionBase::new(ctx.gene.gen_id(), inst_act.clone())
            },
            inst: inst_act.clone(),
            current_time: 0.0,
            from_rotation: 0.0,
            current_rotation: 0.0,
            to_rotation: 0.0,
            rotation_time: TimeRange::default(),
            root_motion: LogicRootMotion::new(ctx, &inst_act.anim_main, 0.0)?,
        })
    }
}

unsafe impl LogicActionAny for LogicActionGeneral {
    #[inline]
    fn typ(&self) -> StateActionType {
        StateActionType::General
    }

    #[inline]
    fn tmpl_typ(&self) -> TmplType {
        TmplType::ActionGeneral
    }

    fn restore(&mut self, state: &(dyn StateActionAny + 'static)) -> XResult<()> {
        if state.id != self._base.id {
            return xresf!(LogicIDMismatch; "state.id={}, self.id={}", state.id, self._base.id);
        }
        let state = state.cast::<StateActionGeneral>()?;

        self._base.restore(&state._base);
        self.current_time = state.current_time;
        self.root_motion.restore(&state.root_motion);
        Ok(())
    }

    fn start(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<ActionStartReturn> {
        self._base.start(ctx, ctxa)?;
        self.current_time = 0.0;
        self.from_rotation = 0.0;
        self.to_rotation = 0.0;
        self.current_rotation = ctxa.chara_physics.direction().to_angle();
        self.rotation_time = TimeRange::default();

        self.fade_in_weight = self.inst.anim_main.fade_in_weight(self.fade_in_weight, ctxa.time_step);

        let mut ret = ActionStartReturn::new();
        if self.handle_input_rotation(ctxa, f32::NEG_INFINITY)? {
            ret.clear_preinput = true;
        }

        ret.custom_events = self
            .inst
            .custom_events
            .find_values((f32::NEG_INFINITY, self.current_time).into())
            .map(|ev| CustomEvent::new(self.inst.tmpl_id, *ev))
            .collect();
        Ok(ret)
    }

    fn update(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<ActionUpdateReturn> {
        self._base.update(ctx, ctxa)?;

        let prev_time = self.current_time;
        self.current_time = (self.current_time + ctxa.time_step).clamp(0.0, self.inst.anim_main.duration);
        self.derive_level = *self
            .inst
            .derive_levels
            .find_value(self.current_time)
            .unwrap_or(&LEVEL_IDLE);
        self.poise_level = match self.inst.attributes.find_value(self.current_time) {
            Some(v) => v.poise_level,
            None => 0,
        };

        if self.fade_in_weight < 1.0 {
            self.fade_in_weight = self.inst.anim_main.fade_in_weight(self.fade_in_weight, ctxa.time_step);
        }

        self.handle_input_rotation(ctxa, prev_time)?;
        self.update_rotation();
        let direction = Vec2xz::from_angle(self.current_rotation);
        let rotation = quat_from_dir_xz(direction);

        self.root_motion
            .update(self.inst.anim_main.ratio_saturating(self.current_time))?;
        let delta_pos = self.root_motion.position_delta();
        let mut velocity = rotation * (Vec3A::new(delta_pos.x, 0.0, delta_pos.z) / ctxa.time_step);
        if abs_diff_ne!(delta_pos.y, 0.0) {
            velocity.y = delta_pos.y / ctxa.time_step;
        }

        let mut ret;
        if strict_lt!(self.current_time, self.inst.anim_main.duration) {
            ret = ActionUpdateReturn::new();
        }
        else {
            self.stop(ctx, ctxa)?;
            ret = ActionUpdateReturn::new();

            if self.inst.derive_levels.end_time() > self.current_time {
                ret.derive_keeping = Some(DeriveKeeping {
                    action_id: self.tmpl_id(),
                    derive_level: *self.inst.derive_levels.end_value().unwrap_or(&LEVEL_IDLE),
                    end_time: ctx.time + (self.inst.derive_levels.end_time() - self.current_time),
                })
            }
        }

        ret.set_velocity(velocity);
        ret.set_direction(direction);
        ret.custom_events = self
            .inst
            .custom_events
            .find_values((prev_time, self.current_time).into())
            .map(|ev| CustomEvent::new(self.inst.tmpl_id, *ev))
            .collect();
        Ok(ret)
    }

    fn save(&self) -> Box<dyn StateActionAny> {
        let mut state = Box::new(StateActionGeneral {
            _base: self._base.save(self.typ(), self.tmpl_typ()),
            current_time: self.current_time,
            current_rotation: self.current_rotation,
            from_rotation: self.from_rotation,
            to_rotation: self.to_rotation,
            rotation_time: self.rotation_time,
            root_motion: self.root_motion.save(),
        });

        let ratio = self.inst.anim_main.ratio_saturating(self.current_time);
        state.animations[0] = StateActionAnimation::new_with_anim(&self.inst.anim_main, ratio, 1.0);
        state
    }
}

impl LogicActionGeneral {
    fn handle_input_rotation(&mut self, ctxa: &ContextAction, prev_time: f32) -> XResult<bool> {
        let world_move = ctxa.input_vars.world_move();
        if !world_move.moving {
            return Ok(false);
        }

        let mut clear_preinput = false;
        for movements in self
            .inst
            .input_movements
            .find_values((prev_time, self.current_time).into())
        {
            match movements {
                InstActionGeneralMovement::RootMotion(rm) => {
                    if rm.mov_ex && world_move.speed == InputMoveSpeed::Fast {
                        self.root_motion.set_position_track(RootTrackName::MoveEx)?;
                        clear_preinput = true;
                        log::info!("root_motion: {:?}", RootTrackName::MoveEx);
                    } else if rm.mov {
                        self.root_motion.set_position_track(RootTrackName::Move)?;
                        clear_preinput = true;
                        log::info!("root_motion: {:?}", RootTrackName::Move);
                    }
                }
                InstActionGeneralMovement::Rotation(rot) => {
                    self.rotation_time = TimeRange::new(self.current_time, self.current_time + rot.duration);
                    let chara_dir = ctxa.chara_physics.direction();
                    self.from_rotation = chara_dir.to_angle();

                    let (turn_angle, input_dir) = match rot.angle >= 0.0 {
                        true => (rot.angle, world_move.direction),
                        false => (-rot.angle, -world_move.direction),
                    };
                    let diff = chara_dir.angle_to(input_dir);
                    if diff.abs() <= turn_angle {
                        self.to_rotation = self.from_rotation + diff;
                    }
                    else {
                        self.to_rotation = self.from_rotation + diff.signum() * turn_angle;
                    }
                    clear_preinput = true;
                    log::info!(
                        "chara_dir: {} input_dir: {} world_move: {:?}",
                        chara_dir,
                        input_dir,
                        world_move.direction
                    );
                    log::info!(
                        "diff: {} turn_angle: {} from: {} to: {}",
                        diff,
                        turn_angle,
                        self.from_rotation,
                        self.to_rotation
                    );
                }
            }
        }

        Ok(clear_preinput)
    }

    fn update_rotation(&mut self) {
        if self.rotation_time.duration() == 0.0 {
            return;
        }

        if self.rotation_time.contains(self.current_time) {
            let t = (self.current_time - self.rotation_time.begin) / self.rotation_time.duration();
            self.current_rotation = lerp_with(self.from_rotation, self.to_rotation, t, ease_in_out_quad);
        }
        else {
            println!(
                "current_time: {} > rotation_time.end: {}",
                self.current_time, self.rotation_time.end
            );
            debug_assert!(self.current_time > self.rotation_time.end);
            self.current_rotation = self.to_rotation;
            self.from_rotation = 0.0;
            self.to_rotation = 0.0;
            self.rotation_time = TimeRange::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;

    use super::*;
    use crate::logic::action::base::LogicActionStatus;
    use crate::logic::action::test_utils::*;
    use crate::utils::tests::FrameTicker;
    use crate::utils::{id, ratio_saturating, s2f, sb, LEVEL_ACTION, LEVEL_ATTACK};

    #[test]
    fn test_state_rkyv() {
        let mut raw_state = Box::new(StateActionGeneral {
            _base: StateActionBase::new(StateActionType::General, TmplType::ActionGeneral),
            current_time: 4.0,
            root_motion: StateRootMotion::default(),
            current_rotation: 0.0,
            from_rotation: 0.5,
            to_rotation: 0.5,
            rotation_time: TimeRange::new(1.0, 2.0),
        });
        raw_state.id = 123;
        raw_state.tmpl_id = id!("Action.Instance.Attack^1A");
        raw_state.status = LogicActionStatus::Activing;
        raw_state.first_frame = 15;
        raw_state.last_frame = 99;
        raw_state.derive_level = 1;
        raw_state.poise_level = 2;
        raw_state.animations[0] = StateActionAnimation::new(sb!("idle.ozz"), 1, true, 0.5, 0.5);

        let state = test_state_action_rkyv(raw_state, StateActionType::General, TmplType::ActionGeneral).unwrap();
        let state = state.cast::<StateActionGeneral>().unwrap();

        assert_eq!(state.id, 123);
        assert_eq!(state.tmpl_id, id!("Action.Instance.Attack^1A"));
        assert_eq!(state.status, LogicActionStatus::Activing);
        assert_eq!(state.first_frame, 15);
        assert_eq!(state.last_frame, 99);
        assert_eq!(state.derive_level, 1);
        assert_eq!(state.poise_level, 2);
        assert_eq!(
            state.animations[0],
            StateActionAnimation::new(sb!("idle.ozz"), 1, true, 0.5, 0.5)
        );
        assert_eq!(state.animations[1], StateActionAnimation::default());
        assert_eq!(state.animations[2], StateActionAnimation::default());
        assert_eq!(state.animations[3], StateActionAnimation::default());
        assert_eq!(state.current_time, 4.0);
        assert_eq!(state.root_motion, StateRootMotion::default());
        assert_eq!(state.current_rotation, 0.0);
        assert_eq!(state.from_rotation, 0.5);
        assert_eq!(state.to_rotation, 0.5);
        assert_eq!(state.rotation_time, TimeRange::new(1.0, 2.0));
    }

    fn new_general(tenv: &mut TestEnv) -> (LogicActionGeneral, Rc<InstActionGeneral>) {
        let inst_gen: Rc<InstActionGeneral> = tenv
            .inst_player
            .find_action_by_id(id!("Action.Instance.Attack^1A"))
            .unwrap();
        let logic_gen = LogicActionGeneral::new(&mut tenv.context_update(), inst_gen.clone()).unwrap();
        (logic_gen, inst_gen)
    }

    static ATTACK1_OZZ: &str = "Girl_Attack_01A.*";

    #[test]
    fn test_logic_new() {
        let mut tenv = TestEnv::new().unwrap();
        let logic_gen = new_general(&mut tenv).0;

        assert_eq!(logic_gen.tmpl_id(), id!("Action.Instance.Attack^1A"));
        assert!(logic_gen.is_starting());
        assert_eq!(logic_gen.first_frame, 0);
        assert_eq!(logic_gen.last_frame, u32::MAX);
        assert_eq!(logic_gen.fade_in_weight, 0.0);
        assert_eq!(logic_gen.current_time, 0.0);
    }

    #[test]
    fn test_logic_general() {
        let mut tenv = TestEnv::new().unwrap();
        let (mut logic_gen, inst_gen) = new_general(&mut tenv);
        let (mut ctx, mut ctxa) = tenv.contexts(true);

        logic_gen.start(&mut ctx, &mut ctxa).unwrap();
        for ft in FrameTicker::new(0..s2f(4.0)) {
            println!("{}", ft.time);
            ctx.time = ft.time;
            let ret = logic_gen.update(&mut ctx, &mut ctxa).unwrap();
            if !ft.last {
                assert!(logic_gen.is_activing());
                assert!(ret.derive_keeping.is_none());
            }
            else {
                assert!(logic_gen.is_stopping());
                assert_eq!(ret.derive_keeping.unwrap().action_id, inst_gen.tmpl_id);
                assert_eq!(ret.derive_keeping.unwrap().derive_level, LEVEL_ATTACK);
                assert_eq!(ret.derive_keeping.unwrap().end_time, 4.5);
            }
            assert_eq!(logic_gen.current_time, ft.time(1));

            let state = logic_gen.save();
            let fade_in_weight = ratio_saturating(ft.time(2), inst_gen.anim_main.fade_in);
            assert_ulps_eq!(state.fade_in_weight, fade_in_weight);
            if ft.time < 2.5 {
                assert_eq!(state.derive_level, LEVEL_ACTION);
            }
            else {
                assert_eq!(state.derive_level, LEVEL_ATTACK);
            }
            assert_eq!(state.poise_level, 1);

            assert_eq!(state.animations[0].animation_id, 0);
            assert_eq!(state.animations[0].files, ATTACK1_OZZ);
            assert_eq!(state.animations[0].ratio, ft.time(1) / inst_gen.anim_main.duration);
            assert_eq!(state.animations[0].weight, 1.0);
            assert!(state.animations[1].is_empty());
        }
    }
}
