use critical_point_csgen::CsOut;
use std::fmt::Debug;
use std::rc::Rc;

use crate::instance::InstActionEmpty;
use crate::logic::action::base::{
    impl_state_action, ActionUpdateReturn, ContextAction, LogicActionAny, LogicActionBase, StateActionAny,
    StateActionBase, StateActionType,
};
use crate::logic::game::ContextUpdate;
use crate::template::TmplType;
use crate::utils::{extend, NumID, XResult};

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateActionEmpty {
    pub _base: StateActionBase,
}

extend!(StateActionEmpty, StateActionBase);

impl Default for StateActionEmpty {
    fn default() -> Self {
        StateActionEmpty {
            _base: StateActionBase::new(StateActionType::Empty, TmplType::ActionEmpty),
        }
    }
}

impl_state_action!(StateActionEmpty, ActionEmpty, Empty, "Empty");

#[repr(C)]
#[derive(Debug)]
pub struct LogicActionEmpty {
    _base: LogicActionBase,
}

extend!(LogicActionEmpty, LogicActionBase);

impl LogicActionEmpty {
    pub fn new(ctx: &mut ContextUpdate<'_>, inst: Rc<InstActionEmpty>) -> LogicActionEmpty {
        LogicActionEmpty {
            _base: LogicActionBase::new(ctx.gene.gen_id(), inst),
        }
    }

    pub fn new_with_id(id: NumID, inst: Rc<InstActionEmpty>) -> LogicActionEmpty {
        LogicActionEmpty {
            _base: LogicActionBase::new(id, inst),
        }
    }
}

unsafe impl LogicActionAny for LogicActionEmpty {
    fn typ(&self) -> StateActionType {
        StateActionType::Empty
    }

    fn tmpl_typ(&self) -> TmplType {
        TmplType::ActionEmpty
    }

    fn save(&self) -> Box<dyn StateActionAny> {
        Box::new(StateActionEmpty {
            _base: self._base.save(self.typ(), self.tmpl_typ()),
        })
    }

    fn restore(&mut self, _state: &(dyn StateActionAny + 'static)) -> XResult<()> {
        self._base.restore(_state);
        Ok(())
    }

    fn update(&mut self, ctx: &mut ContextUpdate<'_>, ctxa: &mut ContextAction<'_, '_>) -> XResult<ActionUpdateReturn> {
        self._base.update(ctx, ctxa)?;
        Ok(ActionUpdateReturn::new())
    }
}
