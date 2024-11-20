use crate::instance::action::base::{query_index, query_switch, ContextActionAssemble, InstAction, InstActionBase};
use crate::template::{TmplActionAim, TmplActionAimAttribute, TmplAnimation, TmplType};
use crate::utils::{extend, Xrc};

#[derive(Debug)]
pub struct InstActionAim {
    _base: InstActionBase,
    pub tmpl: Xrc<TmplActionAim>,
    pub derive_level: u16,
    pub antibreak_level: u16,
    pub aim_start: u32,
}

extend!(InstActionAim, InstActionBase);

unsafe impl InstAction for InstActionAim {
    fn typ(&self) -> TmplType {
        TmplType::ActionAim
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a TmplAnimation>) {
        self.tmpl.animations().for_each(|anime| animations.push(anime));
    }
}

impl InstActionAim {
    pub(crate) fn try_assemble(ctx: &mut ContextActionAssemble<'_>, tmpl: Xrc<TmplActionAim>) -> Option<InstActionAim> {
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

        let mut action = InstActionAim {
            _base: InstActionBase {
                id: tmpl.id.clone(),
                enter_key: tmpl.enter_key,
                enter_level: tmpl.enter_level,
            },
            tmpl: tmpl.clone(),
            derive_level: tmpl.derive_level,
            antibreak_level: tmpl.antibreak_level,
            aim_start: tmpl.aim_start,
        };
        for ((arg, attr), vals) in tmpl.attributes.iter() {
            let idx = query_index(ctx.args, &tmpl.id, arg);
            match attr {
                TmplActionAimAttribute::AimStart => action.aim_start += vals[idx as usize],
            }
        }

        Some(action)
    }
}
