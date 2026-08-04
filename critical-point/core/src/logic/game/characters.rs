use glam::Vec3A;
use glam_ext::Vec2xz;
use std::hint::unlikely;

use crate::logic::character::LogicCharacter;
use crate::utils::{HistoryVec, HistoryVecRest, ShapeSphere, ShapeSphericalCone};

struct StaticWrapper(HistoryVec<Box<LogicCharacter>>);
unsafe impl Sync for StaticWrapper {}
unsafe impl Send for StaticWrapper {}

static STATIC_WRAPPER: StaticWrapper = StaticWrapper(HistoryVec::new());

impl<'t> HistoryVecRest<'t, Box<LogicCharacter>> {
    pub(super) const fn empty() -> HistoryVecRest<'static, Box<LogicCharacter>> {
        unsafe { HistoryVecRest::new(&STATIC_WRAPPER.0, usize::MAX) }
    }

    pub fn search_chara_in_sphere(
        &'t self,
        is_player: bool,
        sphere: &ShapeSphere,
        center: Vec3A,
        indexes: &mut Vec<u32>,
    ) {
        // TODO: use octree to optimize search
        // TODO: use team instead of is_player

        for (idx, chara) in self.index_iter() {
            if is_player != chara.is_player() {
                continue;
            }

            let dist_sq = (chara.physics().position() - center).length_squared();
            if dist_sq <= sphere.radius_sq() {
                indexes.push(idx as u32);
            }
        }
    }

    pub fn search_chara_in_spherical_cone(
        &'t self,
        is_player: bool,
        cone: &ShapeSphericalCone,
        center: Vec3A,
        direction: Vec2xz,
        indexes: &mut Vec<u32>,
    ) {
        const LEN_THRESHOLD_SQ: f32 = 1e-6;
        // TODO: use octree to optimize search
        // TODO: use team instead of is_player

        let direction = direction.as_vec3a();
        if unlikely(direction.length_squared() < LEN_THRESHOLD_SQ) {
            return;
        }
        let dir_len = direction.length();

        let cos_half_angle = cone.half_angle.cos();
        for (idx, chara) in self.index_iter() {
            if is_player != chara.is_player() {
                continue;
            }

            let diff = chara.physics().position() - center;
            let dist_sq = diff.length_squared();
            if dist_sq > cone.radius_sq() {
                continue;
            }

            if dist_sq < LEN_THRESHOLD_SQ {
                indexes.push(idx as u32);
                continue;
            }

            let dot = diff.dot(direction) / (dist_sq.sqrt() * dir_len);
            if dot >= cos_half_angle {
                indexes.push(idx as u32);
            }
        }
    }
}
