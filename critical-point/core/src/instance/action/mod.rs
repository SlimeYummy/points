// mod aim;
mod base;
// mod dodge;
mod dodge_npc;
mod empty;
mod general;
mod general_npc;
// mod guard;
mod hit;
mod idle;
mod move_free;
mod move_free_npc;
mod move_toward_npc;

// pub use aim::*;
pub use base::*;
// pub use dodge::*;
pub use dodge_npc::*;
pub use empty::*;
pub use general::*;
pub use general_npc::*;
// pub use guard::*;
pub use hit::*;
pub use idle::*;
pub use move_free::*;
pub use move_free_npc::*;
pub use move_toward_npc::*;

use std::rc::Rc;

use crate::template::{At, TmplAny, TmplType};
use crate::utils::{DtHashIndex, DtHashMap, TmplID, VirtualKey, XResult, xres};

pub(crate) fn assemble_action(
    ctx: &ContextActionAssemble<'_>,
    tmpl: At<dyn TmplAny>,
) -> XResult<Option<Rc<dyn InstActionAny>>> {
    let act: Rc<dyn InstActionAny> = match tmpl.typ() {
        TmplType::ActionIdle => match InstActionIdle::new_from_action(ctx, unsafe { tmpl.cast_unchecked() }) {
            Some(act) => Rc::new(act),
            None => return Ok(None),
        },
        TmplType::ActionMoveFree => match InstActionMoveFree::new_from_action(ctx, unsafe { tmpl.cast_unchecked() }) {
            Some(act) => Rc::new(act),
            None => return Ok(None),
        },
        TmplType::ActionMoveFreeNpc => {
            match InstActionMoveFreeNpc::new_from_action(ctx, unsafe { tmpl.cast_unchecked() }) {
                Some(act) => Rc::new(act),
                None => return Ok(None),
            }
        }
        TmplType::ActionMoveTowardNpc => {
            match InstActionMoveTowardNpc::new_from_action(ctx, unsafe { tmpl.cast_unchecked() }) {
                Some(act) => Rc::new(act),
                None => return Ok(None),
            }
        }
        TmplType::ActionGeneral => match InstActionGeneral::new_from_action(ctx, unsafe { tmpl.cast_unchecked() })? {
            Some(act) => Rc::new(act),
            None => return Ok(None),
        },
        TmplType::ActionGeneralNpc => {
            match InstActionGeneralNpc::new_from_action(ctx, unsafe { tmpl.cast_unchecked() })? {
                Some(act) => Rc::new(act),
                None => return Ok(None),
            }
        }
        TmplType::ActionDodgeNpc => match InstActionDodgeNpc::new_from_action(ctx, unsafe { tmpl.cast_unchecked() })? {
            Some(act) => Rc::new(act),
            None => return Ok(None),
        },
        // TmplType::ActionDodge => match InstActionDodge::new_from_action(ctx, unsafe { tmpl.cast_unchecked() }) {
        //     Some(act) => Rc::new(act),
        //     None => return Ok(None),
        // },
        // TmplType::ActionGuard => match InstActionGuard::new_from_action(ctx, unsafe { tmpl.cast_unchecked() }) {
        //     Some(act) => Rc::new(act),
        //     None => return Ok(None),
        // },
        // TmplType::ActionAim => match InstActionAim::new_from_action(ctx, unsafe { tmpl.cast_unchecked() }) {
        //     Some(act) => Rc::new(act),
        //     None => return Ok(None),
        // },
        TmplType::ActionHit => match InstActionHit::new_from_action(ctx, unsafe { tmpl.cast_unchecked() })? {
            Some(act) => Rc::new(act),
            None => return Ok(None),
        },
        _ => return xres!(BadType),
    };

    Ok(Some(act))
}

pub(crate) fn collect_action_keys(
    actions: &DtHashMap<TmplID, Rc<dyn InstActionAny>>,
    collect_derive: bool,
) -> XResult<(
    DtHashIndex<VirtualKey, TmplID>,
    DtHashIndex<(TmplID, VirtualKey), InstDeriveRule>,
)> {
    let mut primary_rules: DtHashIndex<VirtualKey, TmplID> = DtHashIndex::new();
    let mut derive_rules: DtHashIndex<(TmplID, VirtualKey), InstDeriveRule> = DtHashIndex::new();

    let mut tmp_rules: Vec<InstDeriveRule> = Vec::new();
    for (act_id, act) in actions {
        if let Some(enter_key) = act.enter_key {
            primary_rules.insert(enter_key.key, *act_id);
        }

        if collect_derive {
            act.derives(&mut tmp_rules);
            for rule in tmp_rules.drain(..) {
                derive_rules.insert((*act_id, rule.key), rule);
            }
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
