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

use crate::template::{TmplAny, TmplType};
use crate::utils::{xres, CastPtr, VirtualKey, XResult, Xrc};

pub(crate) fn try_assemble_action(
    ctx: &mut ContextActionAssemble<'_>,
    tmpl: Xrc<dyn TmplAny>,
) -> XResult<Option<Rc<dyn InstAction>>> {
    let ax: Rc<dyn InstAction> = match tmpl.typ() {
        TmplType::ActionIdle => match InstActionIdle::try_assemble(ctx, unsafe { tmpl.cast_as_unchecked() }) {
            Some(ax) => Rc::new(ax),
            None => return Ok(None),
        },
        TmplType::ActionMove => match InstActionMove::try_assemble(ctx, unsafe { tmpl.cast_as_unchecked() }) {
            Some(ax) => Rc::new(ax),
            None => return Ok(None),
        },
        TmplType::ActionDodge => match InstActionDodge::try_assemble(ctx, unsafe { tmpl.cast_as_unchecked() }) {
            Some(ax) => Rc::new(ax),
            None => return Ok(None),
        },
        TmplType::ActionGuard => match InstActionGuard::try_assemble(ctx, unsafe { tmpl.cast_as_unchecked() }) {
            Some(ax) => Rc::new(ax),
            None => return Ok(None),
        },
        TmplType::ActionAim => match InstActionAim::try_assemble(ctx, unsafe { tmpl.cast_as_unchecked() }) {
            Some(ax) => Rc::new(ax),
            None => return Ok(None),
        },
        _ => return xres!(BadType),
    };
    Ok(Some(ax))
}

pub(crate) fn check_builtin_actions(ctx: &mut ContextActionAssemble<'_>) -> XResult<()> {
    let idle_count = ctx.primary_keys.count(&VirtualKey::Idle);
    if idle_count < 1 {
        return xres!(BadAction; "idle");
    } else if idle_count >= 2 {
        return xres!(BadAction; "idle");
    }

    let run_count = ctx.primary_keys.count(&VirtualKey::Run);
    if run_count < 1 {
        return xres!(BadAction; "run");
    } else if run_count >= 2 {
        return xres!(BadAction; "run");
    }

    let dodge_count = ctx.primary_keys.count(&VirtualKey::Dodge);
    if dodge_count < 1 {
        return xres!(BadAction; "dodge");
    } else if dodge_count >= 2 {
        return xres!(BadAction; "dodge");
    }

    let guard_count = ctx.primary_keys.count(&VirtualKey::Guard);
    if guard_count < 1 {
        return xres!(BadAction; "guard");
    } else if guard_count >= 2 {
        return xres!(BadAction; "guard");
    }

    Ok(())
}
