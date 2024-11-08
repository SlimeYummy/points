mod base;
mod idle;
mod r#move;

pub use base::*;
pub use idle::*;

use std::rc::Rc;

use crate::instance::InstAction;
use crate::logic::game::ContextUpdate;
use crate::template::TmplClass;
use crate::utils::{CastPtr, CastRef, XError, XResult};

pub(crate) fn new_logic_action(
    ctx: &mut ContextUpdate<'_>,
    inst_act: Rc<dyn InstAction>,
) -> XResult<Box<dyn LogicAction>> {
    use TmplClass::*;

    let logic_act: Box<dyn LogicAction> = match inst_act.class() {
        ActionIdle => {
            let inst_act = unsafe { inst_act.cast_as_unchecked() };
            Box::new(LogicActionIdle::new(ctx, inst_act)?)
        }
        _ => return Err(XError::BadType),
    };
    Ok(logic_act)
}

pub(crate) fn try_reuse_logic_action(
    logic_act: &mut Box<dyn LogicAction>,
    ctx: &mut ContextUpdate<'_>,
    inst_act: Rc<dyn InstAction>,
) -> XResult<bool> {
    use TmplClass::*;

    match inst_act.class() {
        ActionIdle => {
            if let Ok(logic_act) = logic_act.cast_mut::<LogicActionIdle>() {
                let inst_act = unsafe { inst_act.cast_as_unchecked() };
                *logic_act = LogicActionIdle::new(ctx, inst_act)?;
                return Ok(true);
            }
        }
        _ => return Ok(false),
    }
    Ok(false)
}
