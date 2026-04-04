use glam::Vec3A;
use std::rc::Rc;

use crate::consts::{DEFAULT_TOWARD_DIR_2D, TEST_ASSET_PATH};
use crate::instance::{InstActionEmpty, InstCharacter};
use crate::logic::action::base::{ActionStartArgs, ContextAction, StateActionAny};
use crate::logic::character::LogicCharaPhysics;
use crate::logic::game::{ContextUpdate, LogicSystems};
use crate::logic::{InputVariables, LogicActionEmpty};
use crate::parameter::ParamPlayer;
use crate::template::TmplDatabase;
use crate::utils::{id, ifelse, ActionType, NumID, VirtualKey, XResult};

pub(super) fn test_state_action_rkyv(
    state: Box<dyn StateActionAny>,
    typ: ActionType,
) -> XResult<Box<dyn StateActionAny>> {
    use rkyv::rancor::Error;
    use rkyv::Archived;

    let buffer = rkyv::to_bytes::<Error>(&state).unwrap();
    let archived = unsafe { rkyv::access_unchecked::<Archived<Box<dyn StateActionAny>>>(&buffer) };
    assert_eq!(archived.typ(), typ);
    let result: Box<dyn StateActionAny> = rkyv::deserialize::<_, Error>(archived).unwrap();
    assert_eq!(result.typ(), typ);
    Ok(result)
}

pub(super) struct TestEnv {
    pub systems: LogicSystems,
    pub inst_player: Rc<InstCharacter>,
    pub chara_physics: LogicCharaPhysics,
    pub inst_empty: Rc<InstActionEmpty>,
    pub logic_empty: LogicActionEmpty,
}

impl TestEnv {
    pub const FRAME: u32 = 100;
    pub const PLAYER_ID: NumID = NumID(NumID::MIN_PLAYER.0 + 1);

    pub fn new() -> XResult<TestEnv> {
        let db = TmplDatabase::new(10240, 150)?;
        let mut systems = LogicSystems::new(db, TEST_ASSET_PATH, None)?;

        let mut ctx = ContextUpdate::new(&mut systems, Self::FRAME, 95);
        let param_player = ParamPlayer {
            character: id!("Character.Instance^1"),
            style: id!("Style.Instance^1A"),
            level: 1,
            ..Default::default()
        };
        let inst_player = Rc::new(InstCharacter::new_player(&mut ctx.context_assemble(), &param_player)?);

        let chara_physics = LogicCharaPhysics::new(
            &mut ctx,
            Self::PLAYER_ID,
            inst_player.clone(),
            Vec3A::ZERO,
            DEFAULT_TOWARD_DIR_2D,
        )?;

        let inst_empty = Rc::new(InstActionEmpty::new());
        let logic_empty = LogicActionEmpty::new(&mut ctx, inst_empty.clone());

        Ok(TestEnv {
            systems,
            inst_player,
            chara_physics,
            inst_empty,
            logic_empty,
        })
    }

    pub fn context_update(&mut self) -> ContextUpdate<'_> {
        ContextUpdate::new(&mut self.systems, Self::FRAME, 95)
    }

    pub fn contexts(
        &mut self,
        key: VirtualKey,
        prev_action: bool,
    ) -> (ContextUpdate<'_>, ContextAction<'_>, ActionStartArgs<'_>) {
        let ctx = ContextUpdate::new(&mut self.systems, 100, 95);
        let ctxa = ContextAction::new_normalized(Self::PLAYER_ID, &self.chara_physics, InputVariables::default(), 1.0);
        let sargs = ActionStartArgs::new(ifelse!(prev_action, Some(&self.logic_empty), None), key, None);
        (ctx, ctxa, sargs)
    }
}
