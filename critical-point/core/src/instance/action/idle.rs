use crate::instance::action::base::{query_switch, ContextActionAssemble, InstAction, InstActionBase};
use crate::template::{TmplActionIdle, TmplAnimation, TmplType};
use crate::utils::{extend, VirtualKey, Xrc};

#[derive(Debug)]
pub struct InstActionIdle {
    _base: InstActionBase,
    pub tmpl: Xrc<TmplActionIdle>,
    pub derive_level: u16,
    pub antibreak_level: u16,
}

extend!(InstActionIdle, InstActionBase);

unsafe impl InstAction for InstActionIdle {
    fn typ(&self) -> TmplType {
        TmplType::ActionIdle
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a TmplAnimation>) {
        self.tmpl.animations().for_each(|anime| animations.push(anime));
    }
}

impl InstActionIdle {
    pub(crate) fn try_assemble(
        ctx: &mut ContextActionAssemble<'_>,
        tmpl: Xrc<TmplActionIdle>,
    ) -> Option<InstActionIdle> {
        if !query_switch(ctx.args, &tmpl.id, &tmpl.enabled) {
            return None;
        }

        ctx.primary_keys.insert(VirtualKey::Idle, tmpl.id.clone());

        Some(InstActionIdle {
            _base: InstActionBase {
                id: tmpl.id.clone(),
                enter_key: Some(VirtualKey::Idle),
                enter_direction: None,
                enter_level: tmpl.enter_level(),
            },
            tmpl: tmpl.clone(),
            derive_level: tmpl.derive_level(),
            antibreak_level: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_TEMPLATE_PATH;
    use crate::template::TmplDatabase;
    use crate::utils::{sb, DtHashIndex, DtHashMap, IDSymbol};

    #[test]
    fn test_inst_idle_assemble() {
        let db = TmplDatabase::new(TEST_TEMPLATE_PATH).unwrap();

        let mut args = DtHashMap::default();
        let mut primary_keys = DtHashIndex::new();
        let mut derive_keys = DtHashIndex::new();

        {
            let tmpl_act = db.find_as::<TmplActionIdle>(&sb!("Action.No1.Idle")).unwrap();
            let mut ctx = ContextActionAssemble {
                args: &args,
                primary_keys: &mut primary_keys,
                derive_keys: &mut derive_keys,
            };
            let inst_act = InstActionIdle::try_assemble(&mut ctx, tmpl_act).unwrap();
            assert_eq!(inst_act.id, sb!("Action.No1.Idle"));
            assert_eq!(inst_act.enter_key, Some(VirtualKey::Idle));
            assert_eq!(inst_act.enter_level, 0);
            assert_eq!(inst_act.derive_level, 0);
            assert_eq!(inst_act.antibreak_level, 0);
        }

        {
            let tmpl_act = db.find_as::<TmplActionIdle>(&sb!("Action.No1.Idle2")).unwrap();
            let mut ctx = ContextActionAssemble {
                args: &args,
                primary_keys: &mut primary_keys,
                derive_keys: &mut derive_keys,
            };
            let inst_act = InstActionIdle::try_assemble(&mut ctx, tmpl_act);
            assert!(inst_act.is_none());
        }

        {
            let tmpl_act = db.find_as::<TmplActionIdle>(&sb!("Action.No1.Idle2")).unwrap();
            args.insert(IDSymbol::new(&sb!("Action.No1.Idle2"), &sb!("flag")), 1);
            let mut ctx = ContextActionAssemble {
                args: &args,
                primary_keys: &mut primary_keys,
                derive_keys: &mut derive_keys,
            };
            let inst_act = InstActionIdle::try_assemble(&mut ctx, tmpl_act).unwrap();
            assert_eq!(inst_act.id, sb!("Action.No1.Idle2"));
        }
    }
}
