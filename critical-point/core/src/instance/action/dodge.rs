use thin_vec::ThinVec;

use crate::instance::action::base::{
    ContextActionAssemble, InstActionAny, InstActionBase, InstAnimation, InstDeriveRule,
};
use crate::template::{At, TmplActionDodge, TmplActionDodgeKeepLevels, TmplActionDodgeTimes};
use crate::utils::{ActionType, F32Range, TmplID, VirtualKeyDir, XResult, extend, sb};

pub type InstActionDodgeTimes = TmplActionDodgeTimes;
pub type InstActionDodgeKeepLevels = TmplActionDodgeKeepLevels;

#[repr(C)]
#[derive(Debug)]
pub struct InstActionDodge {
    pub _base: InstActionBase,
    pub anim_dodge: InstAnimation,
    pub smooth_move_froms: Vec<TmplID>,
    pub smooth_no_leg_fade_in: f32,
    pub smooth_speed_duration: f32,
    pub dodge_time: F32Range,
    pub derive_times: InstActionDodgeTimes,
    pub keep_levels: InstActionDodgeKeepLevels,
    pub derives: ThinVec<InstDeriveRule>,
}

extend!(InstActionDodge, InstActionBase);

unsafe impl InstActionAny for InstActionDodge {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::Dodge
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>) {
        self.animations().for_each(|anime| animations.push(anime));
    }

    fn derives(&self, derive_keys: &mut Vec<InstDeriveRule>) {
        for rule in self.derives.iter() {
            derive_keys.push(rule.clone());
        }
    }
}

impl InstActionDodge {
    pub(crate) fn new_from_action(
        ctx: &ContextActionAssemble<'_>,
        tmpl: At<TmplActionDodge>,
    ) -> XResult<Option<InstActionDodge>> {
        if !ctx.solve_var(&tmpl.enabled) {
            return Ok(None);
        }

        let mut derives = ThinVec::with_capacity(tmpl.derives.len());
        for rule in tmpl.derives.iter() {
            let rule = InstDeriveRule::from_rkyv(ctx, rule);
            if rule.action.is_valid() {
                derives.push(rule);
            }
        }

        let inst = InstActionDodge {
            _base: InstActionBase {
                tmpl_id: tmpl.id,
                tags: tmpl.tags.iter().map(|t| sb!(t)).collect(),
                enter_key: tmpl.enter_key.as_ref().map(|key| VirtualKeyDir::new(*key, None)),
                enter_level: tmpl.enter_level.to_native(),
                ..Default::default()
            },
            anim_dodge: InstAnimation::from_rkyv(&tmpl.anim_dodge),
            smooth_move_froms: tmpl.smooth_move_froms.iter().map(|x| *x).collect(),
            smooth_no_leg_fade_in: tmpl.smooth_no_leg_fade_in.to_native(),
            smooth_speed_duration: tmpl.smooth_speed_duration.to_native(),
            dodge_time: tmpl.dodge_time,
            derive_times: TmplActionDodgeTimes::from_rkyv(&tmpl.derive_times),
            keep_levels: TmplActionDodgeKeepLevels::from_rkyv(&tmpl.keep_levels),
            derives,
        };
        Ok(Some(inst))
    }

    #[inline]
    pub fn animations(&self) -> impl Iterator<Item = &InstAnimation> {
        std::iter::once(&self.anim_dodge)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::{DtHashMap, LEVEL_ACTION, LEVEL_DERIVE, LEVEL_MOVE, VirtualKey, cf2s, id, sb};

    #[test]
    fn test_new_dodge() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let var_indexes = DtHashMap::default();
        let tmpl_act = db.find_as::<TmplActionDodge>(id!("Action.Instance.Dodge^1A")).unwrap();
        let ctx = ContextActionAssemble {
            var_indexes: &var_indexes,
        };
        let inst_act = InstActionDodge::new_from_action(&ctx, tmpl_act).unwrap().unwrap();

        assert_eq!(inst_act.tmpl_id, id!("Action.Instance.Dodge^1A"));
        assert_eq!(inst_act.tags, vec![sb!("Dodge")]);
        assert_eq!(inst_act.enter_key.unwrap(), VirtualKeyDir::new(VirtualKey::Dodge, None));
        assert_eq!(inst_act.enter_level, LEVEL_DERIVE);

        assert_eq!(inst_act.anim_dodge.files, sb!("Girl/Dodge_F_Empty.*"));
        assert_eq!(inst_act.anim_dodge.duration, cf2s(72));
        assert_eq!(inst_act.anim_dodge.fade_in, 0.1);
        assert_eq!(inst_act.anim_dodge.root_motion, true);
        assert_eq!(inst_act.anim_dodge.weapon_control, false);
        assert_eq!(inst_act.anim_dodge.hit_motion, false);
        assert_eq!(inst_act.anim_dodge.shape_key, false);
        assert_eq!(inst_act.anim_dodge.additive_blending, false);

        assert_eq!(inst_act.smooth_move_froms, vec![id!("Action.Instance.Run^1A")]);
        assert_eq!(inst_act.smooth_no_leg_fade_in, cf2s(8));
        assert_eq!(inst_act.smooth_speed_duration, cf2s(10));
        assert_eq!(inst_act.dodge_time, F32Range::new(cf2s(4), cf2s(34)));
        assert_eq!(inst_act.derive_times.quick, cf2s(34));
        assert_eq!(inst_act.derive_times.normal, cf2s(60));
        assert_eq!(inst_act.keep_levels.dodge, LEVEL_ACTION);
        assert_eq!(inst_act.keep_levels.quick, LEVEL_DERIVE);
        assert_eq!(inst_act.keep_levels.normal, LEVEL_MOVE);
    }
}
