use cirtical_point_csgen::{CsEnum, CsOut};
use std::fmt::Debug;
use std::rc::Rc;
use std::u32;

use crate::consts::WEIGHT_THRESHOLD;
use crate::instance::{InstAction, InstActionIdle};
use crate::logic::action::base::{
    ArchivedStateAction, ContextActionNext, ContextActionUpdate, LogicAction, LogicActionBase, StateAction,
    StateActionAnimation, StateActionBase, StateActionType,
};
use crate::logic::game::ContextUpdate;
use crate::template::{TmplActionIdle, TmplType};
use crate::utils::{calc_ratio, extend, CastRef, XError, XResult};

const ANIME_IDLE_ID: u32 = 1;
const ANIME_READY_ID: u32 = 2;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsEnum)]
#[archive_attr(derive(Debug))]
pub enum ActionIdleMode {
    None,
    Idle,
    Ready,
    IdleToReady,
    ReadyToIdle,
    // Random,
}

#[repr(C)]
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateActionIdle {
    pub _base: StateActionBase,
    pub mode: ActionIdleMode,
    pub idle_progress: u32,
    pub ready_progress: u32,
    pub idle_timer: u32,
    pub switch_progress: u32,
}

extend!(StateActionIdle, StateActionBase);

unsafe impl StateAction for StateActionIdle {
    #[inline]
    fn typ(&self) -> StateActionType {
        assert!(self.typ == StateActionType::Idle);
        StateActionType::Idle
    }

    #[inline]
    fn tmpl_typ(&self) -> TmplType {
        assert!(self.tmpl_typ == TmplType::ActionIdle);
        TmplType::ActionIdle
    }
}

impl ArchivedStateAction for rkyv::Archived<StateActionIdle> {
    #[inline]
    fn typ(&self) -> StateActionType {
        StateActionType::Idle
    }

    #[inline]
    fn tmpl_typ(&self) -> TmplType {
        TmplType::ActionIdle
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionIdle {
    _base: LogicActionBase,
    tmpl: Rc<TmplActionIdle>,
    inst: Rc<InstActionIdle>,

    mode: ActionIdleMode,
    idle_progress: u32,
    ready_progress: u32,
    idle_timer: u32,
    switch_progress: u32,
}

extend!(LogicActionIdle, LogicActionBase);

unsafe impl LogicAction for LogicActionIdle {
    #[inline]
    fn typ(&self) -> StateActionType {
        StateActionType::Idle
    }

    #[inline]
    fn tmpl_typ(&self) -> TmplType {
        TmplType::ActionIdle
    }

    #[inline]
    fn restore(&mut self, state: &(dyn StateAction + 'static)) -> XResult<()> {
        self.restore_impl(state)
    }

    #[inline]
    fn next(&mut self, ctx: &mut ContextUpdate<'_>, ctx_an: &ContextActionNext) -> XResult<Option<Rc<dyn InstAction>>> {
        self.next_impl(ctx, ctx_an)
    }

    #[inline]
    fn update(&mut self, ctx: &mut ContextUpdate<'_>, ctx_au: &mut ContextActionUpdate<'_>) -> XResult<()> {
        self.update_impl(ctx, ctx_au)
    }
}

impl LogicActionIdle {
    pub fn new(ctx: &mut ContextUpdate<'_>, inst_act: Rc<InstActionIdle>) -> XResult<LogicActionIdle> {
        Ok(LogicActionIdle {
            _base: LogicActionBase {
                derive_level: inst_act.derive_level,
                antibreak_level: inst_act.antibreak_level,
                ..LogicActionBase::new(ctx.gene.gen_id(), inst_act.id.clone(), ctx.frame)
            },
            tmpl: inst_act.tmpl.clone(),
            inst: inst_act,

            mode: ActionIdleMode::None,
            idle_progress: 0,
            ready_progress: 0,
            idle_timer: 0,
            switch_progress: 0,
        })
    }

    fn restore_impl(&mut self, state: &(dyn StateAction + 'static)) -> XResult<()> {
        if state.id != self._base.id {
            return Err(XError::IDMissMatch);
        }
        let state = state.cast_ref::<StateActionIdle>()?;

        self._base.restore(&state._base);
        self.mode = state.mode;
        self.idle_progress = state.idle_progress;
        self.ready_progress = state.ready_progress;
        self.idle_timer = state.idle_timer;
        self.switch_progress = state.switch_progress;
        Ok(())
    }

    fn save(&self) -> Box<StateActionIdle> {
        Box::new(StateActionIdle {
            _base: self._base.save(self.typ(), self.tmpl_typ()),
            mode: self.mode,
            idle_progress: self.idle_progress,
            ready_progress: self.ready_progress,
            idle_timer: self.idle_timer,
            switch_progress: self.switch_progress,
        })
    }

    fn next_impl(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        ctx_an: &ContextActionNext,
    ) -> XResult<Option<Rc<dyn InstAction>>> {
        self._base.handle_next(ctx, ctx_an, true)
    }

    fn update_impl(&mut self, ctx: &mut ContextUpdate<'_>, ctx_au: &mut ContextActionUpdate<'_>) -> XResult<()> {
        let anim_weight = match self._base.handle_enter_leave(ctx, ctx_au, self.tmpl.enter_time) {
            Some(ratio) => ratio,
            None => return Ok(()),
        };

        if ctx_au.chara_physics.is_idle() {
            self.idle_timer += 1;
        } else {
            self.idle_timer = 0;
        }
        match self.mode {
            ActionIdleMode::None => {
                if ctx_au.chara_physics.is_idle() {
                    self.mode = ActionIdleMode::Idle;
                    self.switch_progress = 0;
                } else {
                    self.mode = ActionIdleMode::Ready;
                    self.switch_progress = self.tmpl.switch_time;
                }
            }
            ActionIdleMode::Idle => {
                if !ctx_au.chara_physics.is_idle() {
                    self.mode = ActionIdleMode::IdleToReady;
                    self.ready_progress = 0;
                    self.switch_progress = 0;
                }
            }
            ActionIdleMode::Ready => {
                if ctx_au.chara_physics.is_idle() && self.idle_timer > self.tmpl.idle_enter_delay {
                    self.mode = ActionIdleMode::ReadyToIdle;
                    self.idle_progress = 0;
                    self.switch_progress = self.tmpl.switch_time;
                }
            }
            ActionIdleMode::ReadyToIdle => {
                if !ctx_au.chara_physics.is_idle() {
                    self.mode = ActionIdleMode::IdleToReady;
                }
            }
            _ => { /* do nothing */ }
        };

        let state = match self.mode {
            ActionIdleMode::Idle => self.do_idle(anim_weight),
            ActionIdleMode::Ready => self.do_ready(anim_weight),
            ActionIdleMode::IdleToReady => self.do_idle_to_ready(anim_weight),
            ActionIdleMode::ReadyToIdle => self.do_ready_to_idle(anim_weight),
            // ActionIdleMode::Random => unimplemented!(),
            _ => {
                unreachable!()
            }
        };
        ctx_au.state(state);
        Ok(())
    }

    fn do_idle(&mut self, weight: f32) -> Box<StateActionIdle> {
        let anime_idle = &self.tmpl.anime_idle;
        self.idle_progress = (self.idle_progress + 1) % anime_idle.duration;
        let state_idle = StateActionAnimation {
            animation_id: ANIME_IDLE_ID,
            file: anime_idle.file.clone(),
            ratio: calc_ratio(self.idle_progress, anime_idle.duration),
            weight,
        };

        let mut state = self.save();
        state.animations[0] = state_idle;
        state
    }

    fn do_ready(&mut self, weight: f32) -> Box<StateActionIdle> {
        let anime_ready = &self.tmpl.anime_ready;
        self.ready_progress = (self.ready_progress + 1) % anime_ready.duration;
        let state_ready = StateActionAnimation {
            animation_id: ANIME_READY_ID,
            file: anime_ready.file.clone(),
            ratio: calc_ratio(self.ready_progress, anime_ready.duration),
            weight,
        };

        let mut state = self.save();
        state.animations[0] = state_ready;
        state
    }

    fn do_idle_to_ready(&mut self, weight: f32) -> Box<StateActionIdle> {
        self.switch_progress += 1;
        let switch_weight = calc_ratio(self.switch_progress, self.tmpl.switch_time);

        let anime_idle = &self.tmpl.anime_idle;
        self.idle_progress = (self.idle_progress + 1) % anime_idle.duration;
        let state_idle = StateActionAnimation {
            animation_id: ANIME_IDLE_ID,
            file: anime_idle.file.clone(),
            ratio: calc_ratio(self.idle_progress, anime_idle.duration),
            weight: weight * (1.0 - switch_weight),
        };

        let anime_ready = &self.tmpl.anime_ready;
        self.ready_progress = (self.ready_progress + 1) % anime_ready.duration;
        let state_ready = StateActionAnimation {
            animation_id: ANIME_READY_ID,
            file: anime_ready.file.clone(),
            ratio: calc_ratio(self.ready_progress, anime_ready.duration),
            weight: weight * switch_weight,
        };

        if self.switch_progress >= self.tmpl.switch_time {
            self.mode = ActionIdleMode::Ready;
        }

        let mut state = self.save();
        state.animations[0] = state_idle;
        state.animations[1] = state_ready;
        state
    }

    fn do_ready_to_idle(&mut self, weight: f32) -> Box<StateActionIdle> {
        self.switch_progress -= 1;
        let switch_weight = calc_ratio(self.tmpl.switch_time - self.switch_progress, self.tmpl.switch_time);

        let anime_ready = &self.tmpl.anime_ready;
        self.ready_progress = (self.ready_progress + 1) % anime_ready.duration;
        let state_ready = StateActionAnimation {
            animation_id: ANIME_READY_ID,
            file: anime_ready.file.clone(),
            ratio: calc_ratio(self.ready_progress, anime_ready.duration),
            weight: weight * (1.0 - switch_weight),
        };

        let anime_idle = &self.tmpl.anime_idle;
        self.idle_progress = (self.idle_progress + 1) % anime_idle.duration;
        let state_idle = StateActionAnimation {
            animation_id: ANIME_IDLE_ID,
            file: anime_idle.file.clone(),
            ratio: calc_ratio(self.idle_progress, anime_idle.duration),
            weight: weight * switch_weight,
        };

        if self.switch_progress <= 0 {
            self.mode = ActionIdleMode::Idle;
        }

        let mut state = self.save();
        state.animations[0] = state_ready;
        state.animations[1] = state_idle;
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::FPS;
    use crate::instance::InstPlayer;
    use crate::logic::character::{LogicCharaPhysics, LogicPlayer};
    use crate::logic::game::{ContextUpdate, LogicSystems};
    use crate::logic::test_utils::*;
    use crate::utils::s;
    use float_cmp::assert_approx_eq;

    static IDLE_OZZ: &str = "girl_animation_logic_stand_idle.ozz";
    static READY_OZZ: &str = "girl_animation_logic_stand_ready.ozz";

    struct AllInOne {
        frame: u32,
        systems: LogicSystems,
        inst_player: Rc<InstPlayer>,
        chara_physics: LogicCharaPhysics,
        logic_idle: LogicActionIdle,
    }

    impl AllInOne {
        fn new(frame: u32) -> AllInOne {
            let mut systems = mock_logic_systems();
            let inst_player = mock_inst_player(&mut systems);
            let chara_physics = LogicCharaPhysics::mock(1, inst_player.clone());
            let mut ctx = ContextUpdate::new_empty(&mut systems);
            ctx.frame = frame;
            let inst_idle: Rc<InstActionIdle> = inst_player.find_action_by_id(&s!("Action.No1.Idle")).unwrap();
            let logic_idle = LogicActionIdle::new(&mut ctx, inst_idle).unwrap();
            AllInOne {
                frame,
                systems,
                inst_player,
                chara_physics,
                logic_idle,
            }
        }

        fn prepare(
            &mut self,
        ) -> (
            &mut LogicActionIdle,
            ContextUpdate<'_>,
            ContextActionUpdate<'_>,
            ContextActionNext,
        ) {
            (
                &mut self.logic_idle,
                ContextUpdate::new_empty(&mut self.systems),
                ContextActionUpdate::new(1, self.inst_player.clone(), &mut self.chara_physics, 0),
                ContextActionNext::new(1, self.inst_player.clone()),
            )
        }
    }

    #[test]
    fn test_logic_action_idle_new() {
        let mut aio = AllInOne::new(666);
        let (logic_idle, _, _, _) = aio.prepare();

        assert_eq!(logic_idle.tmpl_id, s!("Action.No1.Idle"));
        assert_eq!(logic_idle.spawn_frame, 666);
        assert_eq!(logic_idle.death_frame, u32::MAX);
        assert_eq!(logic_idle.enter_progress, 0);
        assert!(!logic_idle.is_leaving);
        assert_eq!(logic_idle.event_idx, 0);
        assert_eq!(logic_idle.derive_level, 0);
        assert_eq!(logic_idle.antibreak_level, 0);
        assert_eq!(logic_idle.body_ratio, 0.0);

        assert_eq!(logic_idle.mode, ActionIdleMode::None);
        assert_eq!(logic_idle.idle_progress, 0);
        assert_eq!(logic_idle.ready_progress, 0);
        assert_eq!(logic_idle.idle_timer, 0);
        assert_eq!(logic_idle.switch_progress, 0);
    }

    #[test]
    fn test_logic_action_idle_first_update() {
        // first action
        {
            let mut aio = AllInOne::new(666);
            let (logic_idle, mut ctx, mut ctx_au, _) = aio.prepare();

            logic_idle.update(&mut ctx, &mut ctx_au).unwrap();
            assert_eq!(logic_idle.spawn_frame, 666);
            assert_eq!(logic_idle.death_frame, u32::MAX);
            assert_eq!(logic_idle.enter_progress, 0);
            assert!(!logic_idle.is_leaving);
            assert_eq!(logic_idle.mode, ActionIdleMode::Idle);
            assert_eq!(logic_idle.idle_progress, 1);
            assert_eq!(logic_idle.ready_progress, 0);
            assert_eq!(logic_idle.idle_timer, 1);
            assert_eq!(logic_idle.switch_progress, 0);

            let state = ctx_au.states[0].cast_ref::<StateActionIdle>().unwrap();
            assert_eq!(state.animations[0].animation_id, ANIME_IDLE_ID);
            assert_eq!(state.animations[0].file, s!(IDLE_OZZ));
            assert_eq!(state.animations[0].ratio, 1.0 / 30.0);
            assert_eq!(state.animations[0].weight, 1.0);
            assert!(state.animations[1].is_empty());
        }

        // derive action
        {
            let mut aio = AllInOne::new(666);
            let (logic_idle, mut ctx, mut ctx_au, _) = aio.prepare();
            let logic_empty = LogicActionEmpty::new(100001);
            ctx_au.prev_action = Some(logic_empty.as_ref());

            logic_idle.update(&mut ctx, &mut ctx_au).unwrap();
            assert_eq!(logic_idle.spawn_frame, 666);
            assert_eq!(logic_idle.death_frame, u32::MAX);
            assert_eq!(logic_idle.enter_progress, 1);
            assert!(!logic_idle.is_leaving);
            assert_eq!(logic_idle.mode, ActionIdleMode::Ready);
            assert_eq!(logic_idle.idle_progress, 0);
            assert_eq!(logic_idle.ready_progress, 1);
            assert_eq!(logic_idle.idle_timer, 0);
            assert_eq!(logic_idle.switch_progress, 5);

            let state = ctx_au.states[0].cast_ref::<StateActionIdle>().unwrap();
            assert_eq!(state.animations[0].animation_id, ANIME_READY_ID);
            assert_eq!(state.animations[0].file, s!(READY_OZZ));
            assert_eq!(state.animations[0].ratio, 1.0 / 30.0);
            assert_eq!(state.animations[0].weight, 1.0 / 5.0);
            assert!(state.animations[1].is_empty());
        }
    }

    #[test]
    fn test_logic_action_idle_idle() {
        let mut aio = AllInOne::new(10);
        let (logic_idle, mut ctx, mut ctx_au, _) = aio.prepare();
        ctx_au.chara_physics.idle = true;
        let logic_empty = LogicActionEmpty::new(100001);
        ctx_au.prev_action = Some(logic_empty.as_ref());

        for idx in 1..=5 {
            ctx_au.unused_weight = 1.0;
            ctx_au.states.clear();
            logic_idle.update(&mut ctx, &mut ctx_au).unwrap();
            assert_eq!(logic_idle.mode, ActionIdleMode::Idle);
            assert_eq!(logic_idle.idle_progress, idx);

            let state = ctx_au.states[0].cast_ref::<StateActionIdle>().unwrap();
            assert_eq!(state.animations[0].animation_id, ANIME_IDLE_ID);
            assert_eq!(state.animations[0].file, s!(IDLE_OZZ));
            assert_eq!(state.animations[0].ratio, (idx as f32) / 30.0);
            assert_eq!(state.animations[0].weight, (idx as f32) / 5.0);
            assert!(state.animations[1].is_empty());
        }

        for idx in 6..=40 {
            ctx_au.unused_weight = 1.0;
            ctx_au.states.clear();
            logic_idle.update(&mut ctx, &mut ctx_au).unwrap();
            assert_eq!(logic_idle.mode, ActionIdleMode::Idle);
            assert_eq!(logic_idle.idle_progress, idx % 30);

            let state = ctx_au.states[0].cast_ref::<StateActionIdle>().unwrap();
            assert_eq!(state.animations[0].animation_id, ANIME_IDLE_ID);
            assert_eq!(state.animations[0].file, s!(IDLE_OZZ));
            assert_eq!(state.animations[0].ratio, ((idx % 30) as f32) / 30.0);
            assert_eq!(state.animations[0].weight, 1.0);
            assert!(state.animations[1].is_empty());
        }
    }

    #[test]
    fn test_logic_action_idle_ready() {
        let mut aio = AllInOne::new(10);
        let (logic_idle, mut ctx, mut ctx_au, _) = aio.prepare();
        ctx_au.chara_physics.idle = false;

        for idx in 1..=40 {
            ctx_au.unused_weight = 1.0;
            ctx_au.states.clear();
            logic_idle.update(&mut ctx, &mut ctx_au).unwrap();
            assert_eq!(logic_idle.mode, ActionIdleMode::Ready);
            assert_eq!(logic_idle.ready_progress, idx % 30);

            let state = ctx_au.states[0].cast_ref::<StateActionIdle>().unwrap();
            assert_eq!(state.animations[0].animation_id, ANIME_READY_ID);
            assert_eq!(state.animations[0].file, s!(READY_OZZ));
            assert_eq!(state.animations[0].ratio, ((idx % 30) as f32) / 30.0);
            assert_eq!(state.animations[0].weight, 1.0);
            assert!(state.animations[1].is_empty());
        }
    }

    #[test]
    fn test_logic_action_idle_idle_to_ready() {
        let mut aio = AllInOne::new(10);
        let (logic_idle, mut ctx, mut ctx_au, _) = aio.prepare();
        let logic_empty = LogicActionEmpty::new(100001);
        ctx_au.chara_physics.idle = false;
        ctx_au.prev_action = Some(logic_empty.as_ref());
        logic_idle.mode = ActionIdleMode::Idle;

        for idx in 1..=5 {
            ctx_au.unused_weight = 1.0;
            ctx_au.states.clear();
            logic_idle.update(&mut ctx, &mut ctx_au).unwrap();
            if idx != 5 {
                assert_eq!(logic_idle.mode, ActionIdleMode::IdleToReady);
            } else {
                assert_eq!(logic_idle.mode, ActionIdleMode::Ready);
            }
            assert_eq!(logic_idle.switch_progress, idx);
            assert_eq!(logic_idle.idle_progress, idx % 30);
            assert_eq!(logic_idle.ready_progress, idx % 30);

            let state = ctx_au.states[0].cast_ref::<StateActionIdle>().unwrap();
            assert_eq!(state.animations[0].animation_id, ANIME_IDLE_ID);
            assert_eq!(state.animations[0].file, s!(IDLE_OZZ));
            assert_eq!(state.animations[0].ratio, (idx as f32) / 30.0);
            assert_eq!(
                state.animations[0].weight,
                ((idx as f32) / 5.0) * (1.0 - (idx as f32) / 5.0)
            );
            assert_eq!(state.animations[1].animation_id, ANIME_READY_ID);
            assert_eq!(state.animations[1].file, s!(READY_OZZ));
            assert_eq!(state.animations[1].ratio, (idx as f32) / 30.0);
            assert_eq!(state.animations[1].weight, ((idx as f32) / 5.0) * ((idx as f32) / 5.0));
            assert!(state.animations[2].is_empty());
        }
    }

    #[test]
    fn test_logic_action_idle_ready_to_idle() {
        let mut aio = AllInOne::new(10);
        let (logic_idle, mut ctx, mut ctx_au, _) = aio.prepare();
        ctx_au.chara_physics.idle = true;
        logic_idle.mode = ActionIdleMode::Ready;

        for idx in 1..=5 {
            ctx_au.unused_weight = 1.0;
            ctx_au.states.clear();
            logic_idle.update(&mut ctx, &mut ctx_au).unwrap();
            assert_eq!(logic_idle.mode, ActionIdleMode::Ready);
            assert_eq!(logic_idle.idle_timer, idx);
        }

        logic_idle.idle_timer = 5 * FPS;
        for idx in 6..=10 {
            let idx5 = idx - 5;
            ctx_au.unused_weight = 1.0;
            ctx_au.states.clear();
            logic_idle.update(&mut ctx, &mut ctx_au).unwrap();
            if idx != 10 {
                assert_eq!(logic_idle.mode, ActionIdleMode::ReadyToIdle);
            } else {
                assert_eq!(logic_idle.mode, ActionIdleMode::Idle);
            }
            assert_eq!(logic_idle.switch_progress, 10 - idx);
            assert_eq!(logic_idle.idle_progress, idx5 % 30);
            assert_eq!(logic_idle.ready_progress, idx % 30);

            let state = ctx_au.states[0].cast_ref::<StateActionIdle>().unwrap();
            assert_eq!(state.animations[0].animation_id, ANIME_READY_ID);
            assert_eq!(state.animations[0].file, s!(READY_OZZ));
            assert_eq!(state.animations[0].ratio, (idx as f32) / 30.0);
            assert_eq!(state.animations[0].weight, 1.0 - (idx5 as f32) / 5.0);
            assert_eq!(state.animations[1].animation_id, ANIME_IDLE_ID);
            assert_eq!(state.animations[1].file, s!(IDLE_OZZ));
            assert_eq!(state.animations[1].ratio, (idx5 as f32) / 30.0);
            assert_eq!(state.animations[1].weight, (idx5 as f32) / 5.0);
            assert!(state.animations[2].is_empty());
        }
    }

    #[test]
    fn test_logic_action_idle_ready_to_idle_2() {
        let mut aio = AllInOne::new(10);
        let (logic_idle, mut ctx, mut ctx_au, _) = aio.prepare();
        logic_idle.mode = ActionIdleMode::Ready;

        ctx_au.chara_physics.idle = true;
        logic_idle.idle_timer = 5 * FPS;
        logic_idle.mode = ActionIdleMode::Ready;
        for _ in 1..=4 {
            ctx_au.unused_weight = 1.0;
            ctx_au.states.clear();
            logic_idle.update(&mut ctx, &mut ctx_au).unwrap();
            assert_eq!(logic_idle.mode, ActionIdleMode::ReadyToIdle);
        }

        ctx_au.chara_physics.idle = false;
        for idx in 5..=8 {
            let idx4 = 8 - idx; // 3 2 1 0
            ctx_au.unused_weight = 1.0;
            ctx_au.states.clear();
            logic_idle.update(&mut ctx, &mut ctx_au).unwrap();
            if idx != 8 {
                assert_eq!(logic_idle.mode, ActionIdleMode::IdleToReady);
            } else {
                assert_eq!(logic_idle.mode, ActionIdleMode::Ready);
            }
            assert_eq!(logic_idle.switch_progress, 5 - idx4);
            assert_eq!(logic_idle.idle_progress, idx % 30);
            assert_eq!(logic_idle.ready_progress, idx % 30);

            let state = ctx_au.states[0].cast_ref::<StateActionIdle>().unwrap();
            assert_eq!(logic_idle.idle_timer, 0);
            assert_eq!(state.animations[0].animation_id, ANIME_IDLE_ID);
            assert_eq!(state.animations[0].file, s!(IDLE_OZZ));
            assert_eq!(state.animations[0].ratio, (idx as f32) / 30.0);
            assert_approx_eq!(f32, state.animations[0].weight, (idx4 as f32) / 5.0);
            assert_eq!(state.animations[1].animation_id, ANIME_READY_ID);
            assert_eq!(state.animations[1].file, s!(READY_OZZ));
            assert_eq!(state.animations[1].ratio, (idx as f32) / 30.0);
            assert_approx_eq!(f32, state.animations[1].weight, 1.0 - (idx4 as f32) / 5.0);
            assert!(state.animations[2].is_empty());
        }
    }

    #[test]
    fn test_logic_action_next() {
        // todo!("not implemented");
    }
}
