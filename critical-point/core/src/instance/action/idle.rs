use crate::instance::action::base::{ContextActionAssemble, InstActionAny, InstActionBase, InstAnimation};
use crate::template::{At, TmplActionIdle, TmplType};
use crate::utils::{extend, TmplID, VirtualKey, VirtualKeyDir};

#[repr(C)]
#[derive(Debug)]
pub struct InstActionIdle {
    pub _base: InstActionBase,
    pub anim_idle: InstAnimation,
    pub anim_ready: InstAnimation,
    pub anim_randoms: Vec<InstAnimation>,
    pub auto_idle_delay: f32,
    pub derive_level: u16,
    pub poise_level: u16,
}

extend!(InstActionIdle, InstActionBase);

unsafe impl InstActionAny for InstActionIdle {
    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::ActionIdle
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>) {
        self.animations().for_each(|anime| animations.push(anime));
    }

    fn derives(&self, _derives: &mut Vec<(VirtualKey, TmplID)>) {}
}

impl InstActionIdle {
    pub(crate) fn try_assemble(ctx: &ContextActionAssemble<'_>, tmpl: At<TmplActionIdle>) -> Option<InstActionIdle> {
        if !ctx.solve_var(&tmpl.enabled) {
            return None;
        }

        Some(InstActionIdle {
            _base: InstActionBase {
                tmpl_id: tmpl.id,
                enter_key: Some(VirtualKeyDir::new(VirtualKey::Idle, None)),
                enter_level: tmpl.enter_level.into(),
                ..Default::default()
            },
            anim_idle: InstAnimation::from_rkyv(&tmpl.anim_idle),
            anim_ready: InstAnimation::from_rkyv(&tmpl.anim_ready),
            anim_randoms: tmpl.anim_randoms.iter().map(InstAnimation::from_rkyv).collect(),
            auto_idle_delay: tmpl.auto_idle_delay.into(),
            derive_level: tmpl.derive_level.into(),
            poise_level: tmpl.poise_level.into(),
        })
    }

    #[inline]
    pub fn animations(&self) -> InstActionIdleIter<'_> {
        InstActionIdleIter::new(self)
    }
}

pub struct InstActionIdleIter<'t> {
    action: &'t InstActionIdle,
    idx: usize,
}

impl<'t> InstActionIdleIter<'t> {
    fn new(action: &'t InstActionIdle) -> Self {
        Self { action, idx: 0 }
    }
}

impl<'t> Iterator for InstActionIdleIter<'t> {
    type Item = &'t InstAnimation;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        self.idx += 1;
        return match idx {
            0 => Some(&self.action.anim_idle),
            1 => Some(&self.action.anim_ready),
            _ => {
                let idx = idx - 2;
                self.action.anim_randoms.get(idx)
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::{TmplDatabase, TmplHashMap};
    use crate::utils::{id, sb};
    use ahash::HashMapExt;

    #[test]
    fn test_ssemble() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let var_indexes = TmplHashMap::new();

        let tmpl_act = db.find_as::<TmplActionIdle>(id!("Action.Instance.Idle/1A")).unwrap();
        let ctx = ContextActionAssemble {
            var_indexes: &var_indexes,
        };
        let inst_act = InstActionIdle::try_assemble(&ctx, tmpl_act).unwrap();
        assert_eq!(inst_act.tmpl_id, id!("Action.Instance.Idle/1A"));
        assert_eq!(inst_act.enter_key.unwrap(), VirtualKeyDir::new(VirtualKey::Idle, None));
        assert_eq!(inst_act.enter_level, 0);
        assert_eq!(inst_act.anim_idle.files, sb!("girl_stand_idle"));
        assert_eq!(inst_act.anim_idle.duration, 2.5);
        assert_eq!(inst_act.anim_idle.fade_in, 0.2);
        assert_eq!(inst_act.anim_ready.files, sb!("girl_stand_ready"));
        assert_eq!(inst_act.anim_ready.duration, 2.0);
        assert_eq!(inst_act.anim_ready.fade_in, 0.4);
        assert_eq!(inst_act.anim_randoms.len(), 0);
        assert_eq!(inst_act.auto_idle_delay, 10.0);
        assert_eq!(inst_act.derive_level, 0);
        assert_eq!(inst_act.poise_level, 0);
    }
}
