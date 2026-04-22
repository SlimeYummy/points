use glam::Vec3A;
use jolt_physics_rs::{
    Body, CollideShapeResult, ContactListener, ContactListenerVTable, ContactManifold, ContactSettings, JVec3,
    SubShapeID, SubShapeIDPair, ValidateResult, vdata,
};
use static_assertions::const_assert_eq;
use std::mem;

use crate::logic::game::LogicGame;
use crate::utils::NumID;

#[repr(align(8))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PhyBodyUserData {
    None,
    Zone,
    Hit { chara_id: NumID, hit: u16 },
    Character { id: NumID },
    _Padding_([u8; 7]),
}

const_assert_eq!(mem::size_of::<PhyBodyUserData>(), 8);

impl PhyBodyUserData {
    pub(crate) fn new_zone() -> PhyBodyUserData {
        PhyBodyUserData::Zone
    }

    pub(crate) fn new_hit(chara_id: NumID, hit: u16) -> PhyBodyUserData {
        PhyBodyUserData::Hit { chara_id, hit }
    }

    pub(crate) fn new_character(id: NumID) -> PhyBodyUserData {
        PhyBodyUserData::Character { id }
    }
}

const PHY_BODY_USER_DATA_PADDING: PhyBodyUserData = PhyBodyUserData::_Padding_([0; 7]);

impl From<PhyBodyUserData> for u64 {
    fn from(value: PhyBodyUserData) -> Self {
        unsafe { mem::transmute(value) }
    }
}

impl From<u64> for PhyBodyUserData {
    fn from(num: u64) -> PhyBodyUserData {
        let raw_padding: [u8; 8] = unsafe { mem::transmute(PHY_BODY_USER_DATA_PADDING) };
        let raw_num: [u8; 8] = num.to_ne_bytes();
        if raw_num[0] >= raw_padding[0] {
            return PhyBodyUserData::None;
        }
        unsafe { mem::transmute::<u64, PhyBodyUserData>(num) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PhyContactState {
    Added,
    Persisted,
    Removed,
}

#[derive(Debug)]
pub(crate) struct PhyHitCharacterEvent<'t> {
    pub(crate) src_chara_id: NumID,
    pub(crate) src_box_index: u16,
    pub(crate) dst_chara_id: NumID,
    pub(crate) src_body: &'t Body,
    pub(crate) dst_body: &'t Body,
    pub(crate) src_sub_shape_id1: SubShapeID,
    pub(crate) dst_sub_shape_id2: SubShapeID,
    pub(crate) world_space_normal: Vec3A,
    pub(crate) penetration_depth: f32,
    pub(crate) collision_point_average: Vec3A,
}

#[vdata(ContactListenerVTable)]
pub(crate) struct PhyContactCollector<'t> {
    game: &'t mut LogicGame,
}

impl<'t> PhyContactCollector<'t> {
    pub(crate) fn new(game: &'t mut LogicGame) -> PhyContactCollector<'t> {
        PhyContactCollector { game }
    }

    fn handle_contact(&mut self, body1: &Body, body2: &Body, manifold: &ContactManifold) {
        use PhyBodyUserData::*;

        let ud1 = PhyBodyUserData::from(body1.get_user_data());
        let ud2 = PhyBodyUserData::from(body2.get_user_data());

        let res = match (ud1, ud2) {
            (
                Hit {
                    chara_id: src_chara_id,
                    hit,
                },
                Character { id: dst_chara_id },
            ) => self.game.on_hit_character(&PhyHitCharacterEvent {
                src_chara_id,
                src_box_index: hit,
                dst_chara_id,
                src_body: body1,
                dst_body: body2,
                src_sub_shape_id1: manifold.sub_shape_id1,
                dst_sub_shape_id2: manifold.sub_shape_id2,
                world_space_normal: manifold.world_space_normal,
                penetration_depth: manifold.penetration_depth,
                collision_point_average: Self::calc_collision_point_average(manifold),
            }),
            (
                Character { id: dst_chara_id },
                Hit {
                    chara_id: src_chara_id,
                    hit,
                },
            ) => self.game.on_hit_character(&PhyHitCharacterEvent {
                src_chara_id,
                src_box_index: hit,
                dst_chara_id,
                src_body: body2,
                dst_body: body1,
                src_sub_shape_id1: manifold.sub_shape_id2,
                dst_sub_shape_id2: manifold.sub_shape_id1,
                world_space_normal: -manifold.world_space_normal,
                penetration_depth: manifold.penetration_depth,
                collision_point_average: Self::calc_collision_point_average(manifold),
            }),
            _ => Ok(()),
        };

        if let Err(e) = res {
            log::error!("PhyContactCollector::handle_contact: {:?}", e);
        }
    }

    fn calc_collision_point_average(manifold: &ContactManifold) -> Vec3A {
        let mut total_pt = Vec3A::ZERO;
        for pt in manifold.relative_contact_points_on1.iter() {
            total_pt += pt;
        }
        for pt in manifold.relative_contact_points_on2.iter() {
            total_pt += pt;
        }
        manifold.base_offset
            + total_pt
                / (manifold.relative_contact_points_on1.len() + manifold.relative_contact_points_on2.len()) as f32
    }
}

impl<'t> ContactListener for PhyContactCollector<'t> {
    fn on_contact_validate(
        &mut self,
        _body1: &Body,
        _body2: &Body,
        _base_offset: JVec3,
        _collision_result: &CollideShapeResult,
    ) -> ValidateResult {
        ValidateResult::AcceptAllContactsForThisBodyPair
    }

    fn on_contact_added(
        &mut self,
        body1: &Body,
        body2: &Body,
        manifold: &ContactManifold,
        _settings: &mut ContactSettings,
    ) {
        self.handle_contact(body1, body2, manifold);
    }

    fn on_contact_persisted(
        &mut self,
        body1: &Body,
        body2: &Body,
        manifold: &ContactManifold,
        _settings: &mut ContactSettings,
    ) {
        self.handle_contact(body1, body2, manifold);
    }

    fn on_contact_removed(&mut self, _pair: &SubShapeIDPair) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phy_body_user_date_layout() {
        let zero = unsafe { mem::transmute::<u64, PhyBodyUserData>(0) };
        assert!(matches!(zero, PhyBodyUserData::None));

        let raw_padding: [u8; 8] = unsafe { mem::transmute(PHY_BODY_USER_DATA_PADDING) };
        assert_eq!(raw_padding[0], 4);

        let none_u64: u64 = PhyBodyUserData::None.into();
        let none = PhyBodyUserData::try_from(none_u64).unwrap();
        assert_eq!(none, PhyBodyUserData::None);

        let stage_u64: u64 = PhyBodyUserData::Zone.into();
        let stage = PhyBodyUserData::try_from(stage_u64).unwrap();
        assert_eq!(stage, PhyBodyUserData::Zone);

        let hit_u64: u64 = PhyBodyUserData::Hit {
            chara_id: NumID(1234),
            hit: 99,
        }
        .into();
        let hit = PhyBodyUserData::try_from(hit_u64).unwrap();
        assert_eq!(hit, PhyBodyUserData::Hit {
            chara_id: NumID(1234),
            hit: 99
        });

        let character_u64: u64 = PhyBodyUserData::Character { id: NumID(7777) }.into();
        let character = PhyBodyUserData::try_from(character_u64).unwrap();
        assert_eq!(character, PhyBodyUserData::Character { id: NumID(7777) });
    }
}
