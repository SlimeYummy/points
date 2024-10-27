use cirtical_point_csgen::CsGen;
use jolt_physics_rs::BodyID;
use std::sync::Arc;

use crate::instance::{assemble_stage, InstStage};
use crate::logic::base::{ArchivedStateAny, LogicAny, LogicClass, StateAny, StateAnyBase, StateClass};
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::parameter::ParamStage;
use crate::utils::{extend, NumID, XResult};

#[repr(C)]
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsGen)]
#[archive_attr(derive(Debug))]
#[cs_attr(Rs, Ref, Arc)]
pub struct StateStageInit {
    pub _base: StateAnyBase,
}

extend!(StateStageInit, StateAnyBase);

unsafe impl StateAny for StateStageInit {
    #[inline]
    fn class(&self) -> StateClass {
        assert_eq!(self.class, StateClass::StageInit);
        StateClass::StageInit
    }

    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn logic_class(&self) -> LogicClass {
        assert_eq!(self.logic_class, LogicClass::Stage);
        LogicClass::Stage
    }
}

impl ArchivedStateAny for rkyv::Archived<StateStageInit> {
    #[inline]
    fn class(&self) -> StateClass {
        StateClass::StageInit
    }

    #[inline]
    fn logic_class(&self) -> LogicClass {
        LogicClass::Stage
    }
}

#[repr(C)]
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsGen)]
#[archive_attr(derive(Debug))]
#[cs_attr(Rs, Ref, Arc)]
pub struct StateStageUpdate {
    pub _base: StateAnyBase,
}

extend!(StateStageUpdate, StateAnyBase);

unsafe impl StateAny for StateStageUpdate {
    #[inline]
    fn class(&self) -> StateClass {
        assert!(self.class == StateClass::StageUpdate);
        StateClass::StageUpdate
    }

    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn logic_class(&self) -> LogicClass {
        assert!(self.logic_class == LogicClass::Stage);
        LogicClass::Stage
    }
}

impl ArchivedStateAny for rkyv::Archived<StateStageUpdate> {
    #[inline]
    fn class(&self) -> StateClass {
        StateClass::StageUpdate
    }

    #[inline]
    fn logic_class(&self) -> LogicClass {
        LogicClass::Stage
    }
}

#[derive(Debug)]
pub struct LogicStage {
    id: NumID,
    inst: InstStage,
    phy_bodies: Vec<BodyID>,
}

impl LogicAny for LogicStage {
    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn class(&self) -> LogicClass {
        LogicClass::Stage
    }

    #[inline]
    fn spawn_frame(&self) -> u32 {
        0
    }

    #[inline]
    fn dead_frame(&self) -> u32 {
        u32::MAX
    }

    #[inline]
    fn update(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<()> {
        self.update_impl(ctx)
    }

    #[inline]
    fn restore(&mut self, ctx: &ContextRestore) -> XResult<()> {
        self.restore_impl(ctx)
    }
}

impl LogicStage {
    pub fn new(ctx: &mut ContextUpdate<'_>, param: &ParamStage) -> XResult<Box<LogicStage>> {
        let inst_stage = assemble_stage(&mut ctx.context_assemble(), param)?;
        let phy_bodies = ctx.asset.load_stage(&inst_stage.asset_id)?;
        for body_id in &phy_bodies {
            ctx.body_itf.add_body(*body_id, false);
        }

        let stage = Box::new(LogicStage {
            id: ctx.gene.gen_id(),
            inst: inst_stage,
            phy_bodies,
        });
        ctx.state_init(Arc::new(StateStageInit {
            _base: StateAnyBase::new(stage.id, StateClass::StageInit, LogicClass::Stage),
        }));
        ctx.state_update(Box::new(StateStageUpdate {
            _base: StateAnyBase::new(stage.id, StateClass::StageUpdate, LogicClass::Stage),
        }));
        Ok(stage)
    }

    pub fn restore_impl(&mut self, ctx: &ContextRestore) -> XResult<()> {
        assert!(ctx.find(self.id).is_ok());
        Ok(())
    }

    pub fn update_impl(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<()> {
        ctx.state_update(Box::new(StateStageUpdate {
            _base: StateAnyBase::new(self.id, StateClass::StageUpdate, LogicClass::Stage),
        }));
        Ok(())
    }
}
