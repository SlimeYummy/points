use jolt_physics_rs::PhysicsSystem;

use crate::consts::TEST_ASSET_PATH;
use crate::instance::InstCharacter;
use crate::logic::game::{ContextUpdate, LogicSystems};
use crate::logic::physics::{PhyBroadPhaseLayerInterface, PhyObjectLayerPairFilter, PhyObjectVsBroadPhaseLayerFilter};
use crate::parameter::ParamPlayer;
use crate::template::TmplDatabase;
use crate::utils::{id, NumID, XResult};

pub(super) struct TestEnv {
    pub systems: LogicSystems,
}

impl TestEnv {
    pub const FRAME: u32 = 100;

    pub fn new() -> XResult<TestEnv> {
        let db = TmplDatabase::new(10240, 150)?;
        let systems = LogicSystems::new(db, TEST_ASSET_PATH, None)?;
        Ok(TestEnv { systems })
    }

    pub fn context_update(&mut self) -> ContextUpdate<'_> {
        ContextUpdate::new(&mut self.systems, Self::FRAME, 95)
    }
}

pub(crate) fn mock_physics_system() -> PhysicsSystem {
    PhysicsSystem::new(
        PhyBroadPhaseLayerInterface::new_vbox(PhyBroadPhaseLayerInterface),
        PhyObjectVsBroadPhaseLayerFilter::new_vbox(PhyObjectVsBroadPhaseLayerFilter),
        PhyObjectLayerPairFilter::new_vbox(PhyObjectLayerPairFilter),
    )
}

// pub(crate) fn mock_logic_systems() -> LogicSystems {
//     let db = TmplDatabase::new(10240, 150).unwrap();
//     LogicSystems::new(db, TEST_ASSET_PATH, None).unwrap()
// }

// pub(crate) fn mock_logic_chara_physics(player_id: NumID, inst_player: Rc<InstCharacter>) {}

// pub(crate) fn mock_inst_player(systems: &mut LogicSystems) -> Rc<InstCharacter> {
//     let mut ctx = ContextUpdate::new(systems, 0, 0);
//     let param_player = ParamPlayer {
//         character: id!("Character.One"),
//         style: id!("Style.One/1"),
//         level: 4,
//         ..Default::default()
//     };
//     let inst_player = assemble_player(&mut ctx.context_assemble(), &param_player).unwrap();
//     Rc::new(inst_player)
// }
