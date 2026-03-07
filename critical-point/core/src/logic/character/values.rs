use std::rc::Rc;

use critical_point_csgen::CsOut;

use crate::animation::HitMotion;
use crate::instance::InstCharacter;
use crate::logic::PhyHitCharacterEvent;
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::logic::character::LogicCharaPhysics;
use crate::utils::{NumID, Symbol};

struct LogicCharaHit {
}

#[repr(C)]
#[derive(
    Debug,
    Default,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsOut,
)]
#[rkyv(derive(Debug))]
pub struct StateCharaValues {
    
}

#[repr(C)]
#[derive(
    Debug,
    Default,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsOut,
)]
#[rkyv(derive(Debug))]
pub struct StateCharaHitGroup {
    name: Symbol,
    idx: u16,
    trigger_times: u16,
}

#[derive(Debug)]
pub(crate) struct LogicCharaValues {
    chara_id: NumID,
    inst_chara: Rc<InstCharacter>,
    hit_groups: Vec<StateCharaHitGroup>,
    hit_motion: Option<Rc<HitMotion>>,
}

impl LogicCharaValues {
    pub(crate) fn new(
        ctx: &mut ContextUpdate,
        chara_id: NumID,
        inst_chara: Rc<InstCharacter>,
    ) -> LogicCharaValues {
        LogicCharaValues {
            chara_id,
            inst_chara,
            hit_groups: Vec::with_capacity(8),
            hit_motion: None,
        }
    }

    pub(crate) fn before_hit(&mut self, physics: &LogicCharaPhysics, event: PhyHitCharacterEvent) {
        if let Some(hit_motion) = &self.hit_motion {
            if let Some(track) = hit_motion.get_track(event.src_hit_id) {
                self.hit_groups[track.group_index as usize].trigger_times += 1;
            }
        }
    }

    pub(crate) fn on_hit(&self) {}

    pub(crate) fn after_hit(&self) {}

    pub(crate) fn before_injure(&self) {}

    pub(crate) fn after_injure(&self) {}

    pub(crate) fn on_injure(&self) {}
}
