mod base;
mod idle;
mod r#move;

pub use base::*;
pub use idle::*;
pub use r#move::*;

use std::rc::Rc;

use crate::instance::InstAction;
use crate::logic::game::ContextUpdate;
use crate::template::TmplType;
use crate::utils::{xres, CastPtr, CastRef, XResult};

pub(crate) fn new_logic_action(
    ctx: &mut ContextUpdate<'_>,
    inst_act: Rc<dyn InstAction + 'static>,
) -> XResult<Box<dyn LogicAction + 'static>> {
    use TmplType::*;

    let logic_act: Box<dyn LogicAction> = match inst_act.typ() {
        ActionIdle => {
            let inst_act = unsafe { inst_act.cast_as_unchecked() };
            Box::new(LogicActionIdle::new(ctx, inst_act)?)
        }
        ActionMove => {
            let inst_act = unsafe { inst_act.cast_as_unchecked() };
            Box::new(LogicActionMove::new(ctx, inst_act)?)
        }
        _ => return xres!(BadType),
    };
    Ok(logic_act)
}

pub(crate) fn try_reuse_logic_action(
    logic_act: &mut Box<dyn LogicAction>,
    ctx: &mut ContextUpdate<'_>,
    inst_act: Rc<dyn InstAction>,
) -> XResult<bool> {
    use TmplType::*;

    match inst_act.typ() {
        ActionIdle => {
            if let Ok(logic_act) = logic_act.cast_mut::<LogicActionIdle>() {
                let inst_act = unsafe { inst_act.cast_as_unchecked() };
                *logic_act = LogicActionIdle::new(ctx, inst_act)?;
                return Ok(true);
            }
        }
        ActionMove => {
            if let Ok(logic_act) = logic_act.cast_mut::<LogicActionMove>() {
                let inst_act = unsafe { inst_act.cast_as_unchecked() };
                *logic_act = LogicActionMove::new(ctx, inst_act)?;
                return Ok(true);
            }
        }
        _ => return Ok(false),
    }
    Ok(false)
}
