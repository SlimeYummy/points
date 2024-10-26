use cirtical_point_csgen::CsGen;
use std::fmt::Debug;
use std::rc::Rc;
use std::u32;

use crate::instance::{InstAction, InstActionIdle};
use crate::logic::action::base::{
    ArchivedStateAction, ContextActionNext, ContextActionUpdate, LogicAction, LogicActionBase, StateAction,
    StateActionAnimation, StateActionBase, WEIGHT_THRESHOLD,
};
use crate::logic::game::ContextUpdate;
use crate::template::{TmplActionIdle, TmplClass};
use crate::utils::{extend, to_ratio, to_ratio_clamp, CastRef, XError, XResult};

const ANIME_IDLE_ID: u32 = 1;
const ANIME_READY_ID: u32 = 2;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsGen)]
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
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsGen)]
#[archive_attr(derive(Debug))]
#[cs_attr(Rs, Ref)]
pub struct StateActionIdle {
    pub _base: StateActionBase,
    pub event_idx: u64,
    pub mode: ActionIdleMode,
    pub is_dying: bool,
    pub enter_progress: u32,
    pub idle_progress: u32,
    pub ready_progress: u32,
    pub idle_timer: u32,
    pub switch_progress: u32,
}

extend!(StateActionIdle, StateActionBase);

unsafe impl StateAction for StateActionIdle {
    #[inline]
    fn class(&self) -> TmplClass {
        TmplClass::ActionIdle
    }
}

impl ArchivedStateAction for rkyv::Archived<StateActionIdle> {
    fn class(&self) -> TmplClass {
        TmplClass::ActionIdle
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionIdle {
    _base: LogicActionBase,
    tmpl: Rc<TmplActionIdle>,
    inst: Rc<InstActionIdle>,

    event_idx: u64,
    mode: ActionIdleMode,
    is_dying: bool,
    enter_progress: u32,
    idle_progress: u32,
    ready_progress: u32,
    idle_timer: u32,
    switch_progress: u32,
}

extend!(LogicActionIdle, LogicActionBase);

unsafe impl LogicAction for LogicActionIdle {
    #[inline]
    fn class(&self) -> TmplClass {
        TmplClass::ActionIdle
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
                id: ctx.gene.gen_id(),
                tmpl_id: inst_act.id.clone(),
                spawn_frame: ctx.frame,
                dead_frame: u32::MAX,
                derive_level: inst_act.derive_level,
                antibreak_level: inst_act.antibreak_level,
                blend_weight: 0.0,
                body_ratio: 0.0,
            },
            tmpl: inst_act.tmpl.clone(),
            inst: inst_act,

            event_idx: 0,
            mode: ActionIdleMode::None,
            is_dying: false,
            enter_progress: 0,
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
        self.event_idx = state.event_idx;
        self.mode = state.mode;
        self.idle_progress = state.idle_progress;
        self.ready_progress = state.ready_progress;
        self.idle_timer = state.idle_timer;
        self.switch_progress = state.switch_progress;
        Ok(())
    }

    fn save(&self) -> Box<StateActionIdle> {
        Box::new(StateActionIdle {
            _base: self._base.save(),
            event_idx: self.event_idx,
            mode: self.mode,
            is_dying: self.is_dying,
            enter_progress: self.enter_progress,
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
        let frame = ctx.frame;
        let mut inst_next = None;
        let mut events = ctx.input.player_events(ctx_an.player_id, frame)?;
        for event in events.iter(self.event_idx) {
            inst_next = ctx_an
                .inst_player
                .search_next_action(&self.tmpl_id, self.derive_level, event.key());
        }
        self.event_idx = events.future_idx();

        if inst_next.is_some() {
            self.is_dying = true;
            events.consume(self.event_idx)?;
        }
        Ok(inst_next)
    }

    fn update_impl(&mut self, ctx: &mut ContextUpdate<'_>, ctx_au: &mut ContextActionUpdate<'_>) -> XResult<()> {
        let local_weight = match ctx_au.prev_action {
            None => 1.0,
            Some(_) => {
                self.enter_progress += 1;
                to_ratio_clamp(self.enter_progress, self.tmpl.enter_time)
            }
        };
        let real_weight = ctx_au.apply_weight(local_weight);
        if self.is_dying && real_weight < WEIGHT_THRESHOLD {
            // action finished
            self.dead_frame = ctx.frame;
            return Ok(());
        }

        if ctx_au.is_idle {
            self.idle_timer += 1;
        } else {
            self.idle_timer = 0;
        }
        match self.mode {
            ActionIdleMode::None => {
                if ctx_au.is_idle {
                    self.mode = ActionIdleMode::Idle;
                    self.switch_progress = 0;
                } else {
                    self.mode = ActionIdleMode::Ready;
                    self.switch_progress = self.tmpl.switch_time;
                }
            }
            ActionIdleMode::Idle => {
                if !ctx_au.is_idle {
                    self.mode = ActionIdleMode::IdleToReady;
                    self.ready_progress = 0;
                    self.switch_progress = 0;
                }
            }
            ActionIdleMode::Ready => {
                if ctx_au.is_idle && self.idle_timer > self.tmpl.idle_enter_delay {
                    self.mode = ActionIdleMode::ReadyToIdle;
                    self.idle_progress = 0;
                    self.switch_progress = self.tmpl.switch_time;
                }
            }
            ActionIdleMode::ReadyToIdle => {
                if !ctx_au.is_idle {
                    self.mode = ActionIdleMode::IdleToReady;
                }
            }
            _ => { /* do nothing */ }
        };

        let state = match self.mode {
            ActionIdleMode::Idle => self.do_idle(real_weight),
            ActionIdleMode::Ready => self.do_ready(real_weight),
            ActionIdleMode::IdleToReady => self.do_idle_to_ready(real_weight),
            ActionIdleMode::ReadyToIdle => self.do_ready_to_idle(real_weight),
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
            ratio: to_ratio(self.idle_progress, anime_idle.duration),
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
            ratio: to_ratio(self.ready_progress, anime_ready.duration),
            weight,
        };

        let mut state = self.save();
        state.animations[0] = state_ready;
        state
    }

    fn do_idle_to_ready(&mut self, weight: f32) -> Box<StateActionIdle> {
        self.switch_progress += 1;
        let switch_weight = to_ratio(self.switch_progress, self.tmpl.switch_time);

        let anime_idle = &self.tmpl.anime_idle;
        self.idle_progress = (self.idle_progress + 1) % anime_idle.duration;
        let state_idle = StateActionAnimation {
            animation_id: ANIME_IDLE_ID,
            file: anime_idle.file.clone(),
            ratio: to_ratio(self.idle_progress, anime_idle.duration),
            weight: weight * (1.0 - switch_weight),
        };

        let anime_ready = &self.tmpl.anime_ready;
        self.ready_progress = (self.ready_progress + 1) % anime_ready.duration;
        let state_ready = StateActionAnimation {
            animation_id: ANIME_READY_ID,
            file: anime_ready.file.clone(),
            ratio: to_ratio(self.ready_progress, anime_ready.duration),
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
        let switch_weight = to_ratio(self.tmpl.switch_time - self.switch_progress, self.tmpl.switch_time);

        let anime_ready = &self.tmpl.anime_ready;
        self.ready_progress = (self.ready_progress + 1) % anime_ready.duration;
        let state_ready = StateActionAnimation {
            animation_id: ANIME_READY_ID,
            file: anime_ready.file.clone(),
            ratio: to_ratio(self.ready_progress, anime_ready.duration),
            weight: weight * (1.0 - switch_weight),
        };

        let anime_idle = &self.tmpl.anime_idle;
        self.idle_progress = (self.idle_progress + 1) % anime_idle.duration;
        let state_idle = StateActionAnimation {
            animation_id: ANIME_IDLE_ID,
            file: anime_idle.file.clone(),
            ratio: to_ratio(self.idle_progress, anime_idle.duration),
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
    use crate::instance::InstPlayer;
    use crate::logic::game::{ContextUpdate, LogicSystems};
    use crate::logic::test_utils::*;
    use crate::utils::{s, FPS};
    use float_cmp::assert_approx_eq;

    static IDLE_OZZ: &str = "girl_animation_logic_stand_idle.ozz";
    static READY_OZZ: &str = "girl_animation_logic_stand_ready.ozz";

    fn prepare<'b>(
        systems: &mut LogicSystems,
        inst_player: Rc<InstPlayer>,
        frame: u32,
    ) -> (
        Box<LogicActionIdle>,
        ContextUpdate<'_>,
        ContextActionUpdate<'b>,
        ContextActionNext,
    ) {
        let mut ctx = ContextUpdate::new_empty(systems);
        ctx.frame = frame;
        let inst_idle: Rc<InstActionIdle> = inst_player.find_action_by_id(&s!("Action.No1.Idle")).unwrap();
        let logic_idle = Box::new(LogicActionIdle::new(&mut ctx, inst_idle).unwrap());
        let ctx_au = ContextActionUpdate::new(1, inst_player.clone(), 0);
        let ctx_an = ContextActionNext::new(1, inst_player.clone());
        (logic_idle, ctx, ctx_au, ctx_an)
    }

    #[test]
    fn test_logic_action_idle_new() {
        let mut systems = new_logic_systems();
        let inst_player = new_inst_player(&mut systems);
        let (logic_idle, _, _, _) = prepare(&mut systems, inst_player, 666);

        assert_eq!(logic_idle.tmpl_id, s!("Action.No1.Idle"));
        assert_eq!(logic_idle.spawn_frame, 666);
        assert_eq!(logic_idle.dead_frame, u32::MAX);
        assert_eq!(logic_idle.derive_level, 0);
        assert_eq!(logic_idle.antibreak_level, 0);
        assert_eq!(logic_idle.blend_weight, 0.0);
        assert_eq!(logic_idle.body_ratio, 0.0);

        assert_eq!(logic_idle.event_idx, 0);
        assert_eq!(logic_idle.mode, ActionIdleMode::None);
        assert!(!logic_idle.is_dying);
        assert_eq!(logic_idle.enter_progress, 0);
        assert_eq!(logic_idle.idle_progress, 0);
        assert_eq!(logic_idle.ready_progress, 0);
        assert_eq!(logic_idle.idle_timer, 0);
        assert_eq!(logic_idle.switch_progress, 0);
    }

    #[test]
    fn test_logic_action_idle_first_update() {
        let mut systems = new_logic_systems();
        let inst_player = new_inst_player(&mut systems);

        // first action
        {
            let (mut logic_idle, mut ctx, mut ctx_au, _) = prepare(&mut systems, inst_player.clone(), 666);
            ctx_au.is_idle = true;

            logic_idle.update(&mut ctx, &mut ctx_au).unwrap();
            assert_eq!(logic_idle.spawn_frame, 666);
            assert_eq!(logic_idle.dead_frame, u32::MAX);
            assert_eq!(logic_idle.mode, ActionIdleMode::Idle);
            assert!(!logic_idle.is_dying);
            assert_eq!(logic_idle.enter_progress, 0);
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
            let (mut logic_idle, mut ctx, mut ctx_au, _) = prepare(&mut systems, inst_player.clone(), 666);
            ctx_au.is_idle = false;
            let logic_empty = LogicActionEmpty::new(100001);
            ctx_au.prev_action = Some(logic_empty.as_ref());

            logic_idle.update(&mut ctx, &mut ctx_au).unwrap();
            assert_eq!(logic_idle.spawn_frame, 666);
            assert_eq!(logic_idle.dead_frame, u32::MAX);
            assert_eq!(logic_idle.mode, ActionIdleMode::Ready);
            assert!(!logic_idle.is_dying);
            assert_eq!(logic_idle.enter_progress, 1);
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
        let mut systems = new_logic_systems();
        let inst_player = new_inst_player(&mut systems);
        let (mut logic_idle, mut ctx, mut ctx_au, _) = prepare(&mut systems, inst_player.clone(), 10);

        ctx_au.is_idle = true;
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
        let mut systems = new_logic_systems();
        let inst_player = new_inst_player(&mut systems);
        let (mut logic_idle, mut ctx, mut ctx_au, _) = prepare(&mut systems, inst_player.clone(), 10);
        ctx_au.is_idle = false;

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
        let mut systems = new_logic_systems();
        let inst_player = new_inst_player(&mut systems);
        let (mut logic_idle, mut ctx, mut ctx_au, _) = prepare(&mut systems, inst_player.clone(), 10);

        let logic_empty = LogicActionEmpty::new(100001);
        ctx_au.is_idle = false;
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
        let mut systems = new_logic_systems();
        let inst_player = new_inst_player(&mut systems);
        let (mut logic_idle, mut ctx, mut ctx_au, _) = prepare(&mut systems, inst_player.clone(), 10);

        ctx_au.is_idle = true;
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
        let mut systems = new_logic_systems();
        let inst_player = new_inst_player(&mut systems);
        let (mut logic_idle, mut ctx, mut ctx_au, _) = prepare(&mut systems, inst_player.clone(), 10);

        ctx_au.is_idle = true;
        logic_idle.idle_timer = 5 * FPS;
        logic_idle.mode = ActionIdleMode::Ready;
        for _ in 1..=4 {
            ctx_au.unused_weight = 1.0;
            ctx_au.states.clear();
            logic_idle.update(&mut ctx, &mut ctx_au).unwrap();
            assert_eq!(logic_idle.mode, ActionIdleMode::ReadyToIdle);
        }

        ctx_au.is_idle = false;
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
