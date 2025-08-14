mod base;
mod empty;
mod general;
mod idle;
mod r#move;
mod root_motion;
#[cfg(test)]
mod test_utils;

pub use base::*;
pub use empty::*;
pub use general::*;
pub use idle::*;
pub use r#move::*;
pub use root_motion::*;

use std::rc::Rc;

use crate::instance::InstActionAny;
use crate::logic::game::ContextUpdate;
use crate::template::TmplType;
use crate::utils::{xres, Castable, XResult};

pub(crate) fn new_logic_action(
    ctx: &mut ContextUpdate<'_>,
    inst_act: Rc<dyn InstActionAny + 'static>,
) -> XResult<Box<dyn LogicActionAny + 'static>> {
    use TmplType::*;

    let logic_act: Box<dyn LogicActionAny> = match inst_act.typ() {
        ActionEmpty => {
            let inst_act = unsafe { inst_act.cast_unchecked() };
            Box::new(LogicActionEmpty::new(ctx, inst_act))
        }
        ActionIdle => {
            let inst_act = unsafe { inst_act.cast_unchecked() };
            Box::new(LogicActionIdle::new(ctx, inst_act)?)
        }
        ActionMove => {
            let inst_act = unsafe { inst_act.cast_unchecked() };
            Box::new(LogicActionMove::new(ctx, inst_act)?)
        }
        ActionGeneral => {
            let inst_act = unsafe { inst_act.cast_unchecked() };
            Box::new(LogicActionGeneral::new(ctx, inst_act)?)
        }
        _ => return xres!(BadType),
    };
    Ok(logic_act)
}

pub(crate) fn try_reuse_logic_action(
    logic_act: &mut Box<dyn LogicActionAny>,
    ctx: &mut ContextUpdate<'_>,
    inst_act: Rc<dyn InstActionAny>,
) -> XResult<bool> {
    use TmplType::*;

    match inst_act.typ() {
        ActionEmpty => {
            if let Ok(logic_act) = logic_act.cast::<LogicActionEmpty>() {
                let inst_act = unsafe { inst_act.cast_unchecked() };
                *logic_act = LogicActionEmpty::new(ctx, inst_act);
                return Ok(true);
            }
        }
        ActionIdle => {
            if let Ok(logic_act) = logic_act.cast::<LogicActionIdle>() {
                let inst_act = unsafe { inst_act.cast_unchecked() };
                *logic_act = LogicActionIdle::new(ctx, inst_act)?;
                return Ok(true);
            }
        }
        ActionMove => {
            if let Ok(logic_act) = logic_act.cast::<LogicActionMove>() {
                let inst_act = unsafe { inst_act.cast_unchecked() };
                *logic_act = LogicActionMove::new(ctx, inst_act)?;
                return Ok(true);
            }
        }
        ActionGeneral => {
            if let Ok(logic_act) = logic_act.cast::<LogicActionGeneral>() {
                let inst_act = unsafe { inst_act.cast_unchecked() };
                *logic_act = LogicActionGeneral::new(ctx, inst_act)?;
                return Ok(true);
            }
        }
        _ => return Ok(false),
    }
    Ok(false)
}
