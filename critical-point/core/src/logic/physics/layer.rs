use jolt_physics_rs::{
    vdata, BroadPhaseLayer, BroadPhaseLayerInterface, BroadPhaseLayerInterfaceVTable, ObjectLayer,
    ObjectLayerPairFilter, ObjectLayerPairFilterVTable, ObjectVsBroadPhaseLayerFilter,
    ObjectVsBroadPhaseLayerFilterVTable,
};

pub(crate) const PHY_BROAD_COUNT: u32 = 2;
pub(crate) const PHY_BROAD_STATIC: BroadPhaseLayer = 0;
pub(crate) const PHY_BROAD_DYNAMIC: BroadPhaseLayer = 1;

#[vdata(BroadPhaseLayerInterfaceVTable)]
pub(crate) struct PhyBroadPhaseLayerInterface;

impl BroadPhaseLayerInterface for PhyBroadPhaseLayerInterface {
    fn get_num_broad_phase_layers(&self) -> u32 {
        PHY_BROAD_COUNT
    }

    fn get_broad_phase_layer(&self, layer: ObjectLayer) -> BroadPhaseLayer {
        if extract_layer_type(layer) <= 1 {
            PHY_BROAD_STATIC
        }
        else {
            PHY_BROAD_DYNAMIC
        }
    }
}

#[macro_export]
macro_rules! phy_layer {
    ($typ:ident, $team:ident) => {
        (crate::logic::PhyLayerType::$typ as u32) | (crate::logic::PhyLayerTeam::$team as u32) << 8
    };
    ($typ:ident, $cond:expr => $team1:ident | $team2:ident) => {
        if $cond {
            (crate::logic::PhyLayerType::$typ as u32) | (crate::logic::PhyLayerTeam::$team1 as u32) << 8
        }
        else {
            (crate::logic::PhyLayerType::$typ as u32) | (crate::logic::PhyLayerTeam::$team1 as u32) << 8
        }
    };
}
pub use phy_layer;

const PHY_LAYER_TYPE_MASK: ObjectLayer = 0xFF;
const PHY_LAYER_TEAM_OFFSET: ObjectLayer = 8;

pub fn extract_layer_type(layer: ObjectLayer) -> u8 {
    (layer & PHY_LAYER_TYPE_MASK) as u8
}

pub fn extract_layer_team(layer: ObjectLayer) -> u32 {
    (layer >> PHY_LAYER_TEAM_OFFSET) as u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhyLayerTeam {
    Player = 0x1,
    Enemy = 0x2,
    All = 0xFF,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhyLayerType {
    StaticScenery = 0,
    StaticTrigger = 1,
    DynamicScenery = 2,
    BreakableScenery = 3,
    DynamicTrigger = 4,
    Bounding = 5,
    Target = 6,
    Hit = 7,
    HitEx = 8,
}

impl PhyLayerType {
    fn to_mask(typ: u8) -> u64 {
        const STATIC_SCENERY: u64 = 1 << (PhyLayerType::StaticScenery as u64);
        const STATIC_TRIGGER: u64 = 1 << (PhyLayerType::StaticTrigger as u64);
        const DYNAMIC_SCENERY: u64 = 1 << (PhyLayerType::DynamicScenery as u64);
        const BREAKABLE_SCENERY: u64 = 1 << (PhyLayerType::BreakableScenery as u64);
        const DYNAMIC_TRIGGER: u64 = 1 << (PhyLayerType::DynamicTrigger as u64);
        const BOUNDING: u64 = 1 << (PhyLayerType::Bounding as u64);
        const TARGET: u64 = 1 << (PhyLayerType::Target as u64);
        const HIT: u64 = 1 << (PhyLayerType::Hit as u64);
        const HIT_EX: u64 = 1 << (PhyLayerType::HitEx as u64);

        const TABLE: [u64; 9] = [
            // StaticScenery
            BOUNDING | HIT_EX,
            // StaticTrigger
            BOUNDING,
            // DynamicScenery
            BOUNDING | HIT_EX,
            // BreakableScenery
            BOUNDING | HIT_EX | HIT,
            // DynamicTrigger
            BOUNDING,
            // Bounding
            STATIC_SCENERY | DYNAMIC_SCENERY | BREAKABLE_SCENERY | STATIC_TRIGGER | DYNAMIC_TRIGGER,
            // Target
            HIT | HIT_EX,
            // Hit
            TARGET | BREAKABLE_SCENERY,
            // HitEx
            TARGET | BREAKABLE_SCENERY | STATIC_SCENERY | DYNAMIC_SCENERY,
        ];

        unsafe { *TABLE.get_unchecked(typ as usize) }
    }

    fn to_bp_mask(typ: u8) -> u64 {
        const TABLE: [u64; 9] = [
            0x2, // StaticScenery
            0x2, // StaticTrigger
            0x2, // DynamicScenery
            0x2, // BreakableScenery
            0x2, // DynamicTrigger
            0x3, // Bounding
            0x2, // Target
            0x2, // Hit
            0x3, // HitEx
        ];

        unsafe { *TABLE.get_unchecked(typ as usize) }
    }
}

#[vdata(ObjectVsBroadPhaseLayerFilterVTable)]
pub(crate) struct PhyObjectVsBroadPhaseLayerFilter;

impl ObjectVsBroadPhaseLayerFilter for PhyObjectVsBroadPhaseLayerFilter {
    fn should_collide(&self, layer: ObjectLayer, bp_layer: BroadPhaseLayer) -> bool {
        let typ = extract_layer_type(layer);
        let bp_mask = PhyLayerType::to_bp_mask(typ);
        bp_mask & (1 << bp_layer) != 0
    }
}

#[vdata(ObjectLayerPairFilterVTable)]
pub(crate) struct PhyObjectLayerPairFilter;

impl ObjectLayerPairFilter for PhyObjectLayerPairFilter {
    fn should_collide(&self, layer1: ObjectLayer, layer2: ObjectLayer) -> bool {
        let team1 = extract_layer_team(layer1);
        let typ1 = extract_layer_type(layer1);
        let mask1 = PhyLayerType::to_mask(typ1);

        let team2 = extract_layer_team(layer2);
        let typ2 = extract_layer_type(layer2);
        let mask2 = PhyLayerType::to_mask(typ2);

        (team1 & team2) != 0 && ((1 << typ1) & mask2) != 0 && ((1 << typ2) & mask1) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broad_phase_layer_interface() {
        let itf = PhyBroadPhaseLayerInterface {};
        assert_eq!(itf.get_broad_phase_layer(phy_layer!(StaticScenery, All)), 0);
        assert_eq!(itf.get_broad_phase_layer(phy_layer!(StaticTrigger, All)), 0);
        assert_eq!(itf.get_broad_phase_layer(phy_layer!(DynamicScenery, All)), 1);
        assert_eq!(itf.get_broad_phase_layer(phy_layer!(BreakableScenery, All)), 1);
        assert_eq!(itf.get_broad_phase_layer(phy_layer!(DynamicTrigger, All)), 1);
        assert_eq!(itf.get_broad_phase_layer(phy_layer!(Bounding, All)), 1);
        assert_eq!(itf.get_broad_phase_layer(phy_layer!(Target, All)), 1);
        assert_eq!(itf.get_broad_phase_layer(phy_layer!(Hit, All)), 1);
        assert_eq!(itf.get_broad_phase_layer(phy_layer!(HitEx, All)), 1);
    }

    #[test]
    fn test_object_vs_broad_phase_layer_filter() {
        let itf = PhyObjectVsBroadPhaseLayerFilter {};

        assert_eq!(itf.should_collide(phy_layer!(StaticScenery, All), 0), false);
        assert_eq!(itf.should_collide(phy_layer!(StaticScenery, All), 1), true);

        assert_eq!(itf.should_collide(phy_layer!(DynamicScenery, All), 0), false);
        assert_eq!(itf.should_collide(phy_layer!(DynamicScenery, All), 1), true);

        assert_eq!(itf.should_collide(phy_layer!(StaticTrigger, All), 0), false);
        assert_eq!(itf.should_collide(phy_layer!(StaticTrigger, All), 1), true);

        assert_eq!(itf.should_collide(phy_layer!(DynamicTrigger, All), 0), false);
        assert_eq!(itf.should_collide(phy_layer!(DynamicTrigger, All), 1), true);

        assert_eq!(itf.should_collide(phy_layer!(BreakableScenery, All), 0), false);
        assert_eq!(itf.should_collide(phy_layer!(BreakableScenery, All), 1), true);

        assert_eq!(itf.should_collide(phy_layer!(Bounding, All), 0), true);
        assert_eq!(itf.should_collide(phy_layer!(Bounding, All), 1), true);

        assert_eq!(itf.should_collide(phy_layer!(Target, All), 0), false);
        assert_eq!(itf.should_collide(phy_layer!(Target, All), 1), true);

        assert_eq!(itf.should_collide(phy_layer!(Hit, All), 0), false);
        assert_eq!(itf.should_collide(phy_layer!(Hit, All), 1), true);

        assert_eq!(itf.should_collide(phy_layer!(HitEx, All), 0), true);
        assert_eq!(itf.should_collide(phy_layer!(HitEx, All), 1), true);
    }

    #[rustfmt::skip]
    #[test]
    fn test_object_layer_pair_filter() {
        let itf = PhyObjectLayerPairFilter{};

        assert_eq!(itf.should_collide(phy_layer!(StaticScenery, All), phy_layer!(StaticScenery, All)), false);
        assert_eq!(itf.should_collide(phy_layer!(StaticTrigger, All), phy_layer!(StaticTrigger, All)), false);

        assert_eq!(itf.should_collide(phy_layer!(StaticScenery, All), phy_layer!(Bounding, Player)), true);
        assert_eq!(itf.should_collide(phy_layer!(StaticTrigger, All), phy_layer!(Bounding, Enemy)), true);

        assert_eq!(itf.should_collide(phy_layer!(StaticScenery, All), phy_layer!(Target, Player)), false);
        assert_eq!(itf.should_collide(phy_layer!(StaticTrigger, All), phy_layer!(Target, Enemy)), false);

        assert_eq!(itf.should_collide(phy_layer!(StaticScenery, All), phy_layer!(Hit, Player)), false);
        assert_eq!(itf.should_collide(phy_layer!(StaticTrigger, All), phy_layer!(Hit, Enemy)), false);

        assert_eq!(itf.should_collide(phy_layer!(StaticScenery, All), phy_layer!(HitEx, Player)), true);
        assert_eq!(itf.should_collide(phy_layer!(StaticTrigger, All), phy_layer!(HitEx, Enemy)), false);

        assert_eq!(itf.should_collide(phy_layer!(Target, Enemy), phy_layer!(Hit, Enemy)), true);
        assert_eq!(itf.should_collide(phy_layer!(Target, Enemy), phy_layer!(HitEx, Enemy)), true);
        assert_eq!(itf.should_collide(phy_layer!(Target, Player), phy_layer!(Hit, Enemy)), false);
        assert_eq!(itf.should_collide(phy_layer!(Target, Player), phy_layer!(HitEx, Enemy)), false);
        
        assert_eq!(itf.should_collide(phy_layer!(Bounding, Enemy), phy_layer!(Hit, Enemy)), false);
        assert_eq!(itf.should_collide(phy_layer!(Bounding, Enemy), phy_layer!(HitEx, Enemy)), false);
        assert_eq!(itf.should_collide(phy_layer!(Bounding, Enemy), phy_layer!(Target, Enemy)), false);
    }
}
