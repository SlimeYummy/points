use crate::instance::action::base::{
    ContextActionAssemble, InstActionAny, InstActionBase, InstAnimation, InstDeriveRule, InstHit, InstTimelinePoint,
    InstTimelineRange,
};
use crate::template::{
    At, TmplActionGeneralNpc, TmplActionGeneralNpcMovement, TmplActionGeneralNpcRotation,
    TmplActionGeneralNpcTranslation,
};
use crate::utils::{ActionType, Symbol, ThinVec, XResult, extend, sb};

pub type InstActionGeneralNpcMovement = TmplActionGeneralNpcMovement;
pub type InstActionGeneralNpcTranslation = TmplActionGeneralNpcTranslation;
pub type InstActionGeneralNpcRotation = TmplActionGeneralNpcRotation;

#[repr(C)]
#[derive(Debug)]
pub struct InstActionGeneralNpc {
    pub _base: InstActionBase,
    pub anim_main: InstAnimation,
    pub adjust_movements: InstTimelinePoint<InstActionGeneralNpcMovement>,
    // pub attributes: InstTimelineRange<InstActionAttributes>,
    pub keep_levels: InstTimelineRange<u16>,
    pub custom_events: InstTimelinePoint<Symbol>,
}

extend!(InstActionGeneralNpc, InstActionBase);

unsafe impl InstActionAny for InstActionGeneralNpc {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::GeneralNpc
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>) {
        self.animations().for_each(|animation| animations.push(animation));
    }

    fn derives(&self, _derive_keys: &mut Vec<InstDeriveRule>) {}
}

impl InstActionGeneralNpc {
    pub(crate) fn new_from_action(
        ctx: &ContextActionAssemble<'_>,
        tmpl: At<TmplActionGeneralNpc>,
    ) -> XResult<Option<InstActionGeneralNpc>> {
        if !ctx.solve_var(&tmpl.enabled) {
            return Ok(None);
        }

        let adjust_movements =
            InstTimelinePoint::from_rkyv(&tmpl.adjust_movements, |t| InstActionGeneralNpcMovement::from_rkyv(t))?;

        // let attributes = InstTimelineRange::from_rkyv(&tmpl.attributes, |archived| {
        //     Ok(InstActionAttributes::from_rkyv(ctx, archived))
        // })?;

        let keep_levels = InstTimelineRange::from_rkyv(&tmpl.keep_levels, |level| Ok(level.to_native()))?;

        let mut hits = ThinVec::with_capacity(tmpl.hits.len());
        for hit in tmpl.hits.iter() {
            hits.push(InstHit::from_rkyv(ctx, hit));
        }

        let custom_events = InstTimelinePoint::from_rkyv(&tmpl.custom_events, |s| Ok(sb!(s)))?;

        let inst = InstActionGeneralNpc {
            _base: InstActionBase {
                tmpl_id: tmpl.id,
                tags: tmpl.tags.iter().map(|t| sb!(t)).collect(),
                hits,
                ..Default::default()
            },
            anim_main: InstAnimation::from_rkyv(&tmpl.anim_main),
            adjust_movements,
            // attributes,
            keep_levels,
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
    use crate::template::TmplDatabase;
    use crate::utils::{DtHashMap, F32Range, LEVEL_ACTION, LEVEL_ATTACK, TimeRange, cf2s, id, sb};

    #[test]
    fn test_new_general_npc() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let var_indexes = DtHashMap::default();

        let tmpl_act = db
            .find_as::<TmplActionGeneralNpc>(id!("Action.InstanceNpc.Attack^1A"))
            .unwrap();
        let ctx = ContextActionAssemble {
            var_indexes: &var_indexes,
        };
        let inst_act = InstActionGeneralNpc::new_from_action(&ctx, tmpl_act).unwrap().unwrap();
        assert_eq!(inst_act.tmpl_id, id!("Action.InstanceNpc.Attack^1A"));
        assert_eq!(inst_act.tags, vec![sb!("Attack")]);

        assert_eq!(inst_act.anim_main.files, sb!("Slime/Attack1A.*"));
        assert_eq!(inst_act.anim_main.duration, cf2s(206));
        assert_eq!(inst_act.anim_main.fade_in, 0.1);
        assert_eq!(inst_act.anim_main.root_motion, true);
        assert_eq!(inst_act.anim_main.weapon_motion, false);
        assert_eq!(inst_act.anim_main.hit_motion, false);

        assert_eq!(inst_act.adjust_movements.len(), 2);
        assert_eq!(inst_act.adjust_movements[0].time, 0.0);
        assert_eq!(
            inst_act.adjust_movements[0].value,
            InstActionGeneralNpcMovement::Rotation(InstActionGeneralNpcRotation {
                duration: cf2s(8),
                max_angle: 45.0f32.to_radians()
            })
        );
        assert_eq!(inst_act.adjust_movements[1].time, cf2s(20));
        assert_eq!(
            inst_act.adjust_movements[1].value,
            InstActionGeneralNpcMovement::Translation(InstActionGeneralNpcTranslation {
                duration: cf2s(20),
                fade_ratio: 0.1,
                distance: F32Range::new(2.0, 5.0),
                speed_ratio: F32Range::new(0.8, 1.5)
            })
        );

        assert_eq!(inst_act.keep_levels.len(), 2);
        assert_eq!(inst_act.keep_levels[0].range, TimeRange::new(0.0, cf2s(150)));
        assert_eq!(inst_act.keep_levels[0].value, LEVEL_ACTION);
        assert_eq!(inst_act.keep_levels[1].range, TimeRange::new(cf2s(150), cf2s(206)));
        assert_eq!(inst_act.keep_levels[1].value, LEVEL_ATTACK);

        assert_eq!(inst_act.custom_events.len(), 1);
        assert_eq!(inst_act.custom_events[0].time, 1.0);
        assert_eq!(inst_act.custom_events[0].value, sb!("CustomEvent"));
    }
}
