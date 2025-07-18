use cirtical_point_csgen::CsOut;

use crate::logic::base::{impl_state, LogicAny, LogicType, StateBase};
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::utils::{extend, NumID, XResult};

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateNpcInit {
    pub _base: StateBase,
}

extend!(StateNpcInit, StateBase);

impl_state!(StateNpcInit, Npc, NpcInit, "NpcInit");

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateNpcUpdate {
    pub _base: StateBase,
}

extend!(StateNpcUpdate, StateBase);

impl_state!(StateNpcUpdate, Npc, NpcUpdate, "NpcUpdate");

#[derive(Debug)]
pub struct LogicNpc {
    id: NumID,
    spawn_frame: u32,
    death_frame: u32,
}

impl LogicAny for LogicNpc {
    #[inline]
    fn typ(&self) -> LogicType {
        LogicType::Npc
    }

    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn spawn_frame(&self) -> u32 {
        self.spawn_frame
    }

    #[inline]
    fn death_frame(&self) -> u32 {
        self.death_frame
    }
}

impl LogicNpc {
    pub fn state(&mut self) -> Box<StateNpcUpdate> {
        unimplemented!();
    }

    pub fn restore(&mut self, _ctx: &ContextRestore) -> XResult<()> {
        unimplemented!();
    }

    pub fn update_ai(&mut self, _ctx: &mut ContextUpdate<'_>) -> XResult<()> {
        unimplemented!();
    }
}
