use jolt_physics_rs::PhysicsSystem;

use crate::consts::TEST_ASSET_PATH;
use crate::logic::game::{ContextUpdate, ContextUpdateEx, GameTime, LogicSystems};
use crate::logic::physics::{PhyBroadPhaseLayerInterface, PhyObjectLayerPairFilter, PhyObjectVsBroadPhaseLayerFilter};
use crate::logic::zone::LogicZone;
use crate::parameter::ParamZone;
use crate::template::TmplDatabase;
use crate::utils::{XResult, id};

pub(super) struct TestEnv {
    pub systems: LogicSystems,
    pub zone: Box<LogicZone>,
    pub time: GameTime,
}

impl TestEnv {
    pub const FRAME: u32 = 100;

    pub fn new() -> XResult<TestEnv> {
        let db = TmplDatabase::new(10240, 150)?;
        let mut systems = LogicSystems::new(db, TEST_ASSET_PATH, None)?;
        let time = GameTime::new(Self::FRAME, 0);
        let mut ctx = ContextUpdate::new(&mut systems, &time);
        let (zone, _) = LogicZone::new(&mut ctx, &ParamZone { zone: id!("Zone.Demo") })?;
        Ok(TestEnv {
            systems,
            zone,
            time: GameTime::new(Self::FRAME, 95),
        })
    }

    pub fn context_update(&mut self) -> ContextUpdate<'_> {
        ContextUpdate::new(&mut self.systems, &self.time)
    }

    pub fn context_update_ex(&mut self) -> ContextUpdateEx<'_> {
        ContextUpdateEx::new(&mut self.systems, &self.time, &self.zone)
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
