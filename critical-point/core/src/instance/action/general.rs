use crate::instance::action::base::{
    calc_motion_distance_ratio, ContextActionAssemble, InstActionAny, InstActionAttributes, InstActionBase,
    InstAnimation, InstDeriveRule, InstTimeline,
};
use crate::template::{At, TmplActionGeneral, TmplType};
use crate::utils::{cos_degree, extend, Bitsetable, DeriveContinue, EnumBitset, TmplID, VirtualKey};

#[repr(C)]
#[derive(Debug)]
pub struct InstActionGeneral {
    pub _base: InstActionBase,
    pub anim_main: InstAnimation,
    pub attributes: InstTimeline<InstActionAttributes>,
    pub motion_distance_ratio: [f32; 2],
    pub motion_toward_cos: f32,
    pub derive_levels: InstTimeline<u16>,
    pub derives: Vec<InstDeriveRule>,
    pub derive_continues: EnumBitset<DeriveContinue, { DeriveContinue::LEN }>,
}

extend!(InstActionGeneral, InstActionBase);

unsafe impl InstActionAny for InstActionGeneral {
    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::ActionGeneral
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>) {
        self.animations().for_each(|animation| animations.push(animation));
    }

    fn derives(&self, derive_keys: &mut Vec<(VirtualKey, TmplID)>) {
        for rule in self.derives.iter() {
            derive_keys.push((rule.key, rule.action));
        }
    }
}

impl InstActionGeneral {
    pub(crate) fn try_assemble(
        ctx: &ContextActionAssemble<'_>,
        tmpl: At<TmplActionGeneral>,
    ) -> Option<InstActionGeneral> {
        if !ctx.solve_var(&tmpl.enabled) {
            return None;
        }

        let mut derives = Vec::with_capacity(tmpl.derives.len());
        for rule in tmpl.derives.iter() {
            let rule = InstDeriveRule::from_rkyv(ctx, rule);
            if rule.action.is_valid() {
                derives.push(rule);
            }
        }

        let attributes = InstTimeline::from_rkyv(&tmpl.attributes, |archived| {
            InstActionAttributes::from_rkyv(ctx, archived)
        });
        let derive_levels = InstTimeline::from_rkyv(&tmpl.derive_levels, |level| ctx.solve_var(level).into());

        let inst = InstActionGeneral {
            _base: InstActionBase {
                tmpl_id: tmpl.id,
                enter_key: tmpl.enter_key.as_ref().cloned(),
                enter_level: tmpl.enter_level.into(),
                ..Default::default()
            },
            derives,
            anim_main: InstAnimation::from_rkyv(&tmpl.anim_main),
            motion_distance_ratio: calc_motion_distance_ratio(tmpl.motion_distance, &tmpl.anim_main),
            motion_toward_cos: cos_degree(tmpl.motion_toward.into()),
            attributes,
            derive_levels,
            derive_continues: tmpl.derive_continues,
        };
        Some(inst)
    }

    #[inline]
    pub fn animations(&self) -> InstActionGeneralIter<'_> {
        InstActionGeneralIter::new(self)
    }
}

pub struct InstActionGeneralIter<'t> {
    action: &'t InstActionGeneral,
    idx: usize,
}

impl<'t> InstActionGeneralIter<'t> {
    fn new(action: &'t InstActionGeneral) -> Self {
        Self { action, idx: 0 }
    }
}

impl<'t> Iterator for InstActionGeneralIter<'t> {
    type Item = &'t InstAnimation;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        self.idx += 1;
        return match idx {
            0 => Some(&self.action.anim_main),
            _ => None,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::{TmplDatabase, TmplHashMap};
    use crate::utils::{id, sb, TimeRange, VirtualDir, VirtualKeyDir, LEVEL_ACTION, LEVEL_ATTACK};
    use ahash::HashMapExt;
    use approx::assert_ulps_eq;

    #[test]
    fn test_assemble() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut var_indexes = TmplHashMap::new();

        {
            let tmpl_act = db
                .find_as::<TmplActionGeneral>(id!("Action.Instance.AttackDerive/1A"))
                .unwrap();
            let ctx = ContextActionAssemble {
                var_indexes: &var_indexes,
            };
            assert!(InstActionGeneral::try_assemble(&ctx, tmpl_act).is_none());
        }
        {
            let tmpl_act = db
                .find_as::<TmplActionGeneral>(id!("Action.Instance.Attack/1A"))
                .unwrap();
            var_indexes.insert(id!("#.Action.Instance.Attack/1A"), 2);
            let ctx = ContextActionAssemble {
                var_indexes: &var_indexes,
            };
            let inst_act = InstActionGeneral::try_assemble(&ctx, tmpl_act).unwrap();
            assert_eq!(inst_act.tmpl_id, id!("Action.Instance.Attack/1A"));
            assert_eq!(
                inst_act.enter_key.unwrap(),
                VirtualKeyDir::new(VirtualKey::Attack1, None)
            );
            assert_eq!(inst_act.enter_level, LEVEL_ATTACK);
            assert_eq!(inst_act.anim_main.files, sb!("girl_attack1_1"));
            assert_eq!(inst_act.anim_main.duration, 4.0);
            assert_eq!(inst_act.anim_main.fade_in, 0.2);
            assert_eq!(inst_act.attributes.len(), 1);
            assert_ulps_eq!(
                inst_act.motion_distance_ratio[0],
                0.7 / inst_act.anim_main.root_max_distance
            );
            assert_ulps_eq!(
                inst_act.motion_distance_ratio[1],
                1.2 / inst_act.anim_main.root_max_distance
            );
            assert_ulps_eq!(inst_act.motion_toward_cos, 0.5);
            assert_eq!(inst_act.attributes[0].value.damage_rdc, 0.2);
            assert_eq!(inst_act.attributes[0].value.shield_dmg_rdc, 0.0);
            assert_eq!(inst_act.attributes[0].value.poise_level, 1);
            assert_eq!(inst_act.derive_levels[0].range, TimeRange::new(0.0, 2.5));
            assert_eq!(inst_act.derive_levels[0].value, LEVEL_ACTION);
            assert_eq!(inst_act.derive_levels[1].range, TimeRange::new(2.5, 4.5));
            assert_eq!(inst_act.derive_levels[1].value, LEVEL_ATTACK);
            assert_eq!(inst_act.derives.len(), 2);
            assert_eq!(inst_act.derives[0].key, VirtualKey::Attack1);
            assert!(inst_act.derives[0].dir.is_none());
            assert_eq!(inst_act.derives[0].action, id!("Action.Instance.AttackDerive/1A"));
            assert_eq!(inst_act.derives[1].key, VirtualKey::Attack2);
            assert_eq!(inst_act.derives[1].dir.unwrap(), VirtualDir::Forward(0.5));
            assert_eq!(inst_act.derives[1].action, id!("Action.Instance.AttackDerive/1A"));
            assert!(inst_act.derive_continues.is_empty());
        }
    }
}
