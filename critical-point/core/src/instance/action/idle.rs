use thin_vec::ThinVec;

use crate::instance::action::base::{
    ContextActionAssemble, InstActionAny, InstActionBase, InstAnimation, InstDeriveRule,
};
use crate::template::{At, TmplActionIdle, TmplNpcActionIdle};
use crate::utils::{extend, sb, ActionType, VirtualKey, VirtualKeyDir};

#[repr(C)]
#[derive(Debug)]
pub struct InstActionIdle {
    pub _base: InstActionBase,
    pub anim_idle: InstAnimation,
    pub anim_ready: Option<InstAnimation>,
    pub anim_randoms: ThinVec<InstAnimation>,
    pub auto_idle_delay: f32,
    pub derive_level: u16,
    pub poise_level: u16,
}

extend!(InstActionIdle, InstActionBase);

unsafe impl InstActionAny for InstActionIdle {
    #[inline]
    fn typ(&self) -> ActionType {
        ActionType::Idle
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>) {
        self.animations().for_each(|anime| animations.push(anime));
    }

    fn derives(&self, _derives: &mut Vec<InstDeriveRule>) {}
}

impl InstActionIdle {
    pub(crate) fn new_from_action(ctx: &ContextActionAssemble<'_>, tmpl: At<TmplActionIdle>) -> Option<InstActionIdle> {
        if !ctx.solve_var(&tmpl.enabled) {
            return None;
        }

        Some(InstActionIdle {
            _base: InstActionBase {
                tmpl_id: tmpl.id,
                tags: tmpl.tags.iter().map(|t| sb!(t)).collect(),
                enter_key: Some(VirtualKeyDir::new(VirtualKey::Idle, None)),
                enter_level: tmpl.enter_level.into(),
                ..Default::default()
            },
            anim_idle: InstAnimation::from_rkyv(&tmpl.anim_idle),
            anim_ready: match tmpl.anim_ready.as_ref() {
                Some(t) => Some(InstAnimation::from_rkyv(t)),
                None => None,
            },
            anim_randoms: tmpl.anim_randoms.iter().map(InstAnimation::from_rkyv).collect(),
            auto_idle_delay: tmpl.auto_idle_delay.into(),
            derive_level: tmpl.derive_level.into(),
            poise_level: tmpl.poise_level.into(),
        })
    }

    pub(crate) fn new_from_npc_action(tmpl: At<TmplNpcActionIdle>) -> Option<InstActionIdle> {
        Some(InstActionIdle {
            _base: InstActionBase {
                tmpl_id: tmpl.id,
                tags: tmpl.tags.iter().map(|t| sb!(t)).collect(),
                enter_key: Some(VirtualKeyDir::new(VirtualKey::Idle, None)),
                enter_level: 0,
                ..Default::default()
            },
            anim_idle: InstAnimation::from_rkyv(&tmpl.anim_idle),
            anim_ready: match tmpl.anim_ready.as_ref() {
                Some(t) => Some(InstAnimation::from_rkyv(t)),
                None => None,
            },
            anim_randoms: ThinVec::new(),
            auto_idle_delay: tmpl.auto_idle_delay.into(),
            derive_level: 0,
            poise_level: tmpl.poise_level.into(),
        })
    }

    #[inline]
    pub fn animations(&self) -> impl Iterator<Item = &InstAnimation> {
        std::iter::from_coroutine(
            #[coroutine]
            || {
                yield &self.anim_idle;
                if let Some(anim) = &self.anim_ready {
                    yield anim;
                }
                for anim in &self.anim_randoms {
                    yield anim;
                }
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TmplDatabase;
    use crate::utils::{id, sb, DtHashMap};

    #[test]
    fn test_new_from_action() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let var_indexes = DtHashMap::default();

        let tmpl_act = db.find_as::<TmplActionIdle>(id!("Action.Instance.Idle^1A")).unwrap();
        let ctx = ContextActionAssemble {
            var_indexes: &var_indexes,
        };
        let inst_act = InstActionIdle::new_from_action(&ctx, tmpl_act).unwrap();
        assert_eq!(inst_act.tmpl_id, id!("Action.Instance.Idle^1A"));
        assert_eq!(inst_act.tags, vec![sb!("Idle")]);
        assert_eq!(inst_act.enter_key.unwrap(), VirtualKeyDir::new(VirtualKey::Idle, None));
        assert_eq!(inst_act.enter_level, 0);
        assert_eq!(inst_act.anim_idle.files, sb!("Girl_Idle_Empty.*"));
        assert_eq!(inst_act.anim_idle.duration, 2.5);
        assert_eq!(inst_act.anim_idle.fade_in, 0.2);
        let anim_ready = inst_act.anim_ready.as_ref().unwrap();
        assert_eq!(anim_ready.files, sb!("Girl_Idle_Axe.*"));
        assert_eq!(anim_ready.duration, 2.0);
        assert_eq!(anim_ready.fade_in, 0.4);
        assert_eq!(inst_act.anim_randoms.len(), 0);
        assert_eq!(inst_act.auto_idle_delay, 10.0);
        assert_eq!(inst_act.derive_level, 0);
        assert_eq!(inst_act.poise_level, 0);
    }

    #[test]
    fn test_new_from_npc_action() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let tmpl_act = db
            .find_as::<TmplNpcActionIdle>(id!("NpcAction.Instance.Idle^1A"))
            .unwrap();
        let inst_act = InstActionIdle::new_from_npc_action(tmpl_act).unwrap();
        assert_eq!(inst_act.tmpl_id, id!("NpcAction.Instance.Idle^1A"));
        assert_eq!(inst_act.tags, vec![sb!("Idle")]);
        assert_eq!(inst_act.enter_key.unwrap(), VirtualKeyDir::new(VirtualKey::Idle, None));
        assert_eq!(inst_act.enter_level, 0);
        assert_eq!(inst_act.anim_idle.files, sb!("TrainingDummy_Idle.*"));
        assert_eq!(inst_act.anim_idle.duration, 4.0);
        assert_eq!(inst_act.anim_idle.fade_in, 0.5);
        assert!(inst_act.anim_ready.is_none());
        assert_eq!(inst_act.anim_randoms.len(), 0);
        assert_eq!(inst_act.auto_idle_delay, 10.0);
        assert_eq!(inst_act.derive_level, 0);
        assert_eq!(inst_act.poise_level, 0);
    }
}
