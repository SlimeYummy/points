use std::fmt::Debug;
use std::rc::Rc;

use crate::instance::{InstAction, InstActionMove};
use crate::logic::action::base::{
    ArchivedStateAction, ContextActionUpdate, LogicAction, LogicActionBase, StateAction, StateActionBase,
};
use crate::logic::game::ContextUpdate;
use crate::template::{TmplActionMove, TmplClass};
use crate::utils::{extend, XResult};

#[repr(C)]
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive_attr(derive(Debug))]
pub struct StateActionMove {
    pub _base: StateActionBase,
    pub event_no: u64,
}

extend!(StateActionMove, StateActionBase);

unsafe impl StateAction for StateActionMove {
    #[inline]
    fn class(&self) -> TmplClass {
        TmplClass::ActionMove
    }
}

impl ArchivedStateAction for rkyv::Archived<StateActionMove> {
    fn class(&self) -> TmplClass {
        TmplClass::ActionMove
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionMove {
    _base: LogicActionBase,
    tmpl: Rc<TmplActionMove>,
    inst: Rc<InstActionMove>,

    event_no: u64,
}

extend!(LogicActionMove, LogicActionBase);

unsafe impl LogicAction for LogicActionMove {
    #[inline]
    fn class(&self) -> TmplClass {
        TmplClass::ActionMove
    }

    #[inline]
    fn restore(&mut self, state: &(dyn StateAction + 'static)) -> XResult<()> {
        unimplemented!()
    }

    #[inline]
    fn next(
        &mut self,
        ctx: &mut ContextUpdate<'_>,
        ctx_an: &super::ContextActionNext,
    ) -> XResult<Option<Rc<dyn InstAction>>> {
        unimplemented!()
    }

    #[inline]
    fn update(&mut self, ctx: &mut ContextUpdate<'_>, ctx_au: &mut ContextActionUpdate<'_>) -> XResult<()> {
        unimplemented!()
    }
}
