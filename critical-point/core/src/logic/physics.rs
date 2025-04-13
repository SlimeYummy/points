use jolt_physics_rs::{
    vdata, BroadPhaseLayer, BroadPhaseLayerInterface, BroadPhaseLayerInterfaceVTable, ObjectLayer,
    ObjectLayerPairFilter, ObjectLayerPairFilterVTable, ObjectVsBroadPhaseLayerFilter,
    ObjectVsBroadPhaseLayerFilterVTable,
};

pub const PHY_LAYER_STATIC: ObjectLayer = 1;
pub const PHY_LAYER_DYNAMIC: ObjectLayer = 2;
pub const PHY_LAYER_PLAYER: ObjectLayer = 3;

pub const PHY_PB_LAYER_COUNT: u32 = 2;
pub const PHY_PB_LAYER_STATIC: BroadPhaseLayer = 0;
pub const PHY_PB_LAYER_DYNAMIC: BroadPhaseLayer = 1;

#[vdata(BroadPhaseLayerInterfaceVTable)]
pub(crate) struct BroadPhaseLayerInterfaceImpl;

impl BroadPhaseLayerInterface for BroadPhaseLayerInterfaceImpl {
    fn get_num_broad_phase_layers(&self) -> u32 {
        PHY_PB_LAYER_COUNT
    }

    fn get_broad_phase_layer(&self, layer: ObjectLayer) -> BroadPhaseLayer {
        match layer {
            PHY_LAYER_STATIC => PHY_PB_LAYER_STATIC,
            PHY_LAYER_DYNAMIC => PHY_PB_LAYER_DYNAMIC,
            PHY_LAYER_PLAYER => PHY_PB_LAYER_DYNAMIC,
            _ => PHY_PB_LAYER_STATIC,
        }
    }
}

#[vdata(ObjectVsBroadPhaseLayerFilterVTable)]
pub(crate) struct ObjectVsBroadPhaseLayerFilterImpl;

impl ObjectVsBroadPhaseLayerFilter for ObjectVsBroadPhaseLayerFilterImpl {
    fn should_collide(&self, layer: ObjectLayer, bp_layer: BroadPhaseLayer) -> bool {
        match layer {
            PHY_LAYER_STATIC => bp_layer != PHY_PB_LAYER_STATIC,
            PHY_LAYER_DYNAMIC => true,
            PHY_LAYER_PLAYER => true,
            _ => false,
        }
    }
}

#[vdata(ObjectLayerPairFilterVTable)]
pub(crate) struct ObjectLayerPairFilterImpl;

impl ObjectLayerPairFilter for ObjectLayerPairFilterImpl {
    fn should_collide(&self, layer1: ObjectLayer, layer2: ObjectLayer) -> bool {
        match layer1 {
            PHY_LAYER_STATIC => layer2 != PHY_LAYER_STATIC,
            PHY_LAYER_DYNAMIC => true,
            PHY_LAYER_PLAYER => layer2 != PHY_LAYER_PLAYER,
            _ => false,
        }
    }
}
