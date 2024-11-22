use cirtical_point_csgen::{CsEnum, CsOut};
use std::fmt::Debug;
use std::rc::Rc;

use crate::consts::WEIGHT_THRESHOLD;
use crate::instance::{InstAction, InstActionMove};
use crate::logic::action::base::{
    ArchivedStateAction, ContextActionNext, ContextActionUpdate, LogicAction, LogicActionBase, StateAction,
    StateActionBase, StateActionType,
};
use crate::logic::game::ContextUpdate;
use crate::template::{TmplActionMove, TmplType};
use crate::utils::{calc_ratio, extend, CastRef, KeyCode, KeyEvent, XError, XResult};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsEnum)]
#[archive_attr(derive(Debug))]
pub enum ActionMoveMode {
    None,
    Move,
    TurnLeft,
    TurnRight,
    YamLeft,
    YamRight,
    Stop,
}

#[repr(C)]
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateActionMove {
    pub _base: StateActionBase,
    pub mode: ActionMoveMode,
    pub switch_progress: u32,
    pub previous_progress: u32,
    pub current_progress: u32,
}

extend!(StateActionMove, StateActionBase);

unsafe impl StateAction for StateActionMove {
    #[inline]
    fn typ(&self) -> StateActionType {
        assert!(self.typ == StateActionType::Move);
        StateActionType::Move
    }

    #[inline]
    fn tmpl_typ(&self) -> TmplType {
        assert!(self.tmpl_typ == TmplType::ActionMove);
        TmplType::ActionMove
    }
}

impl ArchivedStateAction for rkyv::Archived<StateActionMove> {
    #[inline]
    fn typ(&self) -> StateActionType {
        StateActionType::Move
    }

    #[inline]
    fn tmpl_typ(&self) -> TmplType {
        TmplType::ActionMove
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionMove {
    _base: LogicActionBase,
    tmpl: Rc<TmplActionMove>,
    inst: Rc<InstActionMove>,

    pub mode: ActionMoveMode,
    pub switch_progress: u32,
    pub previous_progress: u32,
    pub current_progress: u32,
}

extend!(LogicActionMove, LogicActionBase);

unsafe impl LogicAction for LogicActionMove {
    #[inline]
    fn typ(&self) -> StateActionType {
        StateActionType::Move
    }

    #[inline]
    fn tmpl_typ(&self) -> TmplType {
        TmplType::ActionMove
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

impl LogicActionMove {
    pub fn new(ctx: &mut ContextUpdate<'_>, inst_act: Rc<InstActionMove>) -> XResult<LogicActionMove> {
        Ok(LogicActionMove {
            _base: LogicActionBase {
                derive_level: inst_act.derive_level,
                antibreak_level: inst_act.antibreak_level,
                ..LogicActionBase::new(ctx.gene.gen_id(), inst_act.id.clone(), ctx.frame)
            },
            tmpl: inst_act.tmpl.clone(),
            inst: inst_act,

            mode: ActionMoveMode::None,
            switch_progress: 0,
            previous_progress: 0,
            current_progress: 0,
        })
    }

    fn restore_impl(&mut self, state: &(dyn StateAction + 'static)) -> XResult<()> {
        if state.id != self._base.id {
            return Err(XError::IDMissMatch);
        }
        let state = state.cast_ref::<StateActionMove>()?;

        self._base.restore(&state._base);
        self.mode = state.mode;
        self.switch_progress = state.switch_progress;
        self.previous_progress = state.previous_progress;
        self.current_progress = state.current_progress;
        Ok(())
    }

    fn save(&self) -> Box<StateActionMove> {
        Box::new(StateActionMove {
            _base: self._base.save(self.typ(), self.tmpl_typ()),
            mode: self.mode,
            switch_progress: self.switch_progress,
            previous_progress: self.previous_progress,
            current_progress: self.current_progress,
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

        if self.is_leaving {
            self.mode = ActionMoveMode::Stop;
            return Ok(());
        }

        match self.mode {
            ActionMoveMode::None => {
                if let Some(event) = ctx.input.player_enter_event(ctx_au.player_id)? {
                    if event.key == KeyCode::Run {
                    } else {
                        self.mode = ActionMoveMode::Move;
                    }
                } else {
                    self.mode = ActionMoveMode::Move;
                }
            }
            ActionMoveMode::Move => {}
            ActionMoveMode::TurnLeft => {}
            ActionMoveMode::TurnRight => {}
            ActionMoveMode::YamLeft => {}
            ActionMoveMode::YamRight => {}
            ActionMoveMode::Stop => {}
        }
        return Ok(());
    }
}
