use cirtical_point_csgen::CsOut;
use jolt_physics_rs::BodyID;
use std::sync::Arc;

use crate::instance::{assemble_stage, InstStage};
use crate::logic::base::{ArchivedStateAny, LogicAny, LogicType, StateAny, StateAnyBase, StateType};
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::parameter::ParamStage;
use crate::template::TmplStage;
use crate::utils::{extend, NumID, XResult};

#[repr(C)]
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateStageInit {
    pub _base: StateAnyBase,
    pub view_stage_file: String,
}

extend!(StateStageInit, StateAnyBase);

unsafe impl StateAny for StateStageInit {
    #[inline]
    fn typ(&self) -> StateType {
        assert_eq!(self.typ, StateType::StageInit);
        StateType::StageInit
    }

    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        assert_eq!(self.logic_typ, LogicType::Stage);
        LogicType::Stage
    }
}

impl ArchivedStateAny for rkyv::Archived<StateStageInit> {
    #[inline]
    fn typ(&self) -> StateType {
        StateType::StageInit
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        LogicType::Stage
    }
}

#[repr(C)]
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateStageUpdate {
    pub _base: StateAnyBase,
}

extend!(StateStageUpdate, StateAnyBase);

unsafe impl StateAny for StateStageUpdate {
    #[inline]
    fn typ(&self) -> StateType {
        assert!(self.typ == StateType::StageUpdate);
        StateType::StageUpdate
    }

    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        assert!(self.logic_typ == LogicType::Stage);
        LogicType::Stage
    }
}

impl ArchivedStateAny for rkyv::Archived<StateStageUpdate> {
    #[inline]
    fn typ(&self) -> StateType {
        StateType::StageUpdate
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        LogicType::Stage
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
    fn typ(&self) -> LogicType {
        LogicType::Stage
    }

    #[inline]
    fn spawn_frame(&self) -> u32 {
        0
    }

    #[inline]
    fn death_frame(&self) -> u32 {
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
        let tmpl_stage = ctx.tmpl_db.find_as::<TmplStage>(&inst_stage.tmpl_stage)?;

        let phy_bodies = ctx.asset.load_stage(&tmpl_stage.stage_file)?;
        for body_id in &phy_bodies {
            ctx.body_itf.add_body(*body_id, false);
        }

        let stage = Box::new(LogicStage {
            id: ctx.gene.gen_id(),
            inst: inst_stage,
            phy_bodies,
        });
        ctx.state_init(Arc::new(StateStageInit {
            _base: StateAnyBase::new(stage.id, StateType::StageInit, LogicType::Stage),
            view_stage_file: tmpl_stage.view_stage_file.clone(),
        }));
        ctx.state_update(Box::new(StateStageUpdate {
            _base: StateAnyBase::new(stage.id, StateType::StageUpdate, LogicType::Stage),
        }));
        Ok(stage)
    }

    pub fn restore_impl(&mut self, ctx: &ContextRestore) -> XResult<()> {
        assert!(ctx.find(self.id).is_ok());
        Ok(())
    }

    pub fn update_impl(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<()> {
        ctx.state_update(Box::new(StateStageUpdate {
            _base: StateAnyBase::new(self.id, StateType::StageUpdate, LogicType::Stage),
        }));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::test_utils::*;
    use crate::utils::{s, CastPtr, CastRef};

    #[test]
    fn test_stage_common() {
        let mut systems = mock_logic_systems();
        let mut ctx = ContextUpdate::new_empty(&mut systems);
        let param = ParamStage {
            stage: s!("Stage.Demo"),
        };
        let mut stage = LogicStage::new(&mut ctx, &param).unwrap();

        assert_eq!(ctx.state_set.inits.len(), 1);
        let state_init = &ctx.state_set.inits[0];
        let state_init = state_init.cast_to::<StateStageInit>().unwrap();
        assert_eq!(state_init.id, stage.id);
        assert_eq!(state_init.typ(), StateType::StageInit);
        assert_eq!(state_init.logic_typ(), LogicType::Stage);

        assert_eq!(ctx.state_set.updates.len(), 1);
        let state_update = &ctx.state_set.updates[0];
        let state_update = state_update.cast_ref::<StateStageUpdate>().unwrap();
        assert_eq!(state_update.id, stage.id);
        assert_eq!(state_update.typ(), StateType::StageUpdate);
        assert_eq!(state_update.logic_typ(), LogicType::Stage);

        let mut ctx = ContextUpdate::new_empty(&mut systems);
        stage.update(&mut ctx).unwrap();
        assert_eq!(ctx.state_set.updates.len(), 1);
        let state_update = &ctx.state_set.updates[0];
        assert_eq!(state_update.id, stage.id);
        assert_eq!(state_update.typ(), StateType::StageUpdate);
        assert_eq!(state_update.logic_typ(), LogicType::Stage);
    }
}
