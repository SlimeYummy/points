mod aim;
mod base;
mod dodge;
mod general;
mod guard;
mod idle;
mod r#move;

pub use aim::*;
pub use base::*;
pub use dodge::*;
pub use guard::*;
pub use idle::*;
pub use r#move::*;

use std::rc::Rc;

use crate::template::{TmplAny, TmplClass};
use crate::utils::{CastPtr, KeyCode, XError, XResult, Xrc};

pub(crate) fn try_assemble_action(
    ctx: &mut ContextActionAssemble<'_>,
    tmpl: Xrc<dyn TmplAny>,
) -> XResult<Option<Rc<dyn InstAction>>> {
    let ax: Rc<dyn InstAction> = match tmpl.class() {
        TmplClass::ActionIdle => match InstActionIdle::try_assemble(ctx, unsafe { tmpl.cast_as_unchecked() }) {
            Some(ax) => Rc::new(ax),
            None => return Ok(None),
        },
        TmplClass::ActionMove => match InstActionMove::try_assemble(ctx, unsafe { tmpl.cast_as_unchecked() }) {
            Some(ax) => Rc::new(ax),
            None => return Ok(None),
        },
        TmplClass::ActionDodge => match InstActionDodge::try_assemble(ctx, unsafe { tmpl.cast_as_unchecked() }) {
            Some(ax) => Rc::new(ax),
            None => return Ok(None),
        },
        TmplClass::ActionGuard => match InstActionGuard::try_assemble(ctx, unsafe { tmpl.cast_as_unchecked() }) {
            Some(ax) => Rc::new(ax),
            None => return Ok(None),
        },
        TmplClass::ActionAim => match InstActionAim::try_assemble(ctx, unsafe { tmpl.cast_as_unchecked() }) {
            Some(ax) => Rc::new(ax),
            None => return Ok(None),
        },
        _ => return Err(XError::BadType),
    };
    Ok(Some(ax))
}

pub(crate) fn check_builtin_actions(ctx: &mut ContextActionAssemble<'_>) -> XResult<()> {
    let idle_count = ctx.primary_keys.count(&KeyCode::Idle);
    if idle_count < 1 {
        return Err(XError::bad_action("builtin idle < 1"));
    } else if idle_count >= 2 {
        return Err(XError::bad_action("builtin idle >= 2"));
    }

    let run_count = ctx.primary_keys.count(&KeyCode::Run);
    if run_count < 1 {
        return Err(XError::bad_action("builtin run < 1"));
    } else if run_count >= 2 {
        return Err(XError::bad_action("builtin run >= 2"));
    }

    let dodge_count = ctx.primary_keys.count(&KeyCode::Dodge);
    if dodge_count < 1 {
        return Err(XError::bad_action("builtin dodge < 1"));
    } else if dodge_count >= 2 {
        return Err(XError::bad_action("builtin dodge >= 2"));
    }

    let guard_count = ctx.primary_keys.count(&KeyCode::Guard);
    if guard_count < 1 {
        return Err(XError::bad_action("builtin guard < 1"));
    } else if guard_count >= 2 {
        return Err(XError::bad_action("builtin guard >= 2"));
    }

    Ok(())
}
