use critical_point_macros::csharp_out;
use glam::{Vec3, Vec3A};
use jolt_physics_rs::{BodyCreationSettings, BodyID};
use recastnavigation_rs::detour::{DtNavMesh, DtNavMeshQuery, DtPolyRef, DtQueryFilter};
use std::cell::RefCell;
use std::sync::Arc;

use crate::instance::InstZone;
use crate::logic::base::{LogicAny, LogicType, StateAny, StateBase, StateType, impl_state};
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::logic::physics::{PhyBodyUserData, phy_layer};
use crate::parameter::ParamZone;
use crate::template::TmplZone;
use crate::utils::{NumID, XResult, extend};

const MAX_POLYS: usize = 256;
const MAX_NODES: usize = 2048;

#[derive(Debug)]
struct NavMeshCache {
    query: DtNavMeshQuery,
    polys: Vec<DtPolyRef>,
    path: Vec<[f32; 3]>,
}

impl NavMeshCache {
    fn new(nav_mesh: &DtNavMesh) -> XResult<NavMeshCache> {
        Ok(NavMeshCache {
            query: DtNavMeshQuery::with_mesh(nav_mesh, MAX_NODES)?,
            polys: Vec::with_capacity(MAX_POLYS),
            path: Vec::with_capacity(MAX_POLYS),
        })
    }
}

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct StateZoneInit {
    pub _base: StateBase,
    pub view_file: String,
}

extend!(StateZoneInit, StateBase);

impl_state!(StateZoneInit, Zone, ZoneInit, "ZoneInit");

#[repr(C)]
#[csharp_out(Ref)]
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
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
    nav_mesh: DtNavMesh,
    cache: RefCell<NavMeshCache>,
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
    pub fn new(ctx: &mut ContextUpdate, param: &ParamZone) -> XResult<(Box<LogicZone>, Arc<dyn StateAny>)> {
        let inst_zone = InstZone::new(&mut ctx.context_assemble(), param)?;
        let tmpl_zone = ctx.tmpl_db.find_as::<TmplZone>(inst_zone.tmpl_zone)?;

        let asset = &mut ctx.systems.asset;
        let zone_phy = asset.load_zone_physics(inst_zone.files)?;

        let bofy_itf = &mut ctx.systems.physics.body_itf();

        let mut phy_bodies = Vec::with_capacity(zone_phy.bodies.len());
        for asset_body in &zone_phy.bodies {
            let mut settings = BodyCreationSettings::new_static(
                asset_body.shape.clone(),
                phy_layer!(StaticScenery, All),
                asset_body.position,
                asset_body.rotation,
            );
            settings.user_data = PhyBodyUserData::new_zone().into();

            let body_id = bofy_itf.create_add_body(&settings, false)?;
            phy_bodies.push(body_id);
        }

        let nav_mesh = asset.load_nav_mesh(inst_zone.files)?;
        let cache = RefCell::new(NavMeshCache::new(&nav_mesh)?);

        let zone = Box::new(LogicZone {
            id: NumID::STAGE,
            inst: inst_zone,
            phy_bodies,
            nav_mesh,
            cache,
        });
        let state = Arc::new(StateZoneInit {
            _base: StateBase::new(zone.id, StateType::ZoneInit, LogicType::Zone),
            view_file: tmpl_zone.view_file.to_owned(),
        });
        Ok((zone, state))
    }

    pub fn cleanup(&mut self, ctx: &mut ContextUpdate) -> XResult<()> {
        let bofy_itf = &mut ctx.systems.physics.body_itf();
        for body_id in self.phy_bodies.drain(..) {
            bofy_itf.remove_body(body_id);
        }
        Ok(())
    }

    pub fn state(&mut self) -> Box<StateZoneUpdate> {
        Box::new(StateZoneUpdate {
            _base: StateBase::new(self.id, StateType::ZoneUpdate, LogicType::Zone),
        })
    }

    pub fn restore(&mut self, ctx: &ContextRestore) -> XResult<()> {
        debug_assert!(ctx.find(self.id).is_ok());
        Ok(())
    }

    pub fn update(&mut self, _ctx: &mut ContextUpdate) -> XResult<()> {
        Ok(())
    }

    #[inline]
    pub fn nav_mesh(&self) -> &DtNavMesh {
        &self.nav_mesh
    }

    #[inline]
    pub fn find_path(&self, start_point: Vec3A, end_point: Vec3A, out_path: &mut Vec<Vec3>) -> XResult<()> {
        out_path.clear();

        let mut cache = self.cache.borrow_mut();
        let NavMeshCache { query, polys, path } = &mut *cache;

        // Find nearest polygons for start and end positions
        let extents = [2.0, 10.0, 2.0];
        let filter = DtQueryFilter::default();

        let start_pos = start_point.to_array();
        let end_pos = end_point.to_array();

        let (start_ref, _) = query.find_nearest_poly_1(&start_pos, &extents, &filter)?;
        let (end_ref, _) = query.find_nearest_poly_1(&end_pos, &extents, &filter)?;

        if start_ref.is_null() || end_ref.is_null() {
            return Ok(());
        }

        // Find the polygon path
        polys.clear();
        polys.resize(MAX_POLYS, DtPolyRef::default());
        let poly_count = query.find_path(start_ref, end_ref, &start_pos, &end_pos, &filter, polys)?;

        if poly_count == 0 {
            return Ok(());
        }

        // Calculate the real end position
        let mut real_end_pos = end_pos;
        if polys[poly_count - 1] != end_ref {
            let (closest, _) = query.closest_point_on_poly(polys[poly_count - 1], &end_pos)?;
            real_end_pos = closest;
        }

        // Find straight path
        path.clear();
        path.resize(MAX_POLYS, [0.0; 3]);
        let straight_path_count = query.find_straight_path(
            &start_pos,
            &real_end_pos,
            &polys[..poly_count],
            path,
            None,
            None,
            0, // No special options
        )?;

        // Write results to output
        out_path.clear();
        out_path.reserve(straight_path_count);
        for i in 0..straight_path_count {
            out_path.push(path[i].into());
        }
        Ok(())
    }

    #[inline]
    pub fn find_point(&self, point: Vec3A) -> XResult<Option<Vec3A>> {
        let mut cache = self.cache.borrow_mut();
        let NavMeshCache { query, .. } = &mut *cache;
        let extents = [2.0, 10.0, 2.0];
        let filter = DtQueryFilter::default();
        let pos = point.to_array();

        let (poly_ref, _) = query.find_nearest_poly_1(&pos, &extents, &filter)?;
        if poly_ref.is_null() {
            return Ok(None);
        }

        let (closest, _) = query.closest_point_on_poly(poly_ref, &pos)?;
        Ok(Some(Vec3A::from_array(closest)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::test_utils::*;
    use crate::utils::{Castable, id};

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
