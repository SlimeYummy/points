use critical_point_csgen::CsOut;
use glam::Vec3A;
use glam_ext::Vec2xz;
use std::rc::Rc;
use std::sync::Arc;

use crate::animation::AnimationFileMeta;
use crate::consts::DEFAULT_TOWARD_DIR_2D;
use crate::instance::InstCharacter;
use crate::logic::action::StateActionAny;
use crate::logic::base::{impl_state, LogicAny, LogicType, StateBase, StateType};
use crate::logic::character::{LogicCharaAction, LogicCharaPhysics, StateCharaPhysics};
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::logic::StateCharaAction;
use crate::parameter::{ParamNpc, ParamPlayer};
use crate::template::{TmplNpcCharacter, TmplStyle};
use crate::utils::{extend, CustomEvent, NumID, Symbol, XResult};

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateCharacterInit {
    pub _base: StateBase,
    pub skeleton_file: Symbol,
    pub animation_metas: Vec<AnimationFileMeta>,
    pub view_model: String,
    pub init_position: Vec3A,
    pub init_direction: Vec2xz,
}

extend!(StateCharacterInit, StateBase);

impl_state!(StateCharacterInit, Character, CharacterInit, "CharacterInit");

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateCharacterUpdate {
    pub _base: StateBase,
    pub physics: StateCharaPhysics,
    pub action: StateCharaAction,
    pub actions: Vec<Box<dyn StateActionAny>>,
    pub custom_events: Vec<CustomEvent>,
}

extend!(StateCharacterUpdate, StateBase);

impl_state!(StateCharacterUpdate, Character, CharacterUpdate, "CharacterUpdate");

#[derive(Debug)]
pub struct LogicCharacter {
    id: NumID,
    spawn_frame: u32,
    death_frame: u32,
    inst: Rc<InstCharacter>,
    chara_physics: LogicCharaPhysics,
    chara_action: LogicCharaAction,
}

impl LogicAny for LogicCharacter {
    #[inline]
    fn typ(&self) -> LogicType {
        LogicType::Character
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

impl LogicCharacter {
    pub fn new_player(
        ctx: &mut ContextUpdate,
        param_player: &ParamPlayer,
    ) -> XResult<(Box<LogicCharacter>, Arc<StateCharacterInit>)> {
        let inst_player = Rc::new(InstCharacter::new_player(&mut ctx.context_assemble(), param_player)?);
        let tmpl_style = ctx.tmpl_db.find_as::<TmplStyle>(param_player.style)?;
        Self::new_impl(
            ctx,
            inst_player,
            &tmpl_style.view_model,
            param_player.position,
            DEFAULT_TOWARD_DIR_2D,
        )
    }

    pub fn new_npc(
        ctx: &mut ContextUpdate,
        param: &ParamNpc,
    ) -> XResult<(Box<LogicCharacter>, Arc<StateCharacterInit>)> {
        let inst_npc = Rc::new(InstCharacter::new_npc(&mut ctx.context_assemble(), param)?);
        let tmpl_chara = ctx.tmpl_db.find_as::<TmplNpcCharacter>(param.character)?;
        Self::new_impl(
            ctx,
            inst_npc,
            &tmpl_chara.view_model,
            param.position,
            DEFAULT_TOWARD_DIR_2D,
        )
    }

    fn new_impl(
        ctx: &mut ContextUpdate,
        inst_chara: Rc<InstCharacter>,
        view_model: &str,
        init_position: Vec3A,
        init_direction: Vec2xz,
    ) -> XResult<(Box<LogicCharacter>, Arc<StateCharacterInit>)> {
        let player_id = ctx.gene.gen_player_id()?;
        let mut player = Box::new(LogicCharacter {
            id: player_id,
            spawn_frame: ctx.frame,
            death_frame: u32::MAX,
            inst: inst_chara.clone(),
            chara_physics: LogicCharaPhysics::new(ctx, player_id, inst_chara.clone(), init_position, init_direction)?,
            chara_action: LogicCharaAction::new(ctx, player_id, inst_chara.clone())?,
        });

        let animation_metas = player.chara_action.preload_assets(ctx)?;
        let state_init = Arc::new(StateCharacterInit {
            _base: StateBase::new(player.id, StateType::CharacterInit, LogicType::Character),
            skeleton_file: inst_chara.skeleton_files.clone(),
            animation_metas,
            view_model: view_model.to_string(),
            init_position,
            init_direction,
        });

        player
            .chara_action
            .update(ctx, &player.chara_physics, inst_chara.clone(), true)?;
        player.chara_physics.update(ctx, &player.chara_action)?;
        Ok((player, state_init))
    }

    pub fn state(&mut self) -> XResult<Box<StateCharacterUpdate>> {
        let (action, actions, custom_events) = self.chara_action.take_states()?;
        Ok(Box::new(StateCharacterUpdate {
            _base: StateBase::new(self.id, StateType::CharacterUpdate, LogicType::Character),
            physics: self.chara_physics.state(),
            action,
            actions,
            custom_events,
        }))
    }

    pub fn restore(&mut self, ctx: &ContextRestore) -> XResult<()> {
        let state = ctx.find_as::<StateCharacterUpdate>(self.id)?;
        self.chara_action.restore(ctx, &state.action, &state.actions)?;
        self.chara_physics.restore(ctx, &state.physics)?;
        Ok(())
    }

    pub fn update(&mut self, ctx: &mut ContextUpdate) -> XResult<()> {
        self.chara_action
            .update(ctx, &self.chara_physics, self.inst.clone(), false)?;
        self.chara_physics.update(ctx, &self.chara_action)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::action::StateActionIdle;
    use crate::logic::test_utils::*;
    use crate::utils::{id, sb, Castable};

    fn prepare_player(tenv: &mut TestEnv) -> (Box<LogicCharacter>, Arc<StateCharacterInit>) {
        let param_player = ParamPlayer {
            character: id!("Character.Instance^1"),
            style: id!("Style.Instance^1A"),
            level: 4,
            ..Default::default()
        };
        let mut ctx = tenv.context_update();
        let (logic_player, state_init) = LogicCharacter::new_player(&mut ctx, &param_player).unwrap();
        ctx.input.init(1).unwrap();
        (logic_player, state_init)
    }

    #[test]
    fn test_new() {
        let mut tenv = TestEnv::new().unwrap();
        let (mut logic_player, state_init) = prepare_player(&mut tenv);
        assert_eq!(logic_player.id, 100);
        assert_eq!(logic_player.inst.tmpl_character, id!("Character.Instance^1"));
        assert_eq!(logic_player.inst.tmpl_style, id!("Style.Instance^1A"));

        assert_eq!(state_init.id, 100);
        assert_eq!(state_init.skeleton_file, "Girl.*");
        assert_eq!(state_init.animation_metas.len(), 4);
        let excepted_files = [
            sb!("Girl_Idle_Empty"),
            sb!("Girl_Idle_Axe"),
            sb!("Girl_Run_Empty"),
            sb!("Girl_Attack_01A"),
        ];
        for file in excepted_files.iter() {
            assert!(state_init.animation_metas.iter().find(|f| f.files == *file).is_some());
        }

        let state_update = logic_player.state().unwrap();
        assert_eq!(state_update.id, 100);
        assert_eq!(state_update.actions.len(), 1);
        let state_act = state_update.actions[0].as_ref().cast::<StateActionIdle>().unwrap();
        assert_eq!(state_act.tmpl_id, id!("Action.Instance.Idle^1A"));
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
        assert_eq!(state_act.tmpl_id, id!("Action.Instance.Idle^1A"));
    }
}
