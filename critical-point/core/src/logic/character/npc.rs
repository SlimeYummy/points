use cirtical_point_csgen::CsOut;

use crate::logic::base::{ArchivedStateAny, LogicAny, LogicType, StateAny, StateAnyBase, StateType};
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::utils::{extend, NumID, XResult};

#[repr(C)]
#[derive(Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateNpcInit {
    pub _base: StateAnyBase,
}

extend!(StateNpcInit, StateAnyBase);

unsafe impl StateAny for StateNpcInit {
    #[inline]
    fn typ(&self) -> StateType {
        assert_eq!(self.typ, StateType::NpcInit);
        StateType::NpcInit
    }

    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        assert_eq!(self.logic_typ, LogicType::Npc);
        LogicType::Npc
    }
}

impl ArchivedStateAny for rkyv::Archived<StateNpcInit> {
    #[inline]
    fn typ(&self) -> StateType {
        StateType::NpcInit
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        LogicType::Npc
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateNpcUpdate {
    pub _base: StateAnyBase,
}

extend!(StateNpcUpdate, StateAnyBase);

unsafe impl StateAny for StateNpcUpdate {
    #[inline]
    fn typ(&self) -> StateType {
        assert!(self.typ == StateType::NpcUpdate);
        StateType::NpcUpdate
    }

    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        assert_eq!(self.logic_typ, LogicType::Npc);
        LogicType::Npc
    }
}

impl ArchivedStateAny for rkyv::Archived<StateNpcUpdate> {
    #[inline]
    fn typ(&self) -> StateType {
        StateType::NpcUpdate
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        LogicType::Npc
    }
}

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
