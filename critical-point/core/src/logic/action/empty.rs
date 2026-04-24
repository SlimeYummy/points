use critical_point_csgen::CsOut;
use std::fmt::Debug;
use std::rc::Rc;

use crate::instance::InstActionEmpty;
use crate::logic::action::base::{
    ActionUpdateReturn, ContextAction, LogicActionAny, LogicActionBase, StateActionAny, StateActionBase,
    impl_state_action,
};
use crate::logic::game::ContextUpdate;
use crate::utils::{ActionType, XResult, extend};

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
            _base: StateActionBase::new(ActionType::Empty),
        }
    }
}

impl_state_action!(StateActionEmpty, Empty, "Empty");

#[repr(C)]
#[derive(Debug)]
pub struct LogicActionEmpty {
    _base: LogicActionBase,
}

extend!(LogicActionEmpty, LogicActionBase);

impl LogicActionEmpty {
    pub fn new(ctx: &mut ContextUpdate, inst: Rc<InstActionEmpty>) -> LogicActionEmpty {
        LogicActionEmpty {
            _base: LogicActionBase::new(ctx.gene.gen_action_id(), inst),
        }
    }

    pub fn new_with_id(id: u32, inst: Rc<InstActionEmpty>) -> LogicActionEmpty {
        LogicActionEmpty {
            _base: LogicActionBase::new(id, inst),
        }
    }
}

unsafe impl LogicActionAny for LogicActionEmpty {
    fn typ(&self) -> ActionType {
        ActionType::Empty
    }

    fn save(&self) -> Box<dyn StateActionAny> {
        Box::new(StateActionEmpty {
            _base: self._base.save(self.typ()),
        })
    }

    fn restore(&mut self, _state: &(dyn StateActionAny + 'static)) -> XResult<()> {
        self._base.restore(_state);
        Ok(())
    }

    fn update(&mut self, ctx: &mut ContextUpdate, ctxa: &mut ContextAction) -> XResult<ActionUpdateReturn> {
        self._base.update(ctx, ctxa)?;
        Ok(ActionUpdateReturn::new())
    }
}
