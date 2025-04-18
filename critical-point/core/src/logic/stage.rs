use cirtical_point_csgen::CsOut;
use jolt_physics_rs::BodyID;
use std::sync::Arc;

use crate::instance::{assemble_stage, InstStage};
use crate::logic::base::{ArchivedStateAny, LogicAny, LogicType, StateAny, StateAnyBase, StateType};
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::parameter::ParamStage;
use crate::template::TmplZone;
use crate::utils::{extend, NumID, XResult, STAGE_ID};

#[repr(C)]
#[derive(Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
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
#[derive(Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
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
}

impl LogicStage {
    pub fn new(ctx: &mut ContextUpdate<'_>, param: &ParamStage) -> XResult<(Box<LogicStage>, Arc<dyn StateAny>)> {
        let inst_stage = assemble_stage(&mut ctx.context_assemble(), param)?;
        let tmpl_stage = ctx.tmpl_db.find_as::<TmplZone>(&inst_stage.tmpl_stage)?;

        let asset = &mut ctx.systems.asset;
        let physics = &mut ctx.systems.physics;
        let phy_bodies = asset.load_stage(&tmpl_stage.stage_file, physics.body_itf())?.bodies;
        for body_id in &phy_bodies {
            ctx.physics.body_itf().add_body(*body_id, false);
        }

        let stage = Box::new(LogicStage {
            id: STAGE_ID,
            inst: inst_stage,
            phy_bodies,
        });
        let state = Arc::new(StateStageInit {
            _base: StateAnyBase::new(stage.id, StateType::StageInit, LogicType::Stage),
            view_stage_file: tmpl_stage.view_stage_file.clone(),
        });
        Ok((stage, state))
    }

    pub fn state(&mut self) -> Box<StateStageUpdate> {
        Box::new(StateStageUpdate {
            _base: StateAnyBase::new(self.id, StateType::StageUpdate, LogicType::Stage),
        })
    }

    pub fn restore(&mut self, ctx: &ContextRestore) -> XResult<()> {
        assert!(ctx.find(self.id).is_ok());
        Ok(())
    }

    pub fn update(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::test_utils::*;
    use crate::utils::{sb, CastPtr, CastRef};

    #[test]
    fn test_stage_common() {
        let mut systems = mock_logic_systems();
        let mut ctx = ContextUpdate::new(&mut systems, 0, 0);
        let param = ParamStage {
            stage: sb!("Stage.Demo"),
        };
        let (mut stage, state_init) = LogicStage::new(&mut ctx, &param).unwrap();

        let state_init = state_init.cast_to::<StateStageInit>().unwrap();
        assert_eq!(state_init.id, stage.id);
        assert_eq!(state_init.typ(), StateType::StageInit);
        assert_eq!(state_init.logic_typ(), LogicType::Stage);

        let state_update = stage.state();
        assert_eq!(state_update.id, stage.id);
        assert_eq!(state_update.typ(), StateType::StageUpdate);
        assert_eq!(state_update.logic_typ(), LogicType::Stage);

        let mut ctx = ContextUpdate::new(&mut systems, 1, 0);
        stage.update(&mut ctx).unwrap();
        let state_update = stage.state();
        assert_eq!(state_update.id, stage.id);
        assert_eq!(state_update.typ(), StateType::StageUpdate);
        assert_eq!(state_update.logic_typ(), LogicType::Stage);
    }
}
