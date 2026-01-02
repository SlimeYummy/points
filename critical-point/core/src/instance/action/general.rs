use crate::instance::action::base::{
    ContextActionAssemble, InstActionAny, InstActionAttributes, InstActionBase, InstAnimation, InstDeriveRule,
    InstTimelinePoint, InstTimelineRange,
};
use crate::template::{
    At, TmplActionGeneral, TmplActionGeneralMovement, TmplActionGeneralRootMotion, TmplActionGeneralRotation, TmplType,
};
use crate::utils::{extend, sb, Bitsetable, DeriveContinue, EnumBitset, Symbol, XResult};

pub type InstActionGeneralMovement = TmplActionGeneralMovement;
pub type InstActionGeneralRootMotion = TmplActionGeneralRootMotion;
pub type InstActionGeneralRotation = TmplActionGeneralRotation;

#[repr(C)]
#[derive(Debug)]
pub struct InstActionGeneral {
    pub _base: InstActionBase,
    pub anim_main: InstAnimation,
    pub attributes: InstTimelineRange<InstActionAttributes>,
    pub input_movements: InstTimelinePoint<InstActionGeneralMovement>,
    pub derive_levels: InstTimelineRange<u16>,
    pub derives: Vec<InstDeriveRule>,
    pub derive_continues: EnumBitset<DeriveContinue, { DeriveContinue::LEN }>,
    pub custom_events: InstTimelinePoint<Symbol>,
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

    fn derives(&self, derive_keys: &mut Vec<InstDeriveRule>) {
        for rule in self.derives.iter() {
            derive_keys.push(rule.clone());
        }
    }
}

impl InstActionGeneral {
    pub(crate) fn try_assemble(
        ctx: &ContextActionAssemble<'_>,
        tmpl: At<TmplActionGeneral>,
    ) -> XResult<Option<InstActionGeneral>> {
        if !ctx.solve_var(&tmpl.enabled) {
            return Ok(None);
        }

        let mut derives = Vec::with_capacity(tmpl.derives.len());
        for rule in tmpl.derives.iter() {
            let rule = InstDeriveRule::from_rkyv(ctx, rule);
            if rule.action.is_valid() {
                derives.push(rule);
            }
        }

        let attributes = InstTimelineRange::from_rkyv(&tmpl.attributes, |archived| {
            Ok(InstActionAttributes::from_rkyv(ctx, archived))
        })?;
        let derive_levels = InstTimelineRange::from_rkyv(&tmpl.derive_levels, |level| Ok(ctx.solve_var(level).into()))?;
        let input_movements =
            InstTimelinePoint::from_rkyv(&tmpl.input_movements, |t| InstActionGeneralMovement::from_rkyv(t))?;
        let custom_events = InstTimelinePoint::from_rkyv(&tmpl.custom_events, |s| Symbol::new(s))?;

        let inst = InstActionGeneral {
            _base: InstActionBase {
                tmpl_id: tmpl.id,
                tags: tmpl.tags.iter().map(|t| sb!(t)).collect(),
                enter_key: tmpl.enter_key.as_ref().cloned(),
                enter_level: tmpl.enter_level.into(),
                ..Default::default()
            },
            derives,
            anim_main: InstAnimation::from_rkyv(&tmpl.anim_main),
            input_movements,
            attributes,
            derive_levels,
            derive_continues: tmpl.derive_continues,
            custom_events,
        };
        Ok(Some(inst))
    }

    #[inline]
    pub fn animations(&self) -> impl Iterator<Item = &InstAnimation> {
        std::iter::once(&self.anim_main)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::RootTrackName;
    use crate::template::{TmplDatabase, TmplHashMap};
    use crate::utils::{cf2s, id, sb, InputDir, TimeRange, VirtualKey, VirtualKeyDir, LEVEL_ACTION, LEVEL_ATTACK};
    use ahash::HashMapExt;

    #[test]
    fn test_assemble() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let mut var_indexes = TmplHashMap::new();

        {
            let tmpl_act = db
                .find_as::<TmplActionGeneral>(id!("Action.Instance.AttackDerive^1A"))
                .unwrap();
            let ctx = ContextActionAssemble {
                var_indexes: &var_indexes,
            };
            assert!(InstActionGeneral::try_assemble(&ctx, tmpl_act).unwrap().is_none());
        }
        {
            let tmpl_act = db
                .find_as::<TmplActionGeneral>(id!("Action.Instance.Attack^1A"))
                .unwrap();
            var_indexes.insert(id!("#.Action.Instance.Attack^1A"), 2);
            let ctx = ContextActionAssemble {
                var_indexes: &var_indexes,
            };
            let inst_act = InstActionGeneral::try_assemble(&ctx, tmpl_act).unwrap().unwrap();
            assert_eq!(inst_act.tmpl_id, id!("Action.Instance.Attack^1A"));
            assert_eq!(inst_act.tags, vec![sb!("Attack")]);
            assert_eq!(
                inst_act.enter_key.unwrap(),
                VirtualKeyDir::new(VirtualKey::Attack1, None)
            );
            assert_eq!(inst_act.enter_level, LEVEL_ATTACK);
            assert_eq!(inst_act.anim_main.files, sb!("Girl_Attack_01A.*"));
            assert_eq!(inst_act.anim_main.duration, 4.0);
            assert_eq!(inst_act.anim_main.fade_in, 0.1);
            assert_eq!(inst_act.attributes.len(), 1);
            // assert_eq!(inst_act.input_root_motion, InstActionGeneralRootMotion {
            //     in_place: 0.7,
            //     normal: 0.7,
            //     extended: 1.2,
            // });
            assert_eq!(inst_act.input_movements.len(), 2);
            assert_eq!(inst_act.input_movements[0].time, 0.0);
            assert_eq!(
                inst_act.input_movements[0].value,
                InstActionGeneralMovement::Rotation(InstActionGeneralRotation {
                    duration: cf2s(8),
                    angle: 60.0 * std::f32::consts::PI / 180.0,
                })
            );
            assert_eq!(inst_act.input_movements[1].time, cf2s(24));
            assert_eq!(
                inst_act.input_movements[1].value,
                InstActionGeneralMovement::RootMotion(InstActionGeneralRootMotion {
                    mov: false,
                    mov_ex: true,
                })
            );
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
            assert_eq!(inst_act.derives[0].level, LEVEL_ATTACK + 1);
            assert_eq!(inst_act.derives[0].action, id!("Action.Instance.AttackDerive^1A"));
            assert_eq!(inst_act.derives[1].key, VirtualKey::Attack2);
            assert_eq!(inst_act.derives[1].dir.unwrap(), InputDir::Backward(0.5));
            assert_eq!(inst_act.derives[1].level, LEVEL_ATTACK + 1);
            assert_eq!(inst_act.derives[1].action, id!("Action.Instance.AttackDerive^1A"));
            assert!(inst_act.derive_continues.is_empty());
            assert_eq!(inst_act.custom_events.len(), 2);
            assert_eq!(inst_act.custom_events[0].time, 1.0);
            assert_eq!(inst_act.custom_events[0].value, "Event1s");
            assert_eq!(inst_act.custom_events[1].time, 2.0);
            assert_eq!(inst_act.custom_events[1].value, "Event2s");
        }
    }
}
