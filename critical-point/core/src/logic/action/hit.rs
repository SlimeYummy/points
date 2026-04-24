use critical_point_csgen::{CsEnum, CsOut};
use glam::{Vec2, Vec3Swizzles};
use glam_ext::Vec2xz;
use std::fmt::Debug;
use std::rc::Rc;

use crate::consts::DEFAULT_TOWARD_DIR_2D;
use crate::instance::InstActionHit;
use crate::logic::action::base::{
    ActionStartArgs, ActionStartReturn, ActionUpdateReturn, ContextAction, LogicActionAny, LogicActionBase,
    StateActionAnimation, StateActionAny, StateActionBase, impl_state_action,
};
use crate::logic::action::root_motion::{LogicMultiRootMotion, StateMultiRootMotion};
use crate::logic::game::ContextUpdate;
use crate::utils::{ActionType, Castable, XResult, extend, loose_ge, ratio_warpping, xresf};

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
pub enum ActionHitMode {
    BeHit,
    Down,
    Recovery,
}

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateActionHit {
    pub _base: StateActionBase,
    pub mode: ActionHitMode,
    pub be_hit_index0: u32,
    pub be_hit_index1: u32,
    pub be_hit_ratio: f32,
    pub be_hit_angle_diff: Vec2xz,
    pub current_time: f32,

    pub root_motion: StateMultiRootMotion,
}

extend!(StateActionHit, StateActionBase);
impl_state_action!(StateActionHit, Hit, "Hit");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionHit {
    _base: LogicActionBase,
    inst: Rc<InstActionHit>,

    mode: ActionHitMode,
    be_hit_index0: u32,
    be_hit_index1: u32,
    be_hit_ratio: f32,
    be_hit_angle_diff: Vec2xz,
    current_time: f32,

    root_motion: LogicMultiRootMotion,
}

extend!(LogicActionHit, LogicActionBase);

impl LogicActionHit {
    pub fn new(ctx: &mut ContextUpdate, inst_act: Rc<InstActionHit>) -> XResult<LogicActionHit> {
        let root_motion =
            LogicMultiRootMotion::new_with_capacity(ctx, inst_act.animations(), inst_act.animations_count())?;

        Ok(LogicActionHit {
            _base: LogicActionBase {
                derive_level: inst_act.derive_level,
                poise_level: u16::MAX,
                ..LogicActionBase::new(ctx.gene.gen_action_id(), inst_act.clone())
            },
            inst: inst_act,

            mode: ActionHitMode::BeHit,
            be_hit_index0: u32::MAX,
            be_hit_index1: u32::MAX,
            be_hit_ratio: 1.0,
            be_hit_angle_diff: Vec2xz::from_angle(0.0),
            current_time: 0.0,

            root_motion,
        })
    }
}

unsafe impl LogicActionAny for LogicActionHit {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::Hit
    }

    fn restore(&mut self, state: &(dyn StateActionAny + 'static)) -> XResult<()> {
        if state.id != self._base.id {
            return xresf!(LogicIDMismatch; "state.id={}, self.id={}", state.id, self._base.id);
        }
        let state = state.cast::<StateActionHit>()?;

        self._base.restore(&state._base);
        self.mode = state.mode;
        self.be_hit_index0 = state.be_hit_index0;
        self.be_hit_index1 = state.be_hit_index1;
        self.be_hit_ratio = state.be_hit_ratio;
        self.be_hit_angle_diff = state.be_hit_angle_diff;
        self.current_time = state.current_time;

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

        let hit_dir = args.dir.unwrap_or(-DEFAULT_TOWARD_DIR_2D);
        let chara_dir = ctxa.chara_phy.direction();
        let angle = chara_dir.angle_to(hit_dir);

        let inst = self.inst.clone();
        let res = inst.find_be_hit_by_angle(angle);
        self.be_hit_index0 = res.index0;
        self.be_hit_index1 = res.index1;
        self.be_hit_ratio = res.ratio;
        self.be_hit_angle_diff = Vec2xz::from_angle(res.angle_diff);

        self.mode = ActionHitMode::BeHit;
        self.current_time = 0.0;

        let be_hit = &inst.be_hits[self.be_hit_index0 as usize];
        self.root_motion.set_local_id(be_hit.anim.local_id, 0.0)?;
        Ok(ActionStartReturn::new())
    }

    fn update(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<ActionUpdateReturn> {
        self._base.update(ctx, ctxa)?;
        self.current_time += ctxa.time_step;

        let mut ret = ActionUpdateReturn::new();
        let mut stop = false;

        match self.mode {
            ActionHitMode::BeHit => {
                let be_hit = &self.inst.be_hits[self.be_hit_index0 as usize];

                self.root_motion
                    .update(be_hit.anim.ratio_saturating(self.current_time))?;
                let vel = self.root_motion.position_delta().xz() * ctxa.frac_1_time_step;
                let vel_xz = Vec2xz::from(vel).rotate(self.be_hit_angle_diff);
                ret.set_velocity_2d(vel_xz);

                if loose_ge!(self.current_time, be_hit.anim.duration) {
                    // if self.inst.anim_down.is_some() {
                    //     self.mode = ActionHitMode::Down;
                    // }
                    // else if self.inst.anim_recovery.is_some() {
                    //     self.mode = ActionHitMode::Recovery;
                    // }
                    // else {
                    //     stop = true;
                    // }
                    stop = true;
                }
            }
            ActionHitMode::Down => {
                if loose_ge!(self.current_time, self.inst.max_down_time) {
                    if self.inst.anim_recovery.is_some() {
                        self.mode = ActionHitMode::Recovery;
                    }
                    else {
                        stop = true;
                    }
                }
            }
            ActionHitMode::Recovery => {
                if let Some(anim_recovery) = &self.inst.anim_recovery {
                    if loose_ge!(self.current_time, anim_recovery.duration) {
                        stop = true;
                    }
                }
                else {
                    debug_assert!(false, "Unexpected empty anim_recovery");
                    stop = true;
                }
            }
        }

        if stop {
            self.stop(ctx, ctxa)?;
        }

        // Update fade in time
        if self.fade_in_weight < 1.0 {
            if self.mode == ActionHitMode::BeHit {
                let be_hit = &self.inst.be_hits[self.be_hit_index0 as usize];
                self.fade_in_weight = be_hit.anim.fade_in_weight(self.fade_in_weight, ctxa.time_step);
            }
            else {
                self.fade_in_weight = 1.0
            }
        }

        Ok(ret)
    }

    fn save(&self) -> Box<dyn StateActionAny> {
        let mut state = Box::new(StateActionHit {
            _base: self._base.save(self.typ()),
            mode: self.mode,
            be_hit_index0: self.be_hit_index0,
            be_hit_index1: self.be_hit_index1,
            be_hit_ratio: self.be_hit_ratio,
            be_hit_angle_diff: self.be_hit_angle_diff,
            current_time: self.current_time,
            root_motion: self.root_motion.save(),
        });

        match self.mode {
            ActionHitMode::BeHit => {
                let be_hit0 = &self.inst.be_hits[self.be_hit_index0 as usize];
                let ratio = ratio_warpping(self.current_time, be_hit0.anim.duration);
                state.animations.push(StateActionAnimation::new_with_anim(
                    &be_hit0.anim,
                    ratio,
                    self.be_hit_ratio,
                ));
                if self.be_hit_index1 != u32::MAX {
                    let be_hit1 = &self.inst.be_hits[self.be_hit_index1 as usize];
                    state.animations.push(StateActionAnimation::new_with_anim(
                        &be_hit1.anim,
                        ratio,
                        1.0 - self.be_hit_ratio,
                    ));
                }
            }
            ActionHitMode::Down => {
                if let Some(anim) = &self.inst.anim_down {
                    let ratio = ratio_warpping(self.current_time, anim.duration);
                    state
                        .animations
                        .push(StateActionAnimation::new_with_anim(anim, ratio, 1.0));
                }
            }
            ActionHitMode::Recovery => {
                if let Some(anim) = &self.inst.anim_recovery {
                    let ratio = ratio_warpping(self.current_time, anim.duration);
                    state
                        .animations
                        .push(StateActionAnimation::new_with_anim(anim, ratio, 1.0));
                }
            }
        }

        state.fade_in_weight = self.fade_in_weight;
        state
    }
}
