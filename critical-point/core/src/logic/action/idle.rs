use critical_point_csgen::{CsEnum, CsOut};
use std::fmt::Debug;
use std::rc::Rc;

use crate::instance::InstActionIdle;
use crate::logic::action::base::{
    ActionStartArgs, ActionStartReturn, ActionUpdateReturn, ContextAction, LogicActionAny, LogicActionBase,
    StateActionAnimation, StateActionAny, StateActionBase, impl_state_action,
};
use crate::logic::game::ContextUpdate;
use crate::utils::{ActionType, Castable, XResult, extend, loose_ge, ratio_saturating, ratio_warpping, xresf};

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
pub enum ActionIdleMode {
    Idle,
    Ready,
    IdleToReady,
    ReadyToIdle,
    // Random,
}

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateActionIdle {
    pub _base: StateActionBase,
    pub mode: ActionIdleMode,
    pub idle_time: f32,
    pub ready_time: f32,
    pub auto_idle_time: f32,
    pub switch_time: f32,
}

extend!(StateActionIdle, StateActionBase);
impl_state_action!(StateActionIdle, Idle, "Idle");

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionIdle {
    _base: LogicActionBase,
    inst: Rc<InstActionIdle>,

    mode: ActionIdleMode,
    idle_time: f32,
    ready_time: f32,
    auto_idle_time: f32,
    switch_time: f32,
}

extend!(LogicActionIdle, LogicActionBase);

impl LogicActionIdle {
    pub fn new(ctx: &mut ContextUpdate, inst_act: Rc<InstActionIdle>) -> XResult<LogicActionIdle> {
        Ok(LogicActionIdle {
            _base: LogicActionBase {
                derive_level: inst_act.derive_level,
                poise_level: inst_act.poise_level,
                ..LogicActionBase::new(ctx.gene.gen_action_id(), inst_act.clone())
            },
            inst: inst_act,

            mode: ActionIdleMode::Idle,
            idle_time: 0.0,
            ready_time: 0.0,
            auto_idle_time: 0.0,
            switch_time: 0.0,
        })
    }
}

unsafe impl LogicActionAny for LogicActionIdle {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::Idle
    }

    fn restore(&mut self, state: &(dyn StateActionAny + 'static)) -> XResult<()> {
        if state.id != self._base.id {
            return xresf!(LogicIDMismatch; "state.id={}, self.id={}", state.id, self._base.id);
        }
        let state = state.cast::<StateActionIdle>()?;

        self._base.restore(&state._base);
        self.mode = state.mode;
        self.idle_time = state.idle_time;
        self.ready_time = state.ready_time;
        self.auto_idle_time = state.auto_idle_time;
        self.switch_time = state.switch_time;
        Ok(())
    }

    fn start(
        &mut self,
        ctx: &mut ContextUpdate,
        ctxa: &mut ContextAction,
        args: &ActionStartArgs,
    ) -> XResult<ActionStartReturn> {
        self._base.start(ctx, ctxa, args)?;

        if self.inst.anim_ready.is_some() && !ctxa.chara_phy.is_idle() {
            // Starts in ready state
            self.mode = ActionIdleMode::Ready;
            self.ready_time = 0.0;
        }
        else {
            // Starts in idle state
            self.mode = ActionIdleMode::Idle;
            self.idle_time = 0.0;
        }
        Ok(ActionStartReturn::new())
    }

    fn update(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<ActionUpdateReturn> {
        self._base.update(ctx, ctxa)?;

        let anim_idle = &self.inst.anim_idle;
        let anim_ready = self.inst.anim_ready.as_ref().unwrap_or(&anim_idle);
        let has_ready = self.inst.anim_ready.is_some();

        // Update mode
        let is_idle = ctxa.chara_phy.is_idle();
        match self.mode {
            ActionIdleMode::Idle => {
                if has_ready && !is_idle {
                    self.mode = ActionIdleMode::IdleToReady;
                    self.ready_time = 0.0;
                    self.switch_time = ctxa.time_step;
                }
                else {
                    self.idle_time += ctxa.time_step;
                }
            }
            ActionIdleMode::Ready => {
                match is_idle {
                    true => self.auto_idle_time += ctxa.time_step,
                    false => self.auto_idle_time = 0.0,
                };
                if self.auto_idle_time > self.inst.auto_idle_delay {
                    // TODO: Optimized the animation order in the state
                    self.mode = ActionIdleMode::ReadyToIdle;
                    self.idle_time = 0.0;
                    self.switch_time = ctxa.time_step;
                    self.auto_idle_time = 0.0;
                }
                else {
                    self.ready_time += ctxa.time_step;
                }
            }
            ActionIdleMode::IdleToReady => {
                self.switch_time += ctxa.time_step;
                self.ready_time += ctxa.time_step;
                if loose_ge!(self.switch_time, anim_ready.fade_in) {
                    self.mode = ActionIdleMode::Ready;
                    self.idle_time = 0.0;
                    self.switch_time = 0.0;
                }
            }
            ActionIdleMode::ReadyToIdle => {
                if !is_idle {
                    self.mode = ActionIdleMode::IdleToReady;
                    let progress = ratio_saturating(self.switch_time, anim_idle.fade_in);
                    self.switch_time = (1.0 - progress) * anim_ready.fade_in + ctxa.time_step;
                }
                else {
                    self.switch_time += ctxa.time_step;
                    self.idle_time += ctxa.time_step;
                    if loose_ge!(self.switch_time, anim_idle.fade_in) {
                        self.mode = ActionIdleMode::Idle;
                        self.ready_time = 0.0;
                        self.switch_time = 0.0;
                    }
                }
            }
        };

        // Update fade in time
        if self.fade_in_weight < 1.0 {
            match self.mode {
                ActionIdleMode::Idle | ActionIdleMode::IdleToReady => {
                    self.fade_in_weight = anim_idle.fade_in_weight(self.fade_in_weight, ctxa.time_step);
                }
                ActionIdleMode::Ready | ActionIdleMode::ReadyToIdle => {
                    self.fade_in_weight = anim_ready.fade_in_weight(self.fade_in_weight, ctxa.time_step);
                }
            }
        }

        Ok(ActionUpdateReturn::new())
    }

    fn save(&self) -> Box<dyn StateActionAny> {
        let anim_idle = &self.inst.anim_idle;
        let anim_ready = self.inst.anim_ready.as_ref().unwrap_or(&anim_idle);

        let mut state = Box::new(StateActionIdle {
            _base: self._base.save(self.typ()),
            mode: self.mode,
            idle_time: self.idle_time,
            ready_time: self.ready_time,
            auto_idle_time: self.auto_idle_time,
            switch_time: self.switch_time,
        });

        match self.mode {
            ActionIdleMode::Idle => {
                let ratio = ratio_warpping(self.idle_time, anim_idle.duration);
                state
                    .animations
                    .push(StateActionAnimation::new_with_anim(&anim_idle, ratio, 1.0));
            }
            ActionIdleMode::Ready => {
                let ratio = ratio_warpping(self.ready_time, anim_ready.duration);
                state
                    .animations
                    .push(StateActionAnimation::new_with_anim(&anim_ready, ratio, 1.0));
            }
            ActionIdleMode::IdleToReady => {
                let switch_weight = ratio_saturating(self.switch_time, anim_ready.fade_in);
                let idle_ratio = ratio_warpping(self.idle_time, anim_idle.duration);
                state.animations.push(StateActionAnimation::new_with_anim(
                    &anim_idle,
                    idle_ratio,
                    1.0 - switch_weight,
                ));
                let ready_ratio = ratio_warpping(self.ready_time, anim_ready.duration);
                state.animations.push(StateActionAnimation::new_with_anim(
                    &anim_ready,
                    ready_ratio,
                    switch_weight,
                ));
            }
            ActionIdleMode::ReadyToIdle => {
                let switch_weight = ratio_saturating(self.switch_time, anim_idle.fade_in);
                let ready_ratio = ratio_warpping(self.ready_time, anim_ready.duration);
                state.animations.push(StateActionAnimation::new_with_anim(
                    &anim_ready,
                    ready_ratio,
                    1.0 - switch_weight,
                ));
                let idle_ratio = ratio_warpping(self.idle_time, anim_idle.duration);
                state.animations.push(StateActionAnimation::new_with_anim(
                    &anim_idle,
                    idle_ratio,
                    switch_weight,
                ));
            }
        }

        state.fade_in_weight = self.fade_in_weight;
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::{FPS, SPF};
    use crate::logic::action::base::LogicActionStatus;
    use crate::logic::action::test_utils::*;
    use crate::utils::tests::FrameTicker;
    use crate::utils::{id, s2f, sb};
    use approx::assert_ulps_eq;

    const ANIME_IDLE_ID: u16 = 0;
    const ANIME_READY_ID: u16 = 1;

    #[test]
    fn test_state_rkyv() {
        let mut raw_state = Box::new(StateActionIdle {
            _base: StateActionBase::new(ActionType::Idle),
            mode: ActionIdleMode::IdleToReady,
            idle_time: 10.0,
            ready_time: 20.0,
            auto_idle_time: 0.72,
            switch_time: 5.0,
        });
        raw_state.id = 123;
        raw_state.tmpl_id = id!("Action.Idle");
        raw_state.status = LogicActionStatus::Running;
        raw_state.first_frame = 15;
        raw_state.last_frame = 99;
        raw_state.derive_level = 1;
        raw_state.poise_level = 2;
        raw_state
            .animations
            .push(StateActionAnimation::new(sb!("idle.ozz"), 1, true, false, 0.5, 0.5));

        let state = test_state_action_rkyv(raw_state, ActionType::Idle).unwrap();
        let state = state.cast::<StateActionIdle>().unwrap();

        assert_eq!(state.id, 123);
        assert_eq!(state.tmpl_id, id!("Action.Idle"));
        assert_eq!(state.status, LogicActionStatus::Running);
        assert_eq!(state.first_frame, 15);
        assert_eq!(state.last_frame, 99);
        assert_eq!(state.derive_level, 1);
        assert_eq!(state.poise_level, 2);
        assert_eq!(state.animations.len(), 1);
        assert_eq!(
            state.animations[0],
            StateActionAnimation::new(sb!("idle.ozz"), 1, true, false, 0.5, 0.5)
        );
        assert_eq!(state.mode, ActionIdleMode::IdleToReady);
        assert_eq!(state.idle_time, 10.0);
        assert_eq!(state.ready_time, 20.0);
        assert_eq!(state.auto_idle_time, 0.72);
        assert_eq!(state.switch_time, 5.0);
    }

    fn new_idle(tenv: &mut TestEnv) -> (LogicActionIdle, Rc<InstActionIdle>) {
        let inst_act: Rc<InstActionIdle> = tenv
            .inst_player
            .find_action_by_id(id!("Action.Instance.Idle^1A"))
            .unwrap();
        let logic_act = LogicActionIdle::new(&mut tenv.context_update(), inst_act.clone()).unwrap();
        (logic_act, inst_act)
    }

    static IDLE_OZZ: &str = "Girl/Idle_Empty.*";
    static READY_OZZ: &str = "Girl/Idle_Axe.*";

    #[test]
    fn logic_new() {
        let mut tenv = TestEnv::new().unwrap();
        let (mut logic_idle, inst_idle) = new_idle(&mut tenv);
        let (mut ctx, mut ctxa, sargs) = tenv.contexts(true);
        ctxa.chara_phy.set_idle(true);

        assert_eq!(logic_idle.tmpl_id(), id!("Action.Instance.Idle^1A"));
        assert!(logic_idle.is_starting());
        assert_eq!(logic_idle.first_frame, 0);
        assert_eq!(logic_idle.last_frame, u32::MAX);
        assert_eq!(logic_idle.fade_in_weight, 0.0);
        assert_eq!(logic_idle.mode, ActionIdleMode::Idle);
        assert_eq!(logic_idle.idle_time, 0.0);
        assert_eq!(logic_idle.ready_time, 0.0);
        assert_eq!(logic_idle.auto_idle_time, 0.0);
        assert_eq!(logic_idle.switch_time, 0.0);

        logic_idle.start(&mut ctx, &mut ctxa, &sargs).unwrap();
        assert!(logic_idle.is_running());
        assert_eq!(logic_idle.first_frame, TestEnv::FRAME);
        assert_eq!(logic_idle.last_frame, u32::MAX);
        assert_eq!(logic_idle.fade_in_weight, SPF / inst_idle.anim_idle.fade_in);
        assert_eq!(logic_idle.mode, ActionIdleMode::Idle);
        assert_eq!(logic_idle.idle_time, 0.0);
        assert_eq!(logic_idle.ready_time, 0.0);
        assert_eq!(logic_idle.auto_idle_time, 0.0);
        assert_eq!(logic_idle.switch_time, 0.0);

        let state = logic_idle.save();
        assert_eq!(state.id, logic_idle.id);
        assert_eq!(state.tmpl_id, id!("Action.Instance.Idle^1A"));
        assert_eq!(state.typ, ActionType::Idle);
        assert_eq!(state.status, LogicActionStatus::Running);
        assert_eq!(state.first_frame, TestEnv::FRAME);
        assert_eq!(state.last_frame, u32::MAX);
        assert_eq!(state.animations.len(), 1);
        assert_eq!(state.animations[0].animation_id, ANIME_IDLE_ID);
        assert_eq!(state.animations[0].files, IDLE_OZZ);
        assert_eq!(state.animations[0].ratio, 0.0);
        assert_eq!(state.animations[0].weight, 1.0);
    }

    #[test]
    fn logic_first_update() {
        let mut tenv = TestEnv::new().unwrap();

        // first action
        {
            let (mut logic_idle, inst_idle) = new_idle(&mut tenv);
            let (mut ctx, mut ctxa, sargs) = tenv.contexts(false);
            ctxa.chara_phy.set_idle(true);

            logic_idle.start(&mut ctx, &mut ctxa, &sargs).unwrap();
            let ret = logic_idle.update(&mut ctx, &mut ctxa).unwrap();
            assert!(logic_idle.is_running());
            assert_eq!(logic_idle.first_frame, TestEnv::FRAME);
            assert_eq!(logic_idle.last_frame, u32::MAX);
            assert_eq!(logic_idle.fade_in_weight, 1.0);
            assert_eq!(logic_idle.mode, ActionIdleMode::Idle);
            assert_eq!(logic_idle.idle_time, SPF);
            assert_eq!(logic_idle.ready_time, 0.0);
            assert_eq!(logic_idle.auto_idle_time, 0.0);
            assert_eq!(logic_idle.switch_time, 0.0);

            let state = logic_idle.save();
            assert_eq!(state.id, logic_idle.id);
            assert_eq!(state.tmpl_id, id!("Action.Instance.Idle^1A"));
            assert_eq!(state.typ, ActionType::Idle);
            assert_eq!(state.status, LogicActionStatus::Running);
            assert_eq!(state.first_frame, TestEnv::FRAME);
            assert_eq!(state.last_frame, u32::MAX);
            assert_eq!(state.animations.len(), 1);
            assert_eq!(state.animations[0].animation_id, ANIME_IDLE_ID);
            assert_eq!(state.animations[0].files, IDLE_OZZ);
            assert_eq!(state.animations[0].ratio, inst_idle.anim_idle.ratio_warpping(SPF));
            assert_eq!(state.animations[0].weight, 1.0);
            assert!(ret.new_velocity.is_none());
            assert!(ret.new_direction.is_none());
            assert!(ret.derive_keeping.is_invalid());
        }

        // derive action
        {
            let (mut logic_idle, inst_idle) = new_idle(&mut tenv);
            let (mut ctx, mut ctxa, sargs) = tenv.contexts(true);
            ctxa.chara_phy.set_idle(false);

            logic_idle.start(&mut ctx, &mut ctxa, &sargs).unwrap();
            let ret = logic_idle.update(&mut ctx, &mut ctxa).unwrap();
            assert!(logic_idle.is_running());
            assert_eq!(logic_idle.first_frame, TestEnv::FRAME);
            assert_eq!(logic_idle.last_frame, u32::MAX);
            assert_eq!(logic_idle.fade_in_weight, 2.0 / s2f(0.4) as f32);
            assert_eq!(logic_idle.mode, ActionIdleMode::Ready);
            assert_eq!(logic_idle.idle_time, 0.0);
            assert_eq!(logic_idle.ready_time, SPF);
            assert_eq!(logic_idle.auto_idle_time, 0.0);
            assert_eq!(logic_idle.switch_time, 0.0);

            let state = logic_idle.save();
            assert_eq!(state.animations.len(), 1);
            assert_eq!(state.animations[0].animation_id, ANIME_READY_ID);
            assert_eq!(state.animations[0].files, READY_OZZ);
            assert_eq!(
                state.animations[0].ratio,
                inst_idle.anim_ready.as_ref().unwrap().ratio_warpping(SPF)
            );
            assert_eq!(state.animations[0].weight, 1.0);
            assert!(ret.new_velocity.is_none());
            assert!(ret.new_direction.is_none());
            assert!(ret.derive_keeping.is_invalid());
        }
    }

    #[test]
    fn logic_idle() {
        let mut tenv = TestEnv::new().unwrap();
        let (mut logic_idle, inst_idle) = new_idle(&mut tenv);
        let (mut ctx, mut ctxa, sargs) = tenv.contexts(true);
        ctxa.chara_phy.set_idle(true);

        logic_idle.start(&mut ctx, &mut ctxa, &sargs).unwrap();
        for ft in FrameTicker::new(1..s2f(7.0)) {
            logic_idle.update(&mut ctx, &mut ctxa).unwrap();
            assert_eq!(logic_idle.mode, ActionIdleMode::Idle);
            assert_ulps_eq!(logic_idle.idle_time, ft.time);

            let state = logic_idle.save();
            assert_eq!(state.animations.len(), 1);
            assert_eq!(state.animations[0].animation_id, ANIME_IDLE_ID);
            assert_eq!(state.animations[0].files, IDLE_OZZ);
            assert_ulps_eq!(state.animations[0].ratio, inst_idle.anim_idle.ratio_warpping(ft.time));
            assert_eq!(state.animations[0].weight, 1.0);
        }
    }

    #[test]
    fn logic_ready() {
        let mut tenv = TestEnv::new().unwrap();
        let (mut logic_idle, inst_idle) = new_idle(&mut tenv);
        let (mut ctx, mut ctxa, sargs) = tenv.contexts(true);
        ctxa.chara_phy.set_idle(false);

        logic_idle.start(&mut ctx, &mut ctxa, &sargs).unwrap();
        for ft in FrameTicker::new(1..s2f(7.0)) {
            logic_idle.update(&mut ctx, &mut ctxa).unwrap();
            assert_eq!(logic_idle.mode, ActionIdleMode::Ready);
            assert_ulps_eq!(logic_idle.ready_time, ft.time);

            let state = logic_idle.save();
            assert_eq!(state.animations.len(), 1);
            assert_eq!(state.animations[0].animation_id, ANIME_READY_ID);
            assert_eq!(state.animations[0].files, READY_OZZ);
            assert_ulps_eq!(
                state.animations[0].ratio,
                inst_idle.anim_ready.as_ref().unwrap().ratio_warpping(ft.time)
            );
            assert_eq!(state.animations[0].weight, 1.0);
        }
    }

    #[test]
    fn logic_idle_to_ready() {
        let mut tenv = TestEnv::new().unwrap();
        let (mut logic_idle, inst_idle) = new_idle(&mut tenv);
        let anim_ready = inst_idle.anim_ready.as_ref().unwrap();
        let (mut ctx, mut ctxa, sargs) = tenv.contexts(true);
        ctxa.chara_phy.set_idle(true);
        logic_idle.start(&mut ctx, &mut ctxa, &sargs).unwrap();
        logic_idle.update(&mut ctx, &mut ctxa).unwrap();
        ctxa.chara_phy.set_idle(false);

        for ft in FrameTicker::new(0..s2f(0.4)) {
            logic_idle.update(&mut ctx, &mut ctxa).unwrap();
            let state = logic_idle.save();

            if !ft.last {
                assert_eq!(logic_idle.mode, ActionIdleMode::IdleToReady);
                assert_eq!(logic_idle.switch_time, ft.time(1));
                assert_eq!(state.animations.len(), 2);

                assert_eq!(state.animations[0].animation_id, ANIME_IDLE_ID);
                assert_eq!(state.animations[0].files, IDLE_OZZ);
                assert_eq!(state.animations[0].ratio, SPF / inst_idle.anim_idle.duration);
                let idle_weight = 1.0 - logic_idle.switch_time / anim_ready.fade_in;
                assert_eq!(state.animations[0].weight, idle_weight);

                assert_eq!(state.animations[1].animation_id, ANIME_READY_ID);
                assert_eq!(state.animations[1].files, READY_OZZ);
                assert_eq!(state.animations[1].ratio, ft.time / anim_ready.duration);
                let ready_weight = logic_idle.switch_time / anim_ready.fade_in;
                assert_eq!(state.animations[1].weight, ready_weight);
            }
            else {
                assert_eq!(logic_idle.mode, ActionIdleMode::Ready);
                assert_eq!(logic_idle.switch_time, 0.0);

                assert_eq!(state.animations.len(), 1);
                assert_eq!(state.animations[0].animation_id, ANIME_READY_ID);
                assert_eq!(state.animations[0].files, READY_OZZ);
                assert_eq!(state.animations[0].ratio, ft.time / anim_ready.duration);
                assert_eq!(state.animations[0].weight, 1.0);
            }
        }
    }

    #[test]
    fn logic_ready_to_idle() {
        let mut tenv = TestEnv::new().unwrap();
        let (mut logic_idle, inst_idle) = new_idle(&mut tenv);
        let (mut ctx, mut ctxa, sargs) = tenv.contexts(true);
        ctxa.chara_phy.set_idle(false);
        logic_idle.start(&mut ctx, &mut ctxa, &sargs).unwrap();
        logic_idle.update(&mut ctx, &mut ctxa).unwrap();
        ctxa.chara_phy.set_idle(true);

        logic_idle.auto_idle_time = inst_idle.auto_idle_delay - SPF * 5.1;
        for n in [4.1, 3.1, 2.1, 1.1, 0.1] {
            logic_idle.update(&mut ctx, &mut ctxa).unwrap();
            assert_eq!(logic_idle.mode, ActionIdleMode::Ready);
            assert_ulps_eq!(logic_idle.auto_idle_time, inst_idle.auto_idle_delay - n / FPS);
        }

        for ft in FrameTicker::new(0..s2f(0.2)) {
            logic_idle.update(&mut ctx, &mut ctxa).unwrap();
            let state = logic_idle.save();
            assert_eq!(logic_idle.auto_idle_time, 0.0);

            if !ft.last {
                assert_eq!(logic_idle.mode, ActionIdleMode::ReadyToIdle);
                assert_eq!(logic_idle.switch_time, ft.time(1));

                assert_eq!(logic_idle.mode, ActionIdleMode::ReadyToIdle);
                assert_eq!(state.animations.len(), 2);

                assert_eq!(state.animations[0].animation_id, ANIME_READY_ID);
                assert_eq!(state.animations[0].files, READY_OZZ);
                assert_eq!(
                    state.animations[0].ratio,
                    6.0 / FPS / inst_idle.anim_ready.as_ref().unwrap().duration
                );
                let ready_weight = 1.0 - logic_idle.switch_time / inst_idle.anim_idle.fade_in;
                assert_eq!(state.animations[0].weight, ready_weight);

                assert_eq!(state.animations[1].animation_id, ANIME_IDLE_ID);
                assert_eq!(state.animations[1].files, IDLE_OZZ);
                assert_eq!(state.animations[1].ratio, ft.time / inst_idle.anim_idle.duration);
                let idle_weight = logic_idle.switch_time / inst_idle.anim_idle.fade_in;
                assert_eq!(state.animations[1].weight, idle_weight);
            }
            else {
                assert_eq!(logic_idle.mode, ActionIdleMode::Idle);
                assert_eq!(logic_idle.switch_time, 0.0);

                assert_eq!(state.animations.len(), 1);
                assert_eq!(state.animations[0].animation_id, ANIME_IDLE_ID);
                assert_eq!(state.animations[0].files, IDLE_OZZ);
                assert_eq!(state.animations[0].ratio, ft.time / inst_idle.anim_idle.duration);
                assert_eq!(state.animations[0].weight, 1.0);
            }
        }
    }

    #[test]
    fn logic_ready_to_idle_2() {
        let mut tenv = TestEnv::new().unwrap();
        let (mut logic_idle, inst_idle) = new_idle(&mut tenv);
        let anim_ready = inst_idle.anim_ready.as_ref().unwrap();
        let (mut ctx, mut ctxa, sargs) = tenv.contexts(true);
        ctxa.chara_phy.set_idle(false);
        logic_idle.start(&mut ctx, &mut ctxa, &sargs).unwrap();
        logic_idle.update(&mut ctx, &mut ctxa).unwrap();
        ctxa.chara_phy.set_idle(true);

        logic_idle.auto_idle_time = inst_idle.auto_idle_delay;
        for ft in FrameTicker::new(0..s2f(0.133333)) {
            logic_idle.update(&mut ctx, &mut ctxa).unwrap();
            assert_eq!(logic_idle.mode, ActionIdleMode::ReadyToIdle);
            assert_eq!(logic_idle.switch_time, ft.time(1));
        }

        ctxa.chara_phy.set_idle(false);
        let switch_base = anim_ready.fade_in * (1.0 / 3.0);
        for ft in FrameTicker::new(0..s2f(0.266667)) {
            logic_idle.update(&mut ctx, &mut ctxa).unwrap();
            let state = logic_idle.save();

            if !ft.last {
                assert_eq!(logic_idle.mode, ActionIdleMode::IdleToReady);
                assert_ulps_eq!(logic_idle.switch_time, switch_base + ft.time(1));

                assert_eq!(logic_idle.mode, ActionIdleMode::IdleToReady);
                assert_eq!(state.animations.len(), 2);

                assert_eq!(state.animations[0].animation_id, ANIME_IDLE_ID);
                assert_eq!(state.animations[0].files, IDLE_OZZ);
                assert_ulps_eq!(state.animations[0].ratio, 3.0 / FPS / inst_idle.anim_idle.duration);
                let ready_weight = 1.0 - logic_idle.switch_time / anim_ready.fade_in;
                assert_eq!(state.animations[0].weight, ready_weight);

                assert_eq!(state.animations[1].animation_id, ANIME_READY_ID);
                assert_eq!(state.animations[1].files, READY_OZZ);
                assert_eq!(state.animations[1].ratio, ft.time(1) / anim_ready.duration);
                let idle_weight = logic_idle.switch_time / anim_ready.fade_in;
                assert_eq!(state.animations[1].weight, idle_weight);
            }
            else {
                assert_eq!(logic_idle.mode, ActionIdleMode::Ready);
                assert_eq!(logic_idle.switch_time, 0.0);

                assert_eq!(state.animations.len(), 1);
                assert_eq!(state.animations[0].animation_id, ANIME_READY_ID);
                assert_eq!(state.animations[0].files, READY_OZZ);
                assert_eq!(state.animations[0].ratio, ft.time(1) / anim_ready.duration);
                assert_eq!(state.animations[0].weight, 1.0);
            }
        }
    }
}
