use approx::abs_diff_eq;
use rkyv::Archived;
use std::f32::consts::PI;

use crate::instance::action::base::{
    ContextActionAssemble, InstActionAny, InstActionBase, InstAnimation, InstDeriveRule,
};
use crate::template::{At, TmplActionHit, TmplActionHitBeHit};
use crate::utils::{extend, ifelse, ratio_saturating, sb, xresf, ActionType, TmplID, VirtualKeyDir, XResult};

#[repr(C)]
#[derive(Debug)]
pub struct InstActionHit {
    pub _base: InstActionBase,
    pub be_hits: Vec<InstActionHitBeHit>,
    pub anim_down: Option<InstAnimation>,
    pub max_down_time: f32,
    pub anim_recovery: Option<InstAnimation>,
    pub derive_level: u16,
}

extend!(InstActionHit, InstActionBase);

#[derive(Debug)]
pub struct InstActionHitBeHit {
    pub anim: InstAnimation,
    pub enter_angle: f32,
}

impl InstActionHitBeHit {
    fn vec_from_rkyv(
        archived_be_hits: &Archived<Vec<TmplActionHitBeHit>>,
        tmpl_id: TmplID,
    ) -> XResult<Vec<InstActionHitBeHit>> {
        let mut be_hits = Vec::with_capacity(archived_be_hits.len());
        for a in archived_be_hits.iter() {
            let enter_angle = a.enter_angle.to_native();
            if enter_angle.abs() > PI {
                return xresf!(BadAsset; "tmpl_id={}, be_hits enter_angle={}", tmpl_id, enter_angle);
            }
            be_hits.push(InstActionHitBeHit {
                anim: InstAnimation::from_rkyv(&a.anim),
                enter_angle,
            });
        }

        if be_hits.len() < 1 {
            return xresf!(BadAsset; "tmpl_id={}, too less be_hits", tmpl_id);
        }
        Ok(be_hits)
    }
}

unsafe impl InstActionAny for InstActionHit {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::Hit
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>) {
        self.animations().for_each(|anime| animations.push(anime));
    }

    fn derives(&self, _derives: &mut Vec<InstDeriveRule>) {}
}

impl InstActionHit {
    pub(crate) fn new_from_action(
        ctx: &ContextActionAssemble<'_>,
        tmpl: At<TmplActionHit>,
    ) -> XResult<Option<InstActionHit>> {
        if !ctx.solve_var(&tmpl.enabled) {
            return Ok(None);
        }

        Ok(Some(InstActionHit {
            _base: InstActionBase {
                tmpl_id: tmpl.id,
                tags: tmpl.tags.iter().map(|t| sb!(t)).collect(),
                enter_key: Some(VirtualKeyDir::new(tmpl.enter_key, None)),
                enter_level: tmpl.enter_level.to_native(),
                ..Default::default()
            },
            be_hits: InstActionHitBeHit::vec_from_rkyv(&tmpl.be_hits, tmpl.id)?,
            anim_down: match tmpl.anim_down.as_ref() {
                Some(t) => Some(InstAnimation::from_rkyv(t)),
                None => None,
            },
            max_down_time: tmpl.max_down_time.to_native(),
            anim_recovery: match tmpl.anim_recovery.as_ref() {
                Some(t) => Some(InstAnimation::from_rkyv(t)),
                None => None,
            },
            derive_level: tmpl.derive_level.to_native(),
        }))
    }

    #[inline]
    pub fn animations(&self) -> impl Iterator<Item = &InstAnimation> {
        std::iter::from_coroutine(
            #[coroutine]
            || {
                for be_hit in &self.be_hits {
                    yield &be_hit.anim;
                }
                if let Some(anim) = &self.anim_down {
                    yield &anim;
                }
                if let Some(anim) = &self.anim_recovery {
                    yield &anim;
                }
            },
        )
    }

    #[inline]
    pub fn animations_count(&self) -> usize {
        let mut count = self.be_hits.len();
        if self.anim_down.is_some() {
            count += 1;
        }
        if self.anim_recovery.is_some() {
            count += 1;
        }
        count
    }

    pub fn find_be_hit_by_angle(&self, angle: f32) -> FindBeHit {
        // Notice: all angles are in [-PI, PI], on right hand xz plane.

        let mut res = FindBeHit::default();
        let mut min_diff0 = f32::MAX;
        let mut min_diff1 = f32::MAX;
        for (idx, be_hit) in self.be_hits.iter().enumerate() {
            let mut diff = (angle - be_hit.enter_angle + PI) % (2.0 * PI) - PI;
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
pub struct FindBeHit {
    pub index0: u32,
    pub index1: u32,
    pub ratio: f32,
    pub angle_diff: f32,
}

impl Default for FindBeHit {
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
    use crate::utils::{id, sb, DtHashMap, VirtualKey};

    // #[test]
    // fn test_new() {
    //     let db = TmplDatabase::new(10240, 150).unwrap();
    //     let var_indexes = DtHashMap::default();

    //     let tmpl_act = db.find_as::<TmplActionIdle>(id!("Action.Instance.Idle^1A")).unwrap();
    //     let ctx = ContextActionAssemble {
    //         var_indexes: &var_indexes,
    //     };
    //     let inst_act = InstActionHit::new_from_action(&ctx, tmpl_act).unwrap();
    //     assert_eq!(inst_act.tmpl_id, id!("Action.Instance.Idle^1A"));
    //     assert_eq!(inst_act.tags, vec![sb!("Idle")]);
    //     assert_eq!(inst_act.enter_key.unwrap(), VirtualKeyDir::new(VirtualKey::Idle, None));
    //     assert_eq!(inst_act.enter_level, 0);
    //     assert_eq!(inst_act.anim_idle.files, sb!("Girl_Idle_Empty.*"));
    //     assert_eq!(inst_act.anim_idle.duration, 2.5);
    //     assert_eq!(inst_act.anim_idle.fade_in, 0.2);
    //     let anim_ready = inst_act.anim_ready.as_ref().unwrap();
    //     assert_eq!(anim_ready.files, sb!("Girl_Idle_Axe.*"));
    //     assert_eq!(anim_ready.duration, 2.0);
    //     assert_eq!(anim_ready.fade_in, 0.4);
    //     assert_eq!(inst_act.anim_randoms.len(), 0);
    //     assert_eq!(inst_act.auto_idle_delay, 10.0);
    //     assert_eq!(inst_act.derive_level, 0);
    //     assert_eq!(inst_act.poise_level, 0);
    // }

    #[test]
    fn test_new_npc() {
        let var_indexes = DtHashMap::default();
        let ctx = ContextActionAssemble {
            var_indexes: &var_indexes,
        };

        let db = TmplDatabase::new(10240, 150).unwrap();
        let tmpl_act = db.find_as::<TmplActionHit>(id!("Action.NpcInstance.Hit1^1A")).unwrap();

        let inst_act = InstActionHit::new_from_action(&ctx, tmpl_act).unwrap().unwrap();

        assert_eq!(inst_act.tmpl_id, id!("Action.NpcInstance.Hit1^1A"));
        assert_eq!(inst_act.tags, vec![sb!("Hit")]);
        assert_eq!(inst_act.enter_key.unwrap(), VirtualKeyDir::new(VirtualKey::Hit1, None));
        assert_eq!(inst_act.enter_level, 610);

        assert_eq!(inst_act.be_hits.len(), 1);
        assert_eq!(inst_act.be_hits[0].enter_angle, 15f32.to_radians());
        assert_eq!(inst_act.be_hits[0].anim.files, sb!("TrainingDummy_Hit1_F.*"));
        assert_eq!(inst_act.be_hits[0].anim.duration, 0.5);
        assert_eq!(inst_act.be_hits[0].anim.fade_in, 0.1);

        assert!(inst_act.anim_down.is_none());
        assert_eq!(inst_act.max_down_time, 0.0);
        assert!(inst_act.anim_recovery.is_none());
        assert_eq!(inst_act.derive_level, 600);
    }
}
