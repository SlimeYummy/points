use glam::Vec3A;
use glam_ext::Vec2xz;
use std::rc::Rc;

use crate::consts::{DEFAULT_TOWARD_DIR_2D, TEST_ASSET_PATH};
use crate::input::InputPlayerInputs;
use crate::instance::{InstActionEmpty, InstCharacter};
use crate::logic::LogicActionEmpty;
use crate::logic::action::base::{ActionStartArgs, ContextAction, StateActionAny};
use crate::logic::character::LogicCharaPhysics;
use crate::logic::game::{ContextUpdate, ContextUpdateEx, GameTime, LogicSystems};
use crate::logic::zone::LogicZone;
use crate::parameter::{ParamPlayer, ParamZone};
use crate::template::TmplDatabase;
use crate::utils::{ActionType, NumID, VirtualKey, XResult, id, ifelse};

pub(super) fn test_state_action_rkyv(
    state: Box<dyn StateActionAny>,
    typ: ActionType,
) -> XResult<Box<dyn StateActionAny>> {
    use rkyv::Archived;
    use rkyv::rancor::Error;

    let buffer = rkyv::to_bytes::<Error>(&state).unwrap();
    let archived = unsafe { rkyv::access_unchecked::<Archived<Box<dyn StateActionAny>>>(&buffer) };
    assert_eq!(archived.typ(), typ);
    let result: Box<dyn StateActionAny> = rkyv::deserialize::<_, Error>(archived).unwrap();
    assert_eq!(result.typ(), typ);
    Ok(result)
}

pub(super) struct TestEnv {
    pub systems: LogicSystems,
    pub zone: Box<LogicZone>,
    pub time: GameTime,
    pub inst_chara: Rc<InstCharacter>,
    pub chara_phy: LogicCharaPhysics,
    pub inst_empty: Rc<InstActionEmpty>,
    pub logic_empty: LogicActionEmpty,
}

impl TestEnv {
    pub const FRAME: u32 = 100;
    pub const PLAYER_ID: NumID = NumID::MIN_PLAYER;

    pub fn new() -> XResult<TestEnv> {
        let db = TmplDatabase::new(10240, 150)?;
        let mut systems = LogicSystems::new(db, TEST_ASSET_PATH, None)?;
        systems.input.init(1)?;

        let time_init = GameTime::new(Self::FRAME, 0);
        let (zone, _) = LogicZone::new(&mut ContextUpdate::new(&mut systems, &time_init), &ParamZone {
            zone: id!("Zone.Demo"),
        })?;

        let time = GameTime::new(Self::FRAME, 95);
        let mut ctx = ContextUpdateEx::new(&mut systems, &time, &zone);
        let param_player = ParamPlayer {
            character: id!("Character.Instance^1"),
            style: id!("Style.Instance^1A"),
            level: 1,
            ..Default::default()
        };
        let inst_chara = InstCharacter::new_player(&mut ctx.context_assemble(), &param_player)?;

        let chara_phy = LogicCharaPhysics::new(
            &mut ctx,
            Self::PLAYER_ID,
            inst_chara.clone(),
            Vec3A::ZERO,
            DEFAULT_TOWARD_DIR_2D,
        )?;

        let inst_empty = Rc::new(InstActionEmpty::new());
        let logic_empty = LogicActionEmpty::new(&mut ctx, inst_empty.clone());

        ctx.systems
            .input
            .produce(&[InputPlayerInputs::new(NumID::MIN_PLAYER, 1, vec![])])
            .unwrap();

        Ok(TestEnv {
            systems,
            zone,
            time,
            inst_chara,
            chara_phy,
            inst_empty,
            logic_empty,
        })
    }

    pub fn context_update(&mut self) -> ContextUpdateEx<'_> {
        ContextUpdateEx::new(&mut self.systems, &self.time, &self.zone)
    }

    pub fn contexts(&mut self, prev_action: bool) -> (ContextUpdateEx<'_>, ContextAction<'_>, ActionStartArgs<'_>) {
        let ctx = ContextUpdateEx::new(&mut self.systems, &self.time, &self.zone);
        let mut ctxa = ContextAction::new(Self::PLAYER_ID, self.inst_chara.clone(), &self.chara_phy, None);
        ctxa.set_time_normalized(1.0);
        let sargs = ActionStartArgs::new(
            ifelse!(prev_action, Some(&self.logic_empty), None),
            VirtualKey::Idle,
            Vec2xz::ZERO,
        );
        (ctx, ctxa, sargs)
    }
}
