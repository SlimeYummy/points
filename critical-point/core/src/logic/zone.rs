use cirtical_point_csgen::CsOut;
use jolt_physics_rs::BodyID;
use std::sync::Arc;

use crate::instance::{assemble_zone, InstZone};
use crate::logic::base::{impl_state, LogicAny, LogicType, StateAny, StateBase, StateType};
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::parameter::ParamZone;
use crate::template::TmplZone;
use crate::utils::{extend, NumID, XResult, STAGE_ID};

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateZoneInit {
    pub _base: StateBase,
    pub view_zone_file: String,
}

extend!(StateZoneInit, StateBase);

impl_state!(StateZoneInit, Zone, ZoneInit, "ZoneInit");

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateZoneUpdate {
    pub _base: StateBase,
}

extend!(StateZoneUpdate, StateBase);

impl_state!(StateZoneUpdate, Zone, ZoneUpdate, "ZoneUpdate");

#[derive(Debug)]
pub struct LogicZone {
    id: NumID,
    inst: InstZone,
    phy_bodies: Vec<BodyID>,
}

impl LogicAny for LogicZone {
    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn typ(&self) -> LogicType {
        LogicType::Zone
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

impl LogicZone {
    pub fn new(ctx: &mut ContextUpdate<'_>, param: &ParamZone) -> XResult<(Box<LogicZone>, Arc<dyn StateAny>)> {
        let inst_zone = assemble_zone(&mut ctx.context_assemble(), param)?;
        let tmpl_zone = ctx.tmpl_db.find_as::<TmplZone>(inst_zone.tmpl_zone)?;

        let asset = &mut ctx.systems.asset;
        let physics = &mut ctx.systems.physics;
        let phy_bodies = asset.load_zone(&tmpl_zone.zone_file, physics.body_itf())?.bodies;
        for body_id in &phy_bodies {
            ctx.physics.body_itf().add_body(*body_id, false);
        }

        let zone = Box::new(LogicZone {
            id: STAGE_ID,
            inst: inst_zone,
            phy_bodies,
        });
        let state = Arc::new(StateZoneInit {
            _base: StateBase::new(zone.id, StateType::ZoneInit, LogicType::Zone),
            view_zone_file: tmpl_zone.view_zone_file.to_owned(),
        });
        Ok((zone, state))
    }

    pub fn state(&mut self) -> Box<StateZoneUpdate> {
        Box::new(StateZoneUpdate {
            _base: StateBase::new(self.id, StateType::ZoneUpdate, LogicType::Zone),
        })
    }

    pub fn restore(&mut self, ctx: &ContextRestore) -> XResult<()> {
        assert!(ctx.find(self.id).is_ok());
        Ok(())
    }

    pub fn update(&mut self, _ctx: &mut ContextUpdate<'_>) -> XResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::test_utils::*;
    use crate::utils::{id, Castable};

    #[test]
    fn test_zone_common() {
        let mut tenv = TestEnv::new().unwrap();
        let mut ctx = tenv.context_update();
        let param = ParamZone { zone: id!("Zone.Demo") };
        let (mut zone, state_init) = LogicZone::new(&mut ctx, &param).unwrap();

        let state_init = state_init.clone().cast::<StateZoneInit>().unwrap();
        assert_eq!(state_init.id, zone.id);
        assert_eq!(state_init.typ(), StateType::ZoneInit);
        assert_eq!(state_init.logic_typ(), LogicType::Zone);

        let state_update = zone.state();
        assert_eq!(state_update.id, zone.id);
        assert_eq!(state_update.typ(), StateType::ZoneUpdate);
        assert_eq!(state_update.logic_typ(), LogicType::Zone);

        let mut ctx = tenv.context_update();
        zone.update(&mut ctx).unwrap();
        let state_update = zone.state();
        assert_eq!(state_update.id, zone.id);
        assert_eq!(state_update.typ(), StateType::ZoneUpdate);
        assert_eq!(state_update.logic_typ(), LogicType::Zone);
    }
}
