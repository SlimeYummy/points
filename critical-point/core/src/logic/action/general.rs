use approx::abs_diff_ne;
use critical_point_csgen::CsOut;
use glam::{Quat, Vec3A, Vec3Swizzles};
use std::fmt::Debug;
use std::rc::Rc;

use crate::instance::InstActionGeneral;
use crate::logic::action::base::{
    impl_state_action, ActionStartReturn, ActionUpdateReturn, ContextAction, LogicActionAny, LogicActionBase,
    StateActionAnimation, StateActionAny, StateActionBase, StateActionType,
};
use crate::logic::action::root_motion::{LogicRootMotion, StateRootMotion};
use crate::logic::action::DeriveKeeping;
use crate::logic::game::ContextUpdate;
use crate::ratio_saturating;
use crate::template::TmplType;
use crate::utils::{dir_xz_from_quat, extend, strict_lt, xresf, Castable, XResult, LEVEL_IDLE};

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateActionGeneral {
    pub _base: StateActionBase,
    pub root_motion: StateRootMotion,
    pub current_time: f32,
}

extend!(StateActionGeneral, StateActionBase);
impl_state_action!(StateActionGeneral, ActionGeneral, General, "General");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionGeneral {
    _base: LogicActionBase,
    inst: Rc<InstActionGeneral>,
    root_motion: LogicRootMotion,
    current_time: f32,
    start_rotation: Quat,
    distance_ratio: f32,
}

extend!(LogicActionGeneral, LogicActionBase);

impl LogicActionGeneral {
    pub fn new(ctx: &mut ContextUpdate<'_>, inst_act: Rc<InstActionGeneral>) -> XResult<LogicActionGeneral> {
        Ok(LogicActionGeneral {
            _base: LogicActionBase {
                derive_level: *inst_act.derive_levels.value_by_time(0.0).unwrap_or(&LEVEL_IDLE),
                poise_level: match inst_act.attributes.value_by_time(0.0) {
                    Some(v) => v.poise_level,
                    None => 0,
                },
                ..LogicActionBase::new(ctx.gene.gen_id(), inst_act.clone())
            },
            inst: inst_act.clone(),
            root_motion: LogicRootMotion::new(ctx, &inst_act.anim_main, 0.0)?,
            current_time: 0.0,
            start_rotation: Quat::IDENTITY,
            distance_ratio: 1.0,
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
        self.root_motion.restore(&state.root_motion);
        self.current_time = state.current_time;
        Ok(())
    }

    fn start(&mut self, ctx: &mut ContextUpdate<'_>, ctxa: &mut ContextAction<'_, '_>) -> XResult<ActionStartReturn> {
        self._base.start(ctx, ctxa)?;
        self.current_time = 0.0;
        self.start_rotation = ctxa.chara_physics.rotation_y();

        self.fade_in_weight = self.inst.anim_main.fade_in_weight(self.fade_in_weight, ctxa.time_step);

        let distance_xz = self.root_motion.track().whole_position().xz().length();
        if ctxa.input_vars.optimized_world_move().moving {
            self.distance_ratio = self.inst.motion_distance[1] / distance_xz;
        }
        else {
            self.distance_ratio = self.inst.motion_distance[0] / distance_xz;
        }
        Ok(ActionStartReturn::new())
    }

    fn update(&mut self, ctx: &mut ContextUpdate<'_>, ctxa: &mut ContextAction<'_, '_>) -> XResult<ActionUpdateReturn> {
        self._base.update(ctx, ctxa)?;

        self.current_time = (self.current_time + ctxa.time_step).clamp(0.0, self.inst.anim_main.duration);
        self.derive_level = *self
            .inst
            .derive_levels
            .value_by_time(self.current_time)
            .unwrap_or(&LEVEL_IDLE);
        self.poise_level = match self.inst.attributes.value_by_time(self.current_time) {
            Some(v) => v.poise_level,
            None => 0,
        };

        if self.fade_in_weight < 1.0 {
            self.fade_in_weight = self.inst.anim_main.fade_in_weight(self.fade_in_weight, ctxa.time_step);
        }

        self.root_motion
            .update(ratio_saturating!(self.current_time, self.inst.anim_main.duration))?;
        let rotation = self.start_rotation * self.root_motion.rotation();
        let direction = dir_xz_from_quat(rotation);

        let delta_pos = self.root_motion.position_delta();
        let mut velocity =
            self.start_rotation * (Vec3A::new(delta_pos.x, 0.0, delta_pos.z) * self.distance_ratio / ctxa.time_step);
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
        Ok(ret)
    }

    fn save(&self) -> Box<dyn StateActionAny> {
        let mut state = Box::new(StateActionGeneral {
            _base: self._base.save(self.typ(), self.tmpl_typ()),
            current_time: self.current_time,
            root_motion: self.root_motion.save(),
        });
        state.fade_in_weight = self.fade_in_weight;

        state.animations[0] = StateActionAnimation {
            animation_id: self.inst.anim_main.local_id,
            files: self.inst.anim_main.files.clone(),
            ratio: self.inst.anim_main.ratio_saturating(self.current_time),
            weight: 1.0,
        };
        state
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
        });
        raw_state.id = 123;
        raw_state.tmpl_id = id!("Action.Instance.Attack/1A");
        raw_state.status = LogicActionStatus::Activing;
        raw_state.first_frame = 15;
        raw_state.last_frame = 99;
        raw_state.derive_level = 1;
        raw_state.poise_level = 2;
        raw_state.animations[0] = StateActionAnimation::new(sb!("idle.ozz"), 1, 0.5, 0.5);

        let state = test_state_action_rkyv(raw_state, StateActionType::General, TmplType::ActionGeneral).unwrap();
        let state = state.cast::<StateActionGeneral>().unwrap();

        assert_eq!(state.id, 123);
        assert_eq!(state.tmpl_id, id!("Action.Instance.Attack/1A"));
        assert_eq!(state.status, LogicActionStatus::Activing);
        assert_eq!(state.first_frame, 15);
        assert_eq!(state.last_frame, 99);
        assert_eq!(state.derive_level, 1);
        assert_eq!(state.poise_level, 2);
        assert_eq!(
            state.animations[0],
            StateActionAnimation::new(sb!("idle.ozz"), 1, 0.5, 0.5)
        );
        assert_eq!(state.animations[1], StateActionAnimation::default());
        assert_eq!(state.animations[2], StateActionAnimation::default());
        assert_eq!(state.animations[3], StateActionAnimation::default());
        assert_eq!(state.current_time, 4.0);
    }

    fn new_general(tenv: &mut TestEnv) -> (LogicActionGeneral, Rc<InstActionGeneral>) {
        let inst_gen: Rc<InstActionGeneral> = tenv
            .inst_player
            .find_action_by_id(id!("Action.Instance.Attack/1A"))
            .unwrap();
        let logic_gen = LogicActionGeneral::new(&mut tenv.context_update(), inst_gen.clone()).unwrap();
        (logic_gen, inst_gen)
    }

    static ATTACK1_OZZ: &str = "girl_attack1_1.*";

    #[test]
    fn test_logic_new() {
        let mut tenv = TestEnv::new().unwrap();
        let logic_gen = new_general(&mut tenv).0;

        assert_eq!(logic_gen.tmpl_id(), id!("Action.Instance.Attack/1A"));
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
            let fade_in_weight = ratio_saturating!(ft.time(2), inst_gen.anim_main.fade_in);
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
