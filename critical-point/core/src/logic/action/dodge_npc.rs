use critical_point_macros::csharp_out;
use glam::Vec3A;
use glam_ext::Vec2xz;
use std::rc::Rc;

use crate::animation::RootTrackName;
use crate::instance::{InstActionDodgeNpc, InstActionDodgeNpcDodge};
use crate::logic::action::base::{
    ActionStartArgs, ActionStartReturn, ActionUpdateReturn, ContextAction, LogicActionAny, LogicActionBase,
    StateActionAnimation, StateActionAny, StateActionBase, impl_state_action,
};
use crate::logic::action::root_motion::{LogicMultiRootMotion, StateMultiRootMotion};
use crate::logic::game::ContextUpdateEx;
use crate::ok_or;
use crate::utils::{
    ActionType, Castable, LEVEL_IDLE, RotationReference, TimeRange, XResult, ease_in_out_quad, extend, lerp, lerp_with,
    loose_ge, loose_le, quat_from_default_toward_rot_y, quat_from_default_toward_xz, ratio_warpping, xresf,
};

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct StateActionDodgeNpc {
    pub _base: StateActionBase,
    pub current_time: f32,
    pub dodge_index0: u32,
    pub dodge_angle_diff: f32,
    pub move_scale: f32,
    pub from_rotation: f32,
    pub to_rotation: f32,
    pub current_rotation: f32,
    pub rotation_time: TimeRange,
    pub root_motion: StateMultiRootMotion,
}

extend!(StateActionDodgeNpc, StateActionBase);
impl_state_action!(StateActionDodgeNpc, DodgeNpc, "DodgeNpc");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionDodgeNpc {
    _base: LogicActionBase,
    inst: Rc<InstActionDodgeNpc>,
    current_time: f32,
    dodge_index0: u32,
    dodge_angle_diff: f32,
    move_scale: f32,
    from_rotation: f32,
    to_rotation: f32,
    current_rotation: f32,
    rotation_time: TimeRange,
    root_motion: LogicMultiRootMotion,
}

extend!(LogicActionDodgeNpc, LogicActionBase);

impl LogicActionDodgeNpc {
    pub fn new(ctx: &mut ContextUpdateEx, inst_act: Rc<InstActionDodgeNpc>) -> XResult<LogicActionDodgeNpc> {
        let root_motion = LogicMultiRootMotion::new(ctx, inst_act.anim_dodges.iter().map(|d| &d.anim))?;
        Ok(LogicActionDodgeNpc {
            _base: LogicActionBase {
                keep_level: LEVEL_IDLE,
                ..LogicActionBase::new(ctx.identity.gen_action_id(), inst_act.clone())
            },
            inst: inst_act,
            current_time: 0.0,
            dodge_index0: u32::MAX,
            dodge_angle_diff: 0.0,
            move_scale: 1.0,
            from_rotation: 0.0,
            to_rotation: 0.0,
            current_rotation: 0.0,
            rotation_time: TimeRange::EMPTY,
            root_motion,
        })
    }
}

unsafe impl LogicActionAny for LogicActionDodgeNpc {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::DodgeNpc
    }

    fn restore(&mut self, state: &(dyn StateActionAny + 'static)) -> XResult<()> {
        if state.id != self._base.id {
            return xresf!(LogicIDMismatch; "state.id={}, self.id={}", state.id, self._base.id);
        }
        let state = state.cast::<StateActionDodgeNpc>()?;

        self._base.restore(&state._base);
        self.current_time = state.current_time;
        self.dodge_index0 = state.dodge_index0;
        self.dodge_angle_diff = state.dodge_angle_diff;
        self.move_scale = state.move_scale;
        self.from_rotation = state.from_rotation;
        self.to_rotation = state.to_rotation;
        self.current_rotation = state.current_rotation;
        self.rotation_time = state.rotation_time;
        self.root_motion.restore(&state.root_motion);
        Ok(())
    }

    fn start(
        &mut self,
        ctx: &mut ContextUpdateEx,
        ctxa: &mut ContextAction,
        args: &ActionStartArgs,
    ) -> XResult<ActionStartReturn> {
        self._base.start(ctx, ctxa, args)?;

        self.current_time = 0.0;
        self.current_rotation = ctxa.chara_phy.direction_xz().to_angle();
        self.from_rotation = 0.0;
        self.to_rotation = 0.0;
        self.rotation_time = TimeRange::EMPTY;

        // Determine dodge direction and target distance
        let (dodge_angle, target_distance) = if let Some(ai_thinking) = ctxa.ai_thinking {
            let move_dir = ai_thinking.move_dir;
            debug_assert!(move_dir.length_squared() > 1e-4, "move_dir={:?}", move_dir);
            let chara_dir = ctxa.chara_phy.direction_xz();
            let angle = chara_dir.angle_to(move_dir);

            let dst_pos = Vec2xz::from_vec3a(ai_thinking.move_dst_pos);
            let dist = ctxa.chara_phy.position_xz().distance(dst_pos);
            // println!(
            //     "move_dir={:?} chara_dir={:?} dodge_angle={:?}, target_distance={:?}",
            //     move_dir, chara_dir, angle, dist
            // );
            (angle, Some(dist))
        }
        else {
            (std::f32::consts::PI, None)
        };

        // Find best matching dodge animation
        let inst = self.inst.clone();
        let res = inst.find_dodge_by_angle(dodge_angle);
        self.dodge_index0 = res.index0;
        self.dodge_angle_diff = res.angle_diff;
        // println!(
        //     "dodge_index0={}, res.angle_diff={}, dodge_angle_diff={:?}",
        //     self.dodge_index0, res.angle_diff, self.dodge_angle_diff
        // );

        // Setup root motion and distance scaling
        self.root_motion.set_local_id(self.dodge_index0 as u16, 0.0)?;
        let whole_pos = self
            .root_motion
            .track(self.dodge_index0 as u16)
            .whole_position(RootTrackName::Default);
        let default_dist = Vec2xz::new(whole_pos.x, whole_pos.z).length();
        match target_distance {
            Some(dist) if dist > 1e-4 => {
                let norm_dist = dist.clamp(self.inst.move_distance.min, self.inst.move_distance.max);
                self.move_scale = norm_dist / default_dist;
            }
            _ => self.move_scale = 1.0,
        }

        Ok(ActionStartReturn::new())
    }

    fn update(&mut self, ctx: &mut ContextUpdateEx, ctxa: &mut ContextAction) -> XResult<ActionUpdateReturn> {
        self._base.update(ctx, ctxa)?;

        let inst = self.inst.clone();
        let dodge = &inst.anim_dodges[self.dodge_index0 as usize];

        let prev_time = self.current_time;
        self.current_time = (self.current_time + ctxa.time_step).clamp(0.0, dodge.anim.duration);
        self.keep_level = *dodge.keep_levels.find_value(0.0).unwrap_or(&LEVEL_IDLE);
        self.poise_level = 0; // TODO: implement poise level attributes;

        if self.fade_in_weight < 1.0 {
            self.fade_in_weight = dodge.anim.fade_in_weight(self.fade_in_weight, ctxa.time_step);
        }

        self.try_trigger_rotation(ctxa, dodge, prev_time);
        self.update_rotation();

        self.root_motion
            .update(dodge.anim.ratio_saturating(self.current_time))?;

        let curr_dir = Vec2xz::from_angle(self.current_rotation);
        let mut ret = ActionUpdateReturn::new(curr_dir);

        let vel = self.root_motion.position_delta() * ctxa.frac_1_time_step;
        let rot = quat_from_default_toward_rot_y(self.current_rotation);
        let rotated_vel = rot * vel;
        let scaled_vel = rotated_vel * Vec3A::new(self.move_scale, 1.0, self.move_scale);
        ret.set_velocity(scaled_vel);

        ret.new_gravity = loose_le!(scaled_vel.y, 0.0, 1e-3);

        if loose_ge!(self.current_time, dodge.anim.duration) {
            self.stop(ctx, ctxa)?;
        }

        // println!("current_rotation={} vel={:?} pos={:?}", self.current_rotation, ret.new_velocity, ctxa.chara_phy.position());
        Ok(ret)
    }

    fn save(&self) -> Box<dyn StateActionAny> {
        let dodge0 = &self.inst.anim_dodges[self.dodge_index0 as usize];
        let ratio = ratio_warpping(self.current_time, dodge0.anim.duration);
        let mut state = Box::new(StateActionDodgeNpc {
            _base: self._base.save(self.typ()),
            current_time: self.current_time,
            dodge_index0: self.dodge_index0,
            dodge_angle_diff: self.dodge_angle_diff,
            from_rotation: self.from_rotation,
            to_rotation: self.to_rotation,
            current_rotation: self.current_rotation,
            rotation_time: self.rotation_time,
            move_scale: self.move_scale,
            root_motion: self.root_motion.save(),
        });
        state
            .animations
            .push(StateActionAnimation::new_with_anim(&dodge0.anim, ratio, 1.0));
        state.fade_in_weight = self.fade_in_weight;
        state
    }
}

impl LogicActionDodgeNpc {
    fn try_trigger_rotation(&mut self, ctxa: &ContextAction, dodge: &InstActionDodgeNpcDodge, prev_time: f32) {
        let time_range = TimeRange::new(prev_time, self.current_time);
        if !time_range.contains_lc(dodge.rotation_start) {
            return;
        }

        let ai_thinking = ok_or!(ctxa.ai_thinking; return);
        let mut target_dir = None;
        match dodge.rotation_reference {
            RotationReference::Character => target_dir = Some(ai_thinking.move_dir),
            RotationReference::TargetCharacter => {
                if let Some(tgt_pos) = ai_thinking.target_chara_pos() {
                    let dir = Vec2xz::from_vec3a(tgt_pos) - ctxa.chara_phy.position_xz();
                    if dir.length_squared() > 1e-4 {
                        target_dir = Some(dir.normalize());
                    }
                }
            }
            _ => {}
        };
        let target_dir = ok_or!(target_dir; return);

        let chara_dir = ctxa.chara_phy.direction_xz();
        if loose_ge!(chara_dir.dot(target_dir) - 1.0, 1e-4) {
            return; // Already in same direction, no need to rotate
        }

        let diff = chara_dir.angle_to(target_dir);
        let max_angle = dodge.rotation_max_angle;
        let clamped_diff = match diff.abs() <= max_angle {
            true => diff,
            false => diff.signum() * max_angle,
        };

        self.from_rotation = chara_dir.to_angle();
        self.to_rotation = self.from_rotation + clamped_diff;

        let t = clamped_diff.abs() / max_angle;
        let duration = lerp(dodge.rotation_duration.min, dodge.rotation_duration.max, t);
        self.rotation_time = TimeRange::new(self.current_time, self.current_time + duration);
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
}
