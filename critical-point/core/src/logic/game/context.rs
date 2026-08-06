use critical_point_macros::csharp_out;
use glam::Vec3A;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use crate::consts::FPS;
use crate::instance::ContextAssemble;
use crate::logic::base::StateAny;
use crate::logic::character::LogicCharacter;
use crate::logic::game::game::LogicSystems;
use crate::logic::system::StateSet;
use crate::logic::zone::LogicZone;
use crate::utils::{HistoryVecRest, NumID, Symbol, XResult, force_mut};

//
// Context Update
//

pub struct ContextUpdate<'t> {
    pub(crate) systems: &'t mut LogicSystems,
    pub(crate) time: &'t GameTime,
}

impl Deref for ContextUpdate<'_> {
    type Target = LogicSystems;

    fn deref(&self) -> &Self::Target {
        self.systems
    }
}

impl DerefMut for ContextUpdate<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.systems
    }
}

impl<'t> ContextUpdate<'t> {
    #[inline]
    pub(crate) fn new(systems: &'t mut LogicSystems, time: &'t GameTime) -> ContextUpdate<'t> {
        ContextUpdate { systems, time }
    }

    #[inline]
    pub(crate) fn context_assemble(&mut self) -> ContextAssemble<'_> {
        ContextAssemble {
            tmpl_db: &self.systems.tmpl_db,
            // executor: &mut self.systems.executor,
        }
    }
}

pub struct ContextUpdateEx<'t> {
    pub(crate) systems: &'t mut LogicSystems,
    pub(crate) time: &'t GameTime,
    pub(crate) zone: &'t LogicZone,
    pub(crate) characters: HistoryVecRest<'t, Box<LogicCharacter>>,
    pub(crate) hit_events: &'t [HitCharacterEvent],
}

impl Deref for ContextUpdateEx<'_> {
    type Target = LogicSystems;

    fn deref(&self) -> &Self::Target {
        self.systems
    }
}

impl DerefMut for ContextUpdateEx<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.systems
    }
}

impl<'t> ContextUpdateEx<'t> {
    #[inline]
    pub(crate) fn new(systems: &'t mut LogicSystems, time: &'t GameTime, zone: &'t LogicZone) -> ContextUpdateEx<'t> {
        ContextUpdateEx {
            systems,
            time,
            zone,
            characters: HistoryVecRest::empty(),
            hit_events: &[],
        }
    }

    #[inline]
    pub(crate) fn context_assemble(&mut self) -> ContextAssemble<'_> {
        ContextAssemble {
            tmpl_db: &self.systems.tmpl_db,
            // executor: &mut self.systems.executor,
        }
    }

    // Safety: test only.
    #[cfg(test)]
    pub(crate) fn time_mut(&mut self) -> &mut GameTime {
        unsafe { force_mut(self.time) }
    }
}

//
// Game Time
//

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct GameTime {
    pub(crate) frame: u32,
    pub(crate) synced_frame: u32,
    pub(crate) time: f32,
    pub(crate) synced_time: f32,
}

impl GameTime {
    #[inline]
    pub(crate) fn new(frame: u32, synced_frame: u32) -> GameTime {
        GameTime {
            frame,
            synced_frame,
            time: frame as f32 / FPS,
            synced_time: synced_frame as f32 / FPS,
        }
    }
}

//
// Context Restore
//

pub struct ContextRestore {
    pub frame: u32,
    pub(crate) state_set: Arc<StateSet>,
}

impl ContextRestore {
    #[inline]
    pub fn new(state_set: Arc<StateSet>) -> ContextRestore {
        ContextRestore {
            frame: state_set.frame,
            state_set,
        }
    }

    #[inline]
    pub fn find(&self, id: NumID) -> XResult<&dyn StateAny> {
        self.state_set.find(id)
    }

    #[inline]
    pub fn find_as<T: StateAny + 'static>(&self, id: NumID) -> XResult<&T> {
        self.state_set.find_as(id)
    }
}

//
// Context Hit
//

pub struct ContextHitGenerate<'t, E> {
    pub(crate) frame: u32,
    pub(crate) time: f32,
    pub(crate) events: &'t mut Vec<E>,
}

impl<'t, E> ContextHitGenerate<'t, E> {
    #[inline]
    pub(crate) fn new(frame: u32, events: &'t mut Vec<E>) -> ContextHitGenerate<'t, E> {
        ContextHitGenerate {
            frame,
            time: frame as f32 / FPS,
            events,
        }
    }

    #[inline]
    pub(crate) fn context_update(&mut self, idx: usize) -> ContextHitUpdate<'_, E> {
        ContextHitUpdate::new(self.frame, &mut self.events[idx])
    }
}

pub struct ContextHitUpdate<'t, E> {
    pub(crate) frame: u32,
    pub(crate) time: f32,
    pub(crate) event: &'t mut E,
}

impl<'t, E> ContextHitUpdate<'t, E> {
    #[inline]
    pub(crate) fn new(frame: u32, event: &'t mut E) -> ContextHitUpdate<'t, E> {
        ContextHitUpdate {
            frame,
            time: frame as f32 / FPS,
            event,
        }
    }
}

//
// Hit Event
//

#[repr(C)]
#[csharp_out(Value)]
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct HitCharacterEvent {
    pub src_chara_id: NumID,
    pub dst_chara_id: NumID,
    pub group: Symbol,
    pub box_index: u16,
    pub group_index: u16,
    pub box_hit_times: u16,
    pub group_hit_times: u16,
    // Normal for this collision, direction along which to move dst_chara out of collision along the shortest path.
    pub collision_normal: Vec3A,
    // The average position of all collision points
    pub collision_point_average: Vec3A,
    // The vector pointing from the src_chara position to the dst_chara position.
    pub character_vector: Vec3A,
}
