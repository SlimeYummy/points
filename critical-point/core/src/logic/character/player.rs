use cirtical_point_csgen::CsOut;
use glam::{Quat, Vec3A};
use std::rc::Rc;
use std::sync::Arc;

use super::action::LogicCharaAction;
use super::physics::{LogicCharaPhysics, StateCharaPhysics};
use crate::instance::{assemble_player, InstPlayer};
use crate::logic::action::StateAction;
use crate::logic::base::{ArchivedStateAny, LogicAny, LogicType, StateAny, StateAnyBase, StateType};
use crate::logic::game::{ContextRestore, ContextUpdate};
use crate::parameter::ParamPlayer;
use crate::template::{TmplCharacter, TmplStyle};
use crate::utils::{extend, ASymbol, NumID, XResult};

#[repr(C)]
#[derive(Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
#[cs_attr(Ref)]
pub struct StatePlayerInit {
    pub _base: StateAnyBase,
    pub skeleton_file: ASymbol,
    pub animation_files: Vec<ASymbol>,
    pub view_model: String,
}

extend!(StatePlayerInit, StateAnyBase);

unsafe impl StateAny for StatePlayerInit {
    #[inline]
    fn typ(&self) -> StateType {
        assert_eq!(self.typ, StateType::PlayerInit);
        StateType::PlayerInit
    }

    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        assert_eq!(self.logic_typ, LogicType::Player);
        LogicType::Player
    }
}

impl ArchivedStateAny for rkyv::Archived<StatePlayerInit> {
    #[inline]
    fn typ(&self) -> StateType {
        StateType::PlayerInit
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        LogicType::Player
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
#[cs_attr(Ref)]
pub struct StatePlayerUpdate {
    pub _base: StateAnyBase,
    pub physics: StateCharaPhysics,
    pub actions: Vec<Box<dyn StateAction>>,
}

extend!(StatePlayerUpdate, StateAnyBase);

unsafe impl StateAny for StatePlayerUpdate {
    #[inline]
    fn typ(&self) -> StateType {
        assert!(self.typ == StateType::PlayerUpdate);
        StateType::PlayerUpdate
    }

    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        assert!(self.logic_typ == LogicType::Player);
        LogicType::Player
    }
}

impl ArchivedStateAny for rkyv::Archived<StatePlayerUpdate> {
    #[inline]
    fn typ(&self) -> StateType {
        StateType::PlayerUpdate
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        LogicType::Player
    }
}

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
        let tmpl_chara = ctx.tmpl_db.find_as::<TmplCharacter>(&param_player.character)?;
        let tmpl_style = ctx.tmpl_db.find_as::<TmplStyle>(&param_player.style)?;

        let inst_player = Rc::new(assemble_player(&mut ctx.context_assemble(), param_player)?);
        let player_id = ctx.gene.gen_player_id()?;
        let mut player = Box::new(LogicPlayer {
            id: player_id,
            spawn_frame: ctx.frame,
            death_frame: u32::MAX,
            inst: inst_player.clone(),
            chara_physics: LogicCharaPhysics::new(ctx, player_id, inst_player.clone(), Vec3A::ZERO, Quat::IDENTITY)?,
            chara_action: LogicCharaAction::new(ctx, player_id, inst_player.clone())?,
        });

        let animation_files = player.chara_action.preload_assets(ctx, inst_player.clone())?;
        let state_init = Arc::new(StatePlayerInit {
            _base: StateAnyBase::new(player.id, StateType::PlayerInit, LogicType::Player),
            skeleton_file: ASymbol::from(&tmpl_chara.skeleton),
            animation_files,
            view_model: tmpl_style.view_model.clone(),
        });

        player.chara_action.update(ctx, &player.chara_physics, true)?;
        player.chara_physics.update(ctx, &player.chara_action)?;
        Ok((player, state_init))
    }

    pub fn state(&mut self) -> XResult<Box<StatePlayerUpdate>> {
        Ok(Box::new(StatePlayerUpdate {
            _base: StateAnyBase::new(self.id, StateType::PlayerUpdate, LogicType::Player),
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
    use crate::logic::game::{ContextUpdate, LogicSystems};
    use crate::logic::test_utils::*;
    use crate::utils::{asb, sb, CastRef};

    fn prepare(systems: &mut LogicSystems, frame: u32) -> (Box<LogicPlayer>, Arc<StatePlayerInit>, ContextUpdate<'_>) {
        let mut ctx = ContextUpdate::new(systems, frame, 0);
        let param_player = ParamPlayer {
            character: sb!("Character.No1"),
            style: sb!("Style.No1-1"),
            level: 4,
            ..Default::default()
        };
        let (logic_player, state_init) = LogicPlayer::new(&mut ctx, &param_player).unwrap();
        ctx.input.init(1).unwrap();
        (logic_player, state_init, ctx)
    }

    #[test]
    fn test_logic_player_new() {
        let mut systems = mock_logic_systems();
        let (mut logic_player, state_init, _) = prepare(&mut systems, 0);
        assert_eq!(logic_player.id, 100);
        assert_eq!(logic_player.inst.tmpl_character, sb!("Character.No1"));
        assert_eq!(logic_player.inst.tmpl_style, sb!("Style.No1-1"));

        assert_eq!(state_init.id, 100);
        assert_eq!(state_init.skeleton_file, sb!("skel.ozz"));
        assert_eq!(state_init.animation_files.len(), 2);
        let excepted_files = [asb!("anim_stand_idle.ozz"), asb!("anim_stand_ready.ozz")];
        for file in excepted_files.iter() {
            assert!(state_init.animation_files.contains(file));
        }

        let state_update = logic_player.state().unwrap();
        assert_eq!(state_update.id, 100);
        assert_eq!(state_update.actions.len(), 1);
        let state_act = state_update.actions[0].cast_ref::<StateActionIdle>().unwrap();
        assert_eq!(state_act.tmpl_id, sb!("Action.No1.Idle"));
    }

    #[test]
    fn test_logic_player_update() {
        let mut systems = mock_logic_systems();
        let (mut logic_player, _, mut ctx) = prepare(&mut systems, 1);
        logic_player.update(&mut ctx).unwrap();
        let state_update = logic_player.state().unwrap();
        assert_eq!(state_update.id, 100);
        assert_eq!(state_update.actions.len(), 1);
        let state_act = state_update.actions[0].cast_ref::<StateActionIdle>().unwrap();
        assert_eq!(state_act.tmpl_id, sb!("Action.No1.Idle"));
    }
}
