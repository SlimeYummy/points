use jolt_physics_rs::PhysicsSystem;
use std::fmt::Debug;
use std::rc::Rc;

use crate::consts::TEST_ASSET_PATH;
use crate::instance::{assemble_player, InstPlayer};
use crate::logic::action::{
    impl_state_action, ContextAction, LogicActionAny, LogicActionBase, LogicActionStatus, StateActionAny,
    StateActionBase, StateActionType,
};
use crate::logic::game::{ContextUpdate, LogicSystems};
use crate::logic::physics::{
    BroadPhaseLayerInterfaceImpl, ObjectLayerPairFilterImpl, ObjectVsBroadPhaseLayerFilterImpl,
};
use crate::parameter::ParamPlayer;
use crate::template::{TmplDatabase, TmplType};
use crate::utils::{extend, id, NumID, TmplID, XResult, MIN_PLAYER_ID};

pub(super) struct TestEnv {
    pub systems: LogicSystems,
}

impl TestEnv {
    pub const FRAME: u32 = 100;
    pub const PLAYER_ID: NumID = MIN_PLAYER_ID + 1;

    pub fn new() -> XResult<TestEnv> {
        let db = TmplDatabase::new(10240, 150)?;
        let systems = LogicSystems::new(db, TEST_ASSET_PATH, None)?;
        Ok(TestEnv { systems })
    }

    pub fn context_update(&mut self) -> ContextUpdate<'_> {
        ContextUpdate::new(&mut self.systems, Self::FRAME, 95)
    }
}

// pub(crate) fn mock_physics_system() -> PhysicsSystem {
//     PhysicsSystem::new(
//         BroadPhaseLayerInterfaceImpl::new_vbox(BroadPhaseLayerInterfaceImpl),
//         ObjectVsBroadPhaseLayerFilterImpl::new_vbox(ObjectVsBroadPhaseLayerFilterImpl),
//         ObjectLayerPairFilterImpl::new_vbox(ObjectLayerPairFilterImpl),
//     )
// }

// pub(crate) fn mock_logic_systems() -> LogicSystems {
//     let db = TmplDatabase::new(10240, 150).unwrap();
//     LogicSystems::new(db, TEST_ASSET_PATH, None).unwrap()
// }

// pub(crate) fn mock_logic_chara_physics(player_id: NumID, inst_player: Rc<InstPlayer>) {}

// pub(crate) fn mock_inst_player(systems: &mut LogicSystems) -> Rc<InstPlayer> {
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
