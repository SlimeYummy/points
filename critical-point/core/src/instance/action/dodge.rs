use crate::instance::action::base::{query_index, query_switch, ContextActionAssemble, InstAction, InstActionBase};
use crate::template::{TmplActionDodge, TmplActionDodgeAttribute, TmplAnimation, TmplClass};
use crate::utils::{extend, Xrc};

#[derive(Debug)]
pub struct InstActionDodge {
    _base: InstActionBase,
    pub tmpl: Xrc<TmplActionDodge>,
    pub derive_level: u16,
    pub antibreak_level: u16,
    pub perfect_start: u32,
    pub perfect_duration: u32,
}

extend!(InstActionDodge, InstActionBase);

unsafe impl InstAction for InstActionDodge {
    fn class(&self) -> TmplClass {
        TmplClass::ActionDodge
    }

    fn get_animations<'a>(&'a self, animations: &mut Vec<&'a TmplAnimation>) {
        self.tmpl.animations().for_each(|anime| animations.push(anime));
    }
}

impl InstActionDodge {
    pub(crate) fn try_assemble(
        ctx: &mut ContextActionAssemble<'_>,
        tmpl: Xrc<TmplActionDodge>,
    ) -> Option<InstActionDodge> {
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

        let mut action = InstActionDodge {
            _base: InstActionBase {
                id: tmpl.id.clone(),
                enter_key: tmpl.enter_key,
                enter_level: tmpl.enter_level,
            },
            tmpl: tmpl.clone(),
            derive_level: tmpl.derive_level,
            antibreak_level: tmpl.antibreak_level,
            perfect_start: tmpl.perfect_start,
            perfect_duration: tmpl.perfect_duration,
        };
        for ((arg, attr), vals) in tmpl.attributes.iter() {
            let idx = query_index(ctx.args, &tmpl.id, arg);
            match attr {
                TmplActionDodgeAttribute::EnterLevel => action.enter_level = vals[idx as usize] as u16,
                TmplActionDodgeAttribute::PerfectStart => action.perfect_start = vals[idx as usize],
                TmplActionDodgeAttribute::PerfectDuration => action.perfect_duration = vals[idx as usize],
            }
        }

        Some(action)
    }
}
