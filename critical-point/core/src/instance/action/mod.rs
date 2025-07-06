// mod aim;
mod base;
// mod dodge;
mod empty;
mod general;
// mod guard;
mod idle;
mod r#move;

// pub use aim::*;
pub use base::*;
// pub use dodge::*;
pub use empty::*;
pub use general::*;
// pub use guard::*;
pub use idle::*;
pub use r#move::*;

use std::rc::Rc;

use crate::template::{At, TmplAny, TmplType};
use crate::utils::{xres, DtHashIndex, DtHashMap, TmplID, VirtualKey, XResult};

pub(crate) fn try_assemble_action(
    ctx: &ContextActionAssemble<'_>,
    tmpl: At<dyn TmplAny>,
) -> XResult<Option<Rc<dyn InstActionAny>>> {
    let ax: Rc<dyn InstActionAny> = match tmpl.typ() {
        TmplType::ActionIdle => match InstActionIdle::try_assemble(ctx, unsafe { tmpl.cast_unchecked() }) {
            Some(ax) => Rc::new(ax),
            None => return Ok(None),
        },
        TmplType::ActionMove => match InstActionMove::try_assemble(ctx, unsafe { tmpl.cast_unchecked() }) {
            Some(ax) => Rc::new(ax),
            None => return Ok(None),
        },
        TmplType::ActionGeneral => match InstActionGeneral::try_assemble(ctx, unsafe { tmpl.cast_unchecked() }) {
            Some(ax) => Rc::new(ax),
            None => return Ok(None),
        },
        // TmplType::ActionDodge => match InstActionDodge::try_assemble(ctx, unsafe { tmpl.cast_as_unchecked() }) {
        //     Some(ax) => Rc::new(ax),
        //     None => return Ok(None),
        // },
        // TmplType::ActionGuard => match InstActionGuard::try_assemble(ctx, unsafe { tmpl.cast_as_unchecked() }) {
        //     Some(ax) => Rc::new(ax),
        //     None => return Ok(None),
        // },
        // TmplType::ActionAim => match InstActionAim::try_assemble(ctx, unsafe { tmpl.cast_as_unchecked() }) {
        //     Some(ax) => Rc::new(ax),
        //     None => return Ok(None),
        // },
        _ => return xres!(BadType),
    };

    Ok(Some(ax))
}

pub(crate) fn collect_action_keys(
    actions: &DtHashMap<TmplID, Rc<dyn InstActionAny>>,
) -> XResult<(
    DtHashIndex<VirtualKey, TmplID>,
    DtHashIndex<(TmplID, VirtualKey), TmplID>,
)> {
    let mut primary_rules: DtHashIndex<VirtualKey, TmplID> = DtHashIndex::new();
    let mut derive_rules: DtHashIndex<(TmplID, VirtualKey), TmplID> = DtHashIndex::new();

    let mut tmp_derives: Vec<(VirtualKey, TmplID)> = Vec::new();
    for (act_id, act) in actions {
        if let Some(enter_key) = act.enter_key {
            primary_rules.insert(enter_key.key, *act_id);
        }

        act.derives(&mut tmp_derives);
        for (key, derive_id) in tmp_derives.drain(..) {
            derive_rules.insert((*act_id, key), derive_id);
        }
    }

    // let idle_count = primary_rules.count(&VirtualKey::Idle);
    // if idle_count < 1 {
    //     return xres!(BadAction; "idle");
    // } else if idle_count >= 2 {
    //     return xres!(BadAction; "idle");
    // }

    // let run_count = primary_rules.count(&VirtualKey::Run);
    // if run_count < 1 {
    //     return xres!(BadAction; "run");
    // } else if run_count >= 2 {
    //     return xres!(BadAction; "run");
    // }

    // let dodge_count = primary_rules.count(&VirtualKey::Dodge);
    // if dodge_count < 1 {
    //     return xres!(BadAction; "dodge");
    // } else if dodge_count >= 2 {
    //     return xres!(BadAction; "dodge");
    // }

    // let guard_count = primary_rules.count(&VirtualKey::Guard);
    // if guard_count < 1 {
    //     return xres!(BadAction; "guard");
    // } else if guard_count >= 2 {
    //     return xres!(BadAction; "guard");
    // }

    Ok((primary_rules, derive_rules))
}
