use std::rc::Rc;

use critical_point_macros::{csharp_enum, csharp_out};

use crate::consts::MAX_ACTION_ANIMATION;
use crate::instance::InstActionMoveTowardNpc;
use crate::logic::action::base::{
    ActionStartArgs, ActionStartReturn, ActionUpdateReturn, ContextAction, LogicActionAny, LogicActionBase,
    StateActionAnimation, StateActionAny, StateActionBase, impl_state_action,
};
use crate::logic::game::ContextUpdateEx;
use crate::utils::{ActionType, Castable, LEVEL_MOVE, XResult, extend, xresf};

#[csharp_enum]
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
)]
#[rkyv(derive(Debug))]
pub enum ActionMoveTowardNpcMode {
    // TODO: Define specific modes for toward-based movement
    Placeholder = 0,
}

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, PartialEq, rkyv::Archive, serde::Serialize, serde::Deserialize, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct StateActionMoveTowardNpc {
    pub _base: StateActionBase,
    // TODO: Define state fields for toward-based movement
}

extend!(StateActionMoveTowardNpc, StateActionBase);
impl_state_action!(StateActionMoveTowardNpc, MoveTowardNpc, "MoveTowardNpc");

///
/// Toward move action logic.
/// Applicable to NPC movement controlled by programs or AI.
///
/// Movement logic differs from free movement and will be implemented based on direction-specific navigation.
///
#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionMoveTowardNpc {
    _base: LogicActionBase,
    inst: Rc<InstActionMoveTowardNpc>,
    prev_anim_queue: Vec<StateActionAnimation>,
}

extend!(LogicActionMoveTowardNpc, LogicActionBase);

impl LogicActionMoveTowardNpc {
    pub fn new(ctx: &mut ContextUpdateEx, inst_act: Rc<InstActionMoveTowardNpc>) -> XResult<LogicActionMoveTowardNpc> {
        Ok(LogicActionMoveTowardNpc {
            _base: LogicActionBase {
                keep_level: LEVEL_MOVE,
                poise_level: inst_act.poise_level,
                ..LogicActionBase::new(ctx.identity.gen_action_id(), inst_act.clone())
            },
            inst: inst_act,
            prev_anim_queue: Vec::new(),
        })
    }
}

unsafe impl LogicActionAny for LogicActionMoveTowardNpc {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::MoveTowardNpc
    }

    fn restore(&mut self, state: &(dyn StateActionAny + 'static)) -> XResult<()> {
        if state.id != self._base.id {
            return xresf!(LogicIDMismatch; "state.id={}, self.id={}", state.id, self._base.id);
        }
        let state = state.cast::<StateActionMoveTowardNpc>()?;

        self._base.restore(&state._base);

        self.prev_anim_queue.clear();
        let prev_len = state.animations.len().saturating_sub(1);
        for anim in state.animations.iter().take(prev_len) {
            self.prev_anim_queue.push(anim.clone());
        }
        Ok(())
    }

    fn save(&self) -> Box<dyn StateActionAny> {
        let mut state = Box::new(StateActionMoveTowardNpc {
            _base: self._base.save(self.typ()),
        });

        debug_assert!(self.prev_anim_queue.len() + 1 <= MAX_ACTION_ANIMATION);
        state.animations.extend(self.prev_anim_queue.iter().cloned());
        state.animations.push(StateActionAnimation::default());
        state
    }

    fn start(
        &mut self,
        ctx: &mut ContextUpdateEx,
        ctxa: &mut ContextAction,
        args: &ActionStartArgs,
    ) -> XResult<ActionStartReturn> {
        self._base.start(ctx, ctxa, args)?;
        // TODO: Initialize toward-specific start logic
        Ok(ActionStartReturn::new())
    }

    fn update(&mut self, ctx: &mut ContextUpdateEx, ctxa: &mut ContextAction) -> XResult<ActionUpdateReturn> {
        self._base.update(ctx, ctxa)?;

        // TODO: Implement toward-based movement logic here
        // For now, return neutral result with no movement
        let ret = ActionUpdateReturn::new(ctxa.chara_phy.direction_xz());
        Ok(ret)
    }
}
