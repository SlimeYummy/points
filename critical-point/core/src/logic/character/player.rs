use cirtical_point_csgen::CsOut;
use glam::{Quat, Vec3};
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
use crate::utils::{extend, NumID, Symbol, XResult};

#[repr(C)]
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
#[cs_attr(Ref)]
pub struct StatePlayerInit {
    pub _base: StateAnyBase,
    pub skeleton_file: Symbol,
    pub animation_files: Vec<Symbol>,
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
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
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

    #[inline]
    fn update(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<()> {
        self.update_impl(ctx)
    }

    #[inline]
    fn restore(&mut self, ctx: &ContextRestore) -> XResult<()> {
        self.restore_impl(ctx)
    }
}

impl LogicPlayer {
    pub fn new(ctx: &mut ContextUpdate<'_>, param_player: &ParamPlayer) -> XResult<Box<LogicPlayer>> {
        let tmpl_chara = ctx.tmpl_db.find_as::<TmplCharacter>(&param_player.character)?;
        let tmpl_style = ctx.tmpl_db.find_as::<TmplStyle>(&param_player.style)?;

        let inst_player = Rc::new(assemble_player(&mut ctx.context_assemble(), param_player)?);
        let player_id = ctx.gene.gen_id();
        let mut player = Box::new(LogicPlayer {
            id: player_id,
            spawn_frame: ctx.frame,
            death_frame: u32::MAX,
            inst: inst_player.clone(),
            chara_physics: LogicCharaPhysics::new(ctx, player_id, inst_player.clone(), Vec3::ZERO, Quat::IDENTITY)?,
            chara_action: LogicCharaAction::new(ctx, player_id, inst_player.clone())?,
        });

        let animation_files = player.chara_action.preload_assets(ctx, inst_player.clone())?;
        ctx.state_init(Arc::new(StatePlayerInit {
            _base: StateAnyBase::new(player.id, StateType::PlayerInit, LogicType::Player),
            skeleton_file: tmpl_chara.skeleton.clone(),
            animation_files,
            view_model: tmpl_style.view_model.clone(),
        }));

        let state_physics = player.chara_physics.init(ctx)?;
        let state_actions = player.chara_action.init(ctx, &mut player.chara_physics)?;
        ctx.state_update(Box::new(StatePlayerUpdate {
            _base: StateAnyBase::new(player.id, StateType::PlayerUpdate, LogicType::Player),
            physics: state_physics,
            actions: state_actions,
        }));
        Ok(player)
    }

    pub fn restore_impl(&mut self, ctx: &ContextRestore) -> XResult<()> {
        let state = ctx.find_as::<StatePlayerUpdate>(self.id)?;
        self.chara_action.restore(ctx, &state.actions)?;
        self.chara_physics.restore(ctx, &state.physics)?;
        Ok(())
    }

    pub fn update_impl(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<()> {
        let state_actions = self.chara_action.update(ctx, &mut self.chara_physics)?;
        let state_physics = self.chara_physics.update(ctx)?;

        ctx.state_update(Box::new(StatePlayerUpdate {
            _base: StateAnyBase::new(self.id, StateType::PlayerUpdate, LogicType::Player),
            physics: state_physics,
            actions: state_actions,
        }));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::action::StateActionIdle;
    use crate::logic::game::{ContextUpdate, LogicSystems};
    use crate::logic::system::state::StateSet;
    use crate::logic::test_utils::*;
    use crate::utils::{s, CastPtr, CastRef};

    fn prepare(systems: &mut LogicSystems, clear_state: bool) -> (Box<LogicPlayer>, ContextUpdate<'_>) {
        let mut ctx = ContextUpdate::new_empty(systems);
        let param_player = ParamPlayer {
            character: s!("Character.No1"),
            style: s!("Style.No1-1"),
            level: 4,
            ..Default::default()
        };
        let logic_player = LogicPlayer::new(&mut ctx, &param_player).unwrap();
        if clear_state {
            ctx.state_set = StateSet::new(1, 0, 0);
        }
        ctx.input.init(&[logic_player.id]).unwrap();
        (logic_player, ctx)
    }

    #[test]
    fn test_logic_player_new() {
        let mut systems = mock_logic_systems();
        let (logic_player, ctx) = prepare(&mut systems, false);
        assert_eq!(logic_player.id, 1);
        assert_eq!(logic_player.inst.tmpl_character, s!("Character.No1"));
        assert_eq!(logic_player.inst.tmpl_style, s!("Style.No1-1"));

        assert_eq!(ctx.state_set.inits.len(), 1);
        let state_init = &ctx.state_set.inits[0];
        let state_init = state_init.cast_to::<StatePlayerInit>().unwrap();
        assert_eq!(state_init.id, 1);
        assert_eq!(state_init.skeleton_file, s!("girl_skeleton_logic.ozz"));
        assert_eq!(state_init.animation_files.len(), 2);
        let excepted_files = [
            s!("girl_animation_logic_stand_idle.ozz"),
            s!("girl_animation_logic_stand_ready.ozz"),
        ];
        for file in excepted_files.iter() {
            assert!(state_init.animation_files.contains(file));
        }

        assert_eq!(ctx.state_set.updates.len(), 1);
        let state_update = &ctx.state_set.updates[0];
        let state_update = state_update.cast_ref::<StatePlayerUpdate>().unwrap();
        assert_eq!(state_update.id, 1);
        assert_eq!(state_update.actions.len(), 1);
        let state_act = state_update.actions[0].cast_ref::<StateActionIdle>().unwrap();
        assert_eq!(state_act.tmpl_id, s!("Action.No1.Idle"));
    }

    #[test]
    fn test_logic_player_update() {
        let mut systems = mock_logic_systems();
        let (mut logic_player, mut ctx) = prepare(&mut systems, true);
        logic_player.update(&mut ctx).unwrap();

        assert_eq!(ctx.state_set.inits.len(), 0);
        assert_eq!(ctx.state_set.updates.len(), 1);
        let state_update = &ctx.state_set.updates[0];
        let state_update = state_update.cast_ref::<StatePlayerUpdate>().unwrap();
        assert_eq!(state_update.id, 1);
        assert_eq!(state_update.actions.len(), 1);
        let state_act = state_update.actions[0].cast_ref::<StateActionIdle>().unwrap();
        assert_eq!(state_act.tmpl_id, s!("Action.No1.Idle"));
    }
}
