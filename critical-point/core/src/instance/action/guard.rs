use crate::instance::action::base::{query_index, query_switch, ContextActionAssemble, InstAction, InstActionBase};
use crate::template::{TmplActionGuard, TmplActionGuardAttribute, TmplAnimation, TmplType};
use crate::utils::{extend, Xrc};

#[derive(Debug)]
pub struct InstActionGuard {
    _base: InstActionBase,
    pub tmpl: Xrc<TmplActionGuard>,
    pub derive_level: u16,
    pub antibreak_level: u16,
    pub guard_start: u32,
    pub perfect_start: u32,
    pub perfect_duration: u32,
}

extend!(InstActionGuard, InstActionBase);

unsafe impl InstAction for InstActionGuard {
    fn typ(&self) -> TmplType {
        TmplType::ActionGuard
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a TmplAnimation>) {
        self.tmpl.animations().for_each(|anime| animations.push(anime));
    }
}

impl InstActionGuard {
    pub(crate) fn try_assemble(
        ctx: &mut ContextActionAssemble<'_>,
        tmpl: Xrc<TmplActionGuard>,
    ) -> Option<InstActionGuard> {
        if !query_switch(ctx.args, &tmpl.id, &tmpl.enabled) {
            return None;
        }

        if let Some(enter_key) = tmpl.enter_key {
            ctx.primary_keys.insert(enter_key, tmpl.id.clone());
        }
        for derive in tmpl.derives.iter() {
            if query_switch(ctx.args, &tmpl.id, &derive.enabled) {
                ctx.derive_keys
                    .insert((tmpl.id.clone(), derive.key), derive.action.clone());
            }
        }

        let mut action = InstActionGuard {
            _base: InstActionBase {
                id: tmpl.id.clone(),
                enter_key: tmpl.enter_key,
                enter_direction: None,
                enter_level: tmpl.enter_level,
            },
            tmpl: tmpl.clone(),
            derive_level: tmpl.derive_level,
            antibreak_level: tmpl.antibreak_level(),
            guard_start: tmpl.guard_start,
            perfect_start: tmpl.perfect_start,
            perfect_duration: tmpl.perfect_duration,
        };
        for ((arg, attr), vals) in tmpl.attributes.iter() {
            let idx = query_index(ctx.args, &tmpl.id, arg);
            match attr {
                TmplActionGuardAttribute::EnterLevel => action.enter_level = vals[idx as usize] as u16,
                TmplActionGuardAttribute::PerfectStart => action.perfect_start = vals[idx as usize],
                TmplActionGuardAttribute::PerfectDuration => action.perfect_duration = vals[idx as usize],
            }
        }

        Some(action)
    }
}
