use core::f32::consts::PI;

use crate::instance::action::base::{
    ContextActionAssemble, InstActionAny, InstActionBase, InstAnimation, InstDeriveRule, InstTimelineRange,
};
use crate::template::{At, TmplActionDodgeNpc};
use crate::utils::{ActionType, F32Range, RotationReference, XResult, extend, ifelse, sb};

#[derive(Debug)]
pub struct InstActionDodgeNpcDodge {
    pub anim: InstAnimation,
    pub enter_angle: f32,
    pub rotation_reference: RotationReference,
    pub rotation_start: f32,
    pub rotation_duration: F32Range,
    pub rotation_max_angle: f32,
    pub keep_levels: InstTimelineRange<u16>,
}

#[repr(C)]
#[derive(Debug)]
pub struct InstActionDodgeNpc {
    pub _base: InstActionBase,
    pub move_distance: F32Range,
    pub anim_dodges: Vec<InstActionDodgeNpcDodge>,
}

extend!(InstActionDodgeNpc, InstActionBase);

unsafe impl InstActionAny for InstActionDodgeNpc {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::DodgeNpc
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>) {
        for dodge in &self.anim_dodges {
            animations.push(&dodge.anim);
        }
    }

    fn derives(&self, _derives: &mut Vec<InstDeriveRule>) {}
}

impl InstActionDodgeNpc {
    pub(crate) fn new_from_action(
        ctx: &ContextActionAssemble<'_>,
        tmpl: At<TmplActionDodgeNpc>,
    ) -> XResult<Option<InstActionDodgeNpc>> {
        if !ctx.solve_var(&tmpl.enabled) {
            return Ok(None);
        }

        let mut anim_dodges = Vec::with_capacity(tmpl.anim_dodges.len());
        for tmpl_dodge in tmpl.anim_dodges.iter() {
            let keep_levels = InstTimelineRange::from_rkyv(&tmpl_dodge.keep_levels, |level| Ok(level.to_native()))?;
            anim_dodges.push(InstActionDodgeNpcDodge {
                anim: InstAnimation::from_rkyv(&tmpl_dodge.anim),
                enter_angle: tmpl_dodge.enter_angle.to_native(),
                rotation_reference: tmpl_dodge.rotation_reference,
                rotation_start: tmpl_dodge.rotation_start.to_native(),
                rotation_duration: tmpl_dodge.rotation_duration,
                rotation_max_angle: tmpl_dodge.rotation_max_angle.to_native(),
                keep_levels,
            });
        }

        let inst = InstActionDodgeNpc {
            _base: InstActionBase {
                tmpl_id: tmpl.id,
                tags: tmpl.tags.iter().map(|t| sb!(t)).collect(),
                ..Default::default()
            },
            move_distance: tmpl.move_distance,
            anim_dodges,
        };
        Ok(Some(inst))
    }

    pub fn find_dodge_by_angle(&self, angle: f32) -> FindDodge {
        // Notice: all angles are in [-PI, PI], on right hand xz plane.

        let mut res = FindDodge::default();
        let mut min_diff0 = f32::MAX;
        let mut min_diff1 = f32::MAX;
        for (idx, dodge) in self.anim_dodges.iter().enumerate() {
            let mut diff = (angle - dodge.enter_angle + PI) % (2.0 * PI) - PI;
            if diff <= -PI {
                diff += 2.0 * PI;
            }
            if diff.abs() < min_diff0.abs() {
                res.index1 = res.index0;
                res.index0 = idx as u32;
                min_diff1 = min_diff0;
                min_diff0 = diff;
            }
        }

        res.angle_diff = min_diff0;
        if res.index1 != u32::MAX {
            let ratio = min_diff1.abs() / (min_diff1.abs() + min_diff0.abs());
            res.ratio = ifelse!(ratio < 1.0, ratio, 1.0);
        }
        res
    }
}

#[derive(Debug)]
pub struct FindDodge {
    pub index0: u32,
    pub index1: u32,
    pub ratio: f32,
    pub angle_diff: f32,
}

impl Default for FindDodge {
    fn default() -> Self {
        Self {
            index0: u32::MAX,
            index1: u32::MAX,
            ratio: 1.0,
            angle_diff: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::{DtHashMap, F32Range, LEVEL_ACTION, TimeRange, cf2s, id, sb};

    #[test]
    fn test_new_dodge_npc() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let var_indexes = DtHashMap::default();
        let tmpl_act = db
            .find_as::<TmplActionDodgeNpc>(id!("Action.InstanceNpc.Dodge"))
            .unwrap();
        let ctx = ContextActionAssemble {
            var_indexes: &var_indexes,
        };
        let inst_act = InstActionDodgeNpc::new_from_action(&ctx, tmpl_act).unwrap().unwrap();

        assert_eq!(inst_act.tmpl_id, id!("Action.InstanceNpc.Dodge"));
        assert_eq!(inst_act.tags, vec![sb!("Dodge")]);
        assert_eq!(inst_act.move_distance, F32Range::new(2.0, 4.0));
        assert_eq!(inst_act.anim_dodges.len(), 2);

        assert_eq!(inst_act.anim_dodges[0].anim.files, sb!("Slime/Dodge_L.*"));
        assert_eq!(inst_act.anim_dodges[0].anim.duration, cf2s(110));
        assert_eq!(inst_act.anim_dodges[0].enter_angle, (-90f32).to_radians());
        assert_eq!(
            inst_act.anim_dodges[0].rotation_reference,
            RotationReference::TargetCharacter
        );
        assert_eq!(inst_act.anim_dodges[0].rotation_start, cf2s(84));
        assert_eq!(
            inst_act.anim_dodges[0].rotation_duration,
            F32Range::new(cf2s(16), cf2s(24))
        );
        assert_eq!(inst_act.anim_dodges[0].rotation_max_angle, 180f32.to_radians());
        assert_eq!(inst_act.anim_dodges[0].keep_levels.len(), 1);
        assert_eq!(
            inst_act.anim_dodges[0].keep_levels[0].range,
            TimeRange::new(0.0, cf2s(110))
        );
        assert_eq!(inst_act.anim_dodges[0].keep_levels[0].value, LEVEL_ACTION);

        assert_eq!(inst_act.anim_dodges[1].anim.files, sb!("Slime/Dodge_R.*"));
        assert_eq!(inst_act.anim_dodges[1].anim.duration, cf2s(110));
        assert_eq!(inst_act.anim_dodges[1].enter_angle, (90f32).to_radians());
        assert_eq!(
            inst_act.anim_dodges[1].rotation_reference,
            RotationReference::TargetCharacter
        );
        assert_eq!(inst_act.anim_dodges[1].rotation_start, cf2s(84));
        assert_eq!(
            inst_act.anim_dodges[1].rotation_duration,
            F32Range::new(cf2s(16), cf2s(24))
        );
        assert_eq!(inst_act.anim_dodges[1].rotation_max_angle, 180f32.to_radians());
        assert_eq!(inst_act.anim_dodges[1].keep_levels.len(), 1);
        assert_eq!(
            inst_act.anim_dodges[1].keep_levels[0].range,
            TimeRange::new(0.0, cf2s(110))
        );
        assert_eq!(inst_act.anim_dodges[1].keep_levels[0].value, LEVEL_ACTION);
    }
}
