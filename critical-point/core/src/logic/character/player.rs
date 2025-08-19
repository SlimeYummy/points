use cirtical_point_csgen::CsOut;
use glam::Vec3A;
use std::rc::Rc;
use std::sync::Arc;

use super::action::LogicCharaAction;
use super::physics::{LogicCharaPhysics, StateCharaPhysics};
use crate::consts::DEFAULT_TOWARD_DIR_2D;
use crate::instance::{assemble_player, InstPlayer};
use crate::logic::action::StateActionAny;
use crate::logic::base::{impl_state, LogicAny, LogicType, StateBase, StateType};
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::parameter::ParamPlayer;
use crate::template::TmplStyle;
use crate::utils::{extend, sb, NumID, Symbol, XResult};

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StatePlayerInit {
    pub _base: StateBase,
    pub skeleton_file: Symbol,
    pub animation_files: Vec<Symbol>,
    pub view_model: Symbol,
}

extend!(StatePlayerInit, StateBase);

impl_state!(StatePlayerInit, Player, PlayerInit, "PlayerInit");

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StatePlayerUpdate {
    pub _base: StateBase,
    pub physics: StateCharaPhysics,
    pub actions: Vec<Box<dyn StateActionAny>>,
}

extend!(StatePlayerUpdate, StateBase);

impl_state!(StatePlayerUpdate, Player, PlayerUpdate, "PlayerUpdate");

#[derive(Debug)]
pub struct LogicPlayer {
    id: NumID,
    spawn_frame: u32,
    death_frame: u32,
    inst: Rc<InstPlayer>,
    chara_physics: LogicCharaPhysics,
    chara_action: LogicCharaAction,
}

impl LogicAny for LogicPlayer {
    #[inline]
    fn typ(&self) -> LogicType {
        LogicType::Player
    }

    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn spawn_frame(&self) -> u32 {
        self.spawn_frame
    }

    #[inline]
    fn death_frame(&self) -> u32 {
        self.death_frame
    }
}

impl LogicPlayer {
    pub fn new(
        ctx: &mut ContextUpdate<'_>,
        param_player: &ParamPlayer,
    ) -> XResult<(Box<LogicPlayer>, Arc<StatePlayerInit>)> {
        let tmpl_style = ctx.tmpl_db.find_as::<TmplStyle>(param_player.style)?;

        let inst_player = Rc::new(assemble_player(&mut ctx.context_assemble(), param_player)?);
        let player_id = ctx.gene.gen_player_id()?;
        let mut player = Box::new(LogicPlayer {
            id: player_id,
            spawn_frame: ctx.frame,
            death_frame: u32::MAX,
            inst: inst_player.clone(),
            chara_physics: LogicCharaPhysics::new(
                ctx,
                player_id,
                inst_player.clone(),
                Vec3A::ZERO,
                DEFAULT_TOWARD_DIR_2D,
            )?,
            chara_action: LogicCharaAction::new(ctx, player_id, inst_player.clone())?,
        });

        let animation_files = player.chara_action.preload_assets(ctx, inst_player.clone())?;
        let state_init = Arc::new(StatePlayerInit {
            _base: StateBase::new(player.id, StateType::PlayerInit, LogicType::Player),
            skeleton_file: inst_player.skeleton_files.clone(),
            animation_files,
            view_model: sb!(&tmpl_style.view_model),
        });

        player.chara_action.update(ctx, &player.chara_physics, true)?;
        player.chara_physics.update(ctx, &player.chara_action)?;
        Ok((player, state_init))
    }

    pub fn state(&mut self) -> XResult<Box<StatePlayerUpdate>> {
        Ok(Box::new(StatePlayerUpdate {
            _base: StateBase::new(self.id, StateType::PlayerUpdate, LogicType::Player),
            physics: self.chara_physics.state(),
            actions: self.chara_action.take_states()?,
        }))
    }

    pub fn restore(&mut self, ctx: &ContextRestore) -> XResult<()> {
        let state = ctx.find_as::<StatePlayerUpdate>(self.id)?;
        self.chara_action.restore(ctx, &state.actions)?;
        self.chara_physics.restore(ctx, &state.physics)?;
        Ok(())
    }

    pub fn update(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<()> {
        self.chara_action.update(ctx, &self.chara_physics, false)?;
        self.chara_physics.update(ctx, &self.chara_action)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::action::StateActionIdle;
    use crate::logic::test_utils::*;
    use crate::utils::{id, Castable};

    fn prepare_player(tenv: &mut TestEnv) -> (Box<LogicPlayer>, Arc<StatePlayerInit>) {
        let param_player = ParamPlayer {
            character: id!("Character.Instance/1"),
            style: id!("Style.Instance/1A"),
            level: 4,
            ..Default::default()
        };
        let mut ctx = tenv.context_update();
        let (logic_player, state_init) = LogicPlayer::new(&mut ctx, &param_player).unwrap();
        ctx.input.init(1).unwrap();
        (logic_player, state_init)
    }

    #[test]
    fn test_new() {
        let mut tenv = TestEnv::new().unwrap();
        let (mut logic_player, state_init) = prepare_player(&mut tenv);
        assert_eq!(logic_player.id, 100);
        assert_eq!(logic_player.inst.tmpl_character, id!("Character.Instance/1"));
        assert_eq!(logic_player.inst.tmpl_style, id!("Style.Instance/1A"));

        assert_eq!(state_init.id, 100);
        assert_eq!(state_init.skeleton_file, "girl.*");
        assert_eq!(state_init.animation_files.len(), 4);
        let excepted_files = [
            sb!("girl_stand_idle"),
            sb!("girl_stand_ready"),
            sb!("girl_run"),
            sb!("girl_attack1_1"),
        ];
        for file in excepted_files.iter() {
            assert!(state_init.animation_files.contains(file));
        }

        let state_update = logic_player.state().unwrap();
        assert_eq!(state_update.id, 100);
        assert_eq!(state_update.actions.len(), 1);
        let state_act = state_update.actions[0].as_ref().cast::<StateActionIdle>().unwrap();
        assert_eq!(state_act.tmpl_id, id!("Action.Instance.Idle/1A"));
    }

    #[test]
    fn test_logic_player_update() {
        let mut tenv = TestEnv::new().unwrap();
        let (mut logic_player, _) = prepare_player(&mut tenv);
        logic_player.update(&mut tenv.context_update()).unwrap();
        let state_update = logic_player.state().unwrap();
        assert_eq!(state_update.id, 100);
        assert_eq!(state_update.actions.len(), 1);
        let state_act = state_update.actions[0].as_ref().cast::<StateActionIdle>().unwrap();
        assert_eq!(state_act.tmpl_id, id!("Action.Instance.Idle/1A"));
    }
}
