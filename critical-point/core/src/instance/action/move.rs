use crate::instance::action::base::{
    continue_if_none, ContextActionAssemble, InstActionAny, InstActionBase, InstAnimation,
};
use crate::template::{At, TmplActionMove, TmplType};
use crate::utils::{extend, TmplID, VirtualKey, VirtualKeyDir};

#[derive(Debug)]
#[repr(C)]
pub struct InstActionMove {
    pub _base: InstActionBase,
    pub anim_move: InstAnimation,
    pub anim_turn_left: Option<InstAnimation>,
    pub anim_turn_right: Option<InstAnimation>,
    pub anim_stop: Option<InstAnimation>,
    pub yam_time: f32,
    pub turn_time: f32,
    pub derive_level: u16,
    pub poise_level: u16,
}

extend!(InstActionMove, InstActionBase);

unsafe impl InstActionAny for InstActionMove {
    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::ActionMove
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>) {
        self.animations().for_each(|anime| animations.push(anime));
    }

    fn derives(&self, _derives: &mut Vec<(VirtualKey, TmplID)>) {}
}

impl InstActionMove {
    pub(crate) fn try_assemble(ctx: &ContextActionAssemble<'_>, tmpl: At<TmplActionMove>) -> Option<InstActionMove> {
        if !ctx.solve_var(&tmpl.enabled) {
            return None;
        }

        Some(InstActionMove {
            _base: InstActionBase {
                tmpl_id: tmpl.id,
                enter_key: Some(VirtualKeyDir::new(VirtualKey::Run, None)),
                enter_level: tmpl.enter_level.into(),
                ..Default::default()
            },
            anim_move: InstAnimation::from_rkyv(&tmpl.anim_move),
            anim_turn_left: tmpl.anim_turn_left.as_ref().map(InstAnimation::from_rkyv),
            anim_turn_right: tmpl.anim_turn_right.as_ref().map(InstAnimation::from_rkyv),
            anim_stop: tmpl.anim_stop.as_ref().map(InstAnimation::from_rkyv),
            yam_time: tmpl.yam_time.into(),
            turn_time: tmpl.turn_time.into(),
            derive_level: tmpl.derive_level.into(),
            poise_level: tmpl.poise_level.into(),
        })
    }

    #[inline]
    pub fn animations(&self) -> InstActionMoveIter<'_> {
        InstActionMoveIter::new(self)
    }
}

pub struct InstActionMoveIter<'t> {
    action: &'t InstActionMove,
    idx: usize,
}

impl<'t> InstActionMoveIter<'t> {
    fn new(action: &'t InstActionMove) -> Self {
        Self { action, idx: 0 }
    }
}

impl<'t> Iterator for InstActionMoveIter<'t> {
    type Item = &'t InstAnimation;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let idx = self.idx;
            self.idx += 1;
            return match idx {
                0 => Some(&self.action.anim_move),
                1 => continue_if_none!(&self.action.anim_turn_left),
                2 => continue_if_none!(&self.action.anim_turn_right),
                3 => continue_if_none!(&self.action.anim_stop),
                _ => None,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::{TmplDatabase, TmplHashMap};
    use crate::utils::{id, sb, LEVEL_MOVE};
    use ahash::HashMapExt;

    #[test]
    fn test_assemble() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let var_indexes = TmplHashMap::new();

        let tmpl_act = db.find_as::<TmplActionMove>(id!("Action.Instance.Run/1A")).unwrap();
        let ctx = ContextActionAssemble {
            var_indexes: &var_indexes,
        };
        let inst_act = InstActionMove::try_assemble(&ctx, tmpl_act).unwrap();
        assert_eq!(inst_act.tmpl_id, id!("Action.Instance.Run/1A"));
        assert_eq!(inst_act.enter_key.unwrap(), VirtualKeyDir::new(VirtualKey::Run, None));
        assert_eq!(inst_act.enter_level, LEVEL_MOVE);
        assert_eq!(inst_act.anim_move.files, sb!("girl_run"));
        assert_eq!(inst_act.anim_move.duration, 3.0);
        assert_eq!(inst_act.anim_move.fade_in, 0.2);
        assert!(inst_act.anim_turn_left.is_none());
        assert!(inst_act.anim_turn_right.is_none());
        assert!(inst_act.anim_stop.is_none());
        assert_eq!(inst_act.yam_time, 0.4);
        assert_eq!(inst_act.turn_time, 1.0);
        assert_eq!(inst_act.derive_level, LEVEL_MOVE);
        assert_eq!(inst_act.poise_level, 0);
    }
}
