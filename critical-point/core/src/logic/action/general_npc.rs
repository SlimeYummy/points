use core::f32;
use glam::Vec3Swizzles;
use glam_ext::Vec2xz;
use std::fmt::Debug;
use std::rc::Rc;

use crate::instance::{InstActionGeneralNpc, InstActionGeneralNpcMovement};
use crate::logic::action::ActionStartArgs;
use crate::logic::action::base::{
    ActionStartReturn, ActionUpdateReturn, ContextAction, LogicActionAny, LogicActionBase, StateActionAnimation,
    StateActionAny, StateActionBase, impl_state_action,
};
use crate::logic::action::root_motion::{LogicRootMotion, StateRootMotion};
use crate::logic::game::ContextUpdate;
use crate::utils::{
    ActionType, Castable, CustomEvent, F32Range, LEVEL_IDLE, TimeRange, XResult, ease_in_out_quad, ease_in_quad,
    extend, lerp, lerp_trapezoid_with, lerp_with, ok_or, quat_from_dir_xz, strict_lt, xresf,
};

#[repr(C)]
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct StateActionGeneralNpc {
    pub _base: StateActionBase,
    pub current_time: f32,

    pub from_rotation: f32,
    pub to_rotation: f32,
    pub current_rotation: f32,
    pub rotation_time: TimeRange,

    pub translation_speed_ratio: f32,
    pub translation_fade_ratio: f32,
    pub translation_time: TimeRange,

    pub root_motion: StateRootMotion,
}

extend!(StateActionGeneralNpc, StateActionBase);
impl_state_action!(StateActionGeneralNpc, GeneralNpc, "GeneralNpc");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionGeneralNpc {
    _base: LogicActionBase,
    inst: Rc<InstActionGeneralNpc>,
    current_time: f32,

    from_rotation: f32,
    to_rotation: f32,
    current_rotation: f32,
    rotation_time: TimeRange,

    translation_speed_ratio: f32,
    translation_fade_ratio: f32,
    translation_time: TimeRange,

    root_motion: LogicRootMotion,
}

extend!(LogicActionGeneralNpc, LogicActionBase);

impl LogicActionGeneralNpc {
    pub fn new(ctx: &mut ContextUpdate, inst_act: Rc<InstActionGeneralNpc>) -> XResult<LogicActionGeneralNpc> {
        Ok(LogicActionGeneralNpc {
            _base: LogicActionBase {
                keep_level: *inst_act.keep_levels.find_value(0.0).unwrap_or(&LEVEL_IDLE),
                // poise_level: match inst_act.attributes.find_value(0.0) {
                //     Some(v) => v.poise_level,
                //     None => 0,
                // },
                ..LogicActionBase::new(ctx.identity.gen_action_id(), inst_act.clone())
            },
            inst: inst_act.clone(),
            current_time: 0.0,

            from_rotation: 0.0,
            current_rotation: 0.0,
            to_rotation: 0.0,
            rotation_time: TimeRange::EMPTY,

            translation_speed_ratio: 1.0,
            translation_fade_ratio: 0.0,
            translation_time: TimeRange::EMPTY,

            root_motion: LogicRootMotion::new(ctx, &inst_act.anim_main, 0.0)?,
        })
    }
}

unsafe impl LogicActionAny for LogicActionGeneralNpc {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::GeneralNpc
    }

    fn restore(&mut self, state: &(dyn StateActionAny + 'static)) -> XResult<()> {
        if state.id != self._base.id {
            return xresf!(LogicIDMismatch; "state.id={}, self.id={}", state.id, self._base.id);
        }
        let state = state.cast::<StateActionGeneralNpc>()?;

        self._base.restore(&state._base);
        self.current_time = state.current_time;

        self.from_rotation = state.from_rotation;
        self.to_rotation = state.to_rotation;
        self.current_rotation = state.current_rotation;
        self.rotation_time = state.rotation_time;

        self.translation_speed_ratio = state.translation_speed_ratio;
        self.translation_fade_ratio = state.translation_fade_ratio;
        self.translation_time = state.translation_time;

        self.root_motion.restore(&state.root_motion);
        Ok(())
    }

    fn start(
        &mut self,
        ctx: &mut ContextUpdate,
        ctxa: &mut ContextAction,
        args: &ActionStartArgs,
    ) -> XResult<ActionStartReturn> {
        self._base.start(ctx, ctxa, args)?;

        self.current_time = 0.0;

        self.from_rotation = 0.0;
        self.to_rotation = 0.0;
        self.current_rotation = ctxa.chara_phy.direction_xz().to_angle();
        self.rotation_time = TimeRange::EMPTY;

        self.translation_speed_ratio = 1.0;
        self.translation_fade_ratio = 0.0;
        self.translation_time = TimeRange::EMPTY;

        let mut ret = ActionStartReturn::new();
        self.handle_ai_movement(ctx, ctxa, f32::NEG_INFINITY)?;

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
        // self.poise_level = match self.inst.attributes.find_value(self.current_time) {
        //     Some(v) => v.poise_level,
        //     None => 0,
        // };

        if self.fade_in_weight < 1.0 {
            self.fade_in_weight = self.inst.anim_main.fade_in_weight(self.fade_in_weight, ctxa.time_step);
        }

        self.handle_ai_movement(ctx, ctxa, prev_time)?;
        self.update_rotation();

        let direction = Vec2xz::from_angle(self.current_rotation);
        let rotation = quat_from_dir_xz(direction);

        self.root_motion
            .update(self.inst.anim_main.ratio_saturating(self.current_time))?;

        let mut delta_pos = self.root_motion.position_delta();
        let real_speed_ratio = self.update_translation(ctxa);

        delta_pos.x *= real_speed_ratio;
        delta_pos.z *= real_speed_ratio;

        let velocity = rotation * delta_pos * ctxa.frac_1_time_step;

        let mut ret;
        if strict_lt!(self.current_time, self.inst.anim_main.duration) {
            ret = ActionUpdateReturn::new();
        }
        else {
            self.stop(ctx, ctxa)?;
            ret = ActionUpdateReturn::new();
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
        let mut state = Box::new(StateActionGeneralNpc {
            _base: self._base.save(self.typ()),
            current_time: self.current_time,

            from_rotation: self.from_rotation,
            to_rotation: self.to_rotation,
            current_rotation: self.current_rotation,
            rotation_time: self.rotation_time,

            translation_speed_ratio: self.translation_speed_ratio,
            translation_fade_ratio: self.translation_fade_ratio,
            translation_time: self.translation_time,

            root_motion: self.root_motion.save(),
        });

        let ratio = self.inst.anim_main.ratio_saturating(self.current_time);
        state
            .animations
            .push(StateActionAnimation::new_with_anim(&self.inst.anim_main, ratio, 1.0));
        state
    }
}

impl LogicActionGeneralNpc {
    fn handle_ai_movement(&mut self, _ctx: &mut ContextUpdate, ctxa: &ContextAction, prev_time: f32) -> XResult<()> {
        let ai_thinking = match ctxa.ai_thinking {
            Some(ai_thinking) => ai_thinking,
            None => return Ok(()),
        };

        for movements in self
            .inst
            .adjust_movements
            .find_values((prev_time, self.current_time).into())
        {
            match movements {
                InstActionGeneralNpcMovement::Translation(trans) => {
                    if trans.duration == 0.0 || trans.speed_ratio.is_empty() {
                        continue;
                    }

                    let target_chara_pos = ok_or!(ai_thinking.target_chara_pos(); continue);
                    let target_pos = Vec2xz::from_vec3a(target_chara_pos);

                    let dist = ctxa.chara_phy.position_xz().distance(target_pos);
                    if dist <= trans.distance.min {
                        self.translation_speed_ratio = trans.speed_ratio.min;
                    }
                    else if dist >= trans.distance.max {
                        self.translation_speed_ratio = trans.speed_ratio.max;
                    }
                    else {
                        self.translation_speed_ratio = lerp(
                            trans.speed_ratio.min,
                            trans.speed_ratio.max,
                            (dist - trans.distance.min) / (trans.distance.max - trans.distance.min),
                        );
                    }

                    self.translation_fade_ratio = trans.fade_ratio.clamp(0.0, 0.5);
                    self.translation_time = TimeRange::new(self.current_time, self.current_time + trans.duration);
                }
                InstActionGeneralNpcMovement::Rotation(rot) => {
                    let target_chara_pos = ok_or!(ai_thinking.target_chara_pos(); continue);
                    let target_pos = Vec2xz::from_vec3a(target_chara_pos);

                    let mut target_dir = target_pos - ctxa.chara_phy.position_xz();
                    if target_dir.length_squared() <= 1e-4 {
                        continue;
                    }
                    target_dir = target_dir.normalize();

                    self.rotation_time = TimeRange::new(self.current_time, self.current_time + rot.duration);
                    let chara_dir = ctxa.chara_phy.direction_xz();
                    self.from_rotation = chara_dir.to_angle();

                    let (turn_angle, target_dir) = (rot.max_angle.abs(), target_dir);
                    let diff = chara_dir.angle_to(target_dir);
                    if diff.abs() <= turn_angle {
                        self.to_rotation = self.from_rotation + diff;
                    }
                    else {
                        self.to_rotation = self.from_rotation + diff.signum() * turn_angle;
                    }
                }
            }
        }

        Ok(())
    }

    fn update_rotation(&mut self) {
        if self.rotation_time.is_empty() {
            return;
        }

        if self.rotation_time.contains_lc(self.current_time) {
            let t = (self.current_time - self.rotation_time.begin) / self.rotation_time.duration();
            self.current_rotation = lerp_with(self.from_rotation, self.to_rotation, t, ease_in_out_quad);
        }
        else {
            debug_assert!(self.current_time >= self.rotation_time.end);
            self.current_rotation = self.to_rotation;
            self.from_rotation = 0.0;
            self.to_rotation = 0.0;
            self.rotation_time = TimeRange::default();
        }
    }

    // Return real speed ratio
    fn update_translation(&mut self, _ctxa: &ContextAction) -> f32 {
        if self.translation_time.is_empty() {
            return 1.0;
        }

        if self.translation_time.contains_lc(self.current_time) {
            let t = (self.current_time - self.translation_time.begin) / self.translation_time.duration();
            lerp_trapezoid_with(
                1.0,
                self.translation_speed_ratio,
                t,
                ease_in_quad,
                self.translation_fade_ratio,
            )
        }
        else {
            debug_assert!(self.current_time >= self.translation_time.end);
            self.translation_time = TimeRange::EMPTY;
            self.translation_speed_ratio = 1.0;
            1.0
        }
    }
}
