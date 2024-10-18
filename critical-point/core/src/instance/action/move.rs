use crate::instance::action::base::{query_switch, ContextActionAssemble, InstAction, InstActionBase};
use crate::template::{TmplActionMove, TmplAnimation, TmplClass};
use crate::utils::{extend, Xrc};

#[derive(Debug)]
pub struct InstActionMove {
    _base: InstActionBase,
    pub tmpl: Xrc<TmplActionMove>,
    pub derive_level: u16,
    pub antibreak_level: u16,
}

extend!(InstActionMove, InstActionBase);

unsafe impl InstAction for InstActionMove {
    fn class(&self) -> TmplClass {
        TmplClass::ActionMove
    }

    fn get_animations<'a>(&'a self, animations: &mut Vec<&'a TmplAnimation>) {
        self.tmpl.animations().for_each(|anime| animations.push(anime));
    }
}

impl InstActionMove {
    pub(crate) fn try_assemble(
        ctx: &mut ContextActionAssemble<'_>,
        tmpl: Xrc<TmplActionMove>,
    ) -> Option<InstActionMove> {
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

        Some(InstActionMove {
            _base: InstActionBase {
                id: tmpl.id.clone(),
                enter_key: tmpl.enter_key,
                enter_level: tmpl.enter_level(),
            },
            tmpl: tmpl.clone(),
            derive_level: tmpl.derive_level(),
            antibreak_level: tmpl.antibreak_level,
        })
    }
}
