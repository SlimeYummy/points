use cirtical_point_csgen::CsOut;
use jolt_physics_rs::{BodyInterface, PhysicsSystem};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::asset::AssetLoader;
use crate::consts::MAX_INPUT_WINDOW;
use crate::instance::ContextAssemble;
use crate::logic::base::{ArchivedStateAny, LogicAny, LogicType, StateAny, StateAnyBase, StateType};
use crate::logic::character::{LogicNpc, LogicPlayer};
use crate::logic::physics::{
    BroadPhaseLayerInterfaceImpl, ObjectLayerPairFilterImpl, ObjectVsBroadPhaseLayerFilterImpl,
};
use crate::logic::stage::LogicStage;
use crate::logic::system::generation::SystemGeneration;
use crate::logic::system::input::{InputFrameEvents, InputPlayerEvents, SystemInput};
use crate::logic::system::save::SystemSave;
use crate::logic::system::state::{StateSet, SystemState};
use crate::parameter::{ParamPlayer, ParamStage};
use crate::script::ScriptExecutor;
use crate::template::TmplDatabase;
use crate::utils::{bubble_sort_by, extend, xres, CastRef, HistoryVec, NumID, XResult, GAME_ID};

//
// LogicLoop
//

pub struct LogicLoop {
    systems: LogicSystems,
    game: Box<LogicGame>,
    frame: u32, // The current game frame for library user's side
}

impl LogicLoop {
    pub fn new<P: AsRef<Path>>(
        tmpl_db: TmplDatabase,
        asset_path: P,
        param_stage: ParamStage,
        param_players: Vec<ParamPlayer>,
        save_path: Option<PathBuf>,
    ) -> XResult<(LogicLoop, Arc<StateSet>)> {
        let mut systems = LogicSystems::new(tmpl_db, asset_path, save_path)?;
        systems.input.init(param_players.len())?;

        let mut ctx = ContextUpdate {
            systems: &mut systems,
            frame: 0,
            synced_frame: 0,
        };
        let (game, state_set) = LogicGame::new(&mut ctx, param_stage, param_players)?;
        systems.state.init(state_set.clone())?;

        let logic_loop = LogicLoop {
            systems,
            game,
            frame: 0,
        };
        Ok((logic_loop, state_set))
    }

    pub fn update(&mut self, mut player_events: Vec<InputPlayerEvents>) -> XResult<Vec<Arc<StateSet>>> {
        if self.systems.stopped {
            return xres!(Unexpected; "system stopped");
        }

        let systems = &mut self.systems;
        let game = &mut self.game;
        self.frame += 1;

        if let Some(save) = systems.save.as_mut() {
            let player_events = InputFrameEvents::new(self.frame, &player_events);
            save.save_input(player_events)?;
        }

        // Insert new input events
        bubble_sort_by(&mut player_events, |a, b| {
            a.player_id < b.player_id && a.frame < b.frame
        });
        let base_frame = systems.input.produce(&player_events)?.min(game.frame);

        // Restore to base_frame
        if base_frame < game.frame {
            systems.state.restore(base_frame)?;
            systems.gene.restore(base_frame);
            // TODO: restore physics.

            let ctx = ContextRestore::new(systems.state[base_frame].clone());
            game.restore(&ctx)?;
            assert_eq!(game.frame, base_frame);
        }

        // Update frame to current
        while game.frame < self.frame {
            let frame = game.frame + 1;
            let synced_frame = systems.input.synced_frame();
            let mut ctx = ContextUpdate {
                systems,
                frame,
                synced_frame,
            };
            let state_set = game.update(&mut ctx)?;
            systems.state.append(state_set.clone())?;

            systems.gene.update(frame);
        }
        let ret_states = systems.state.range(base_frame + 1..)?.cloned().collect();

        systems.input.confirm()?;
        let state_sets = systems.state.confirm(systems.input.synced_frame())?;
        if let Some(save) = self.systems.save.as_mut() {
            save.save_states(state_sets)?;
        }

        Ok(ret_states)
    }

    pub fn stop(&mut self) -> XResult<()> {
        self.systems.stop()?;
        Ok(())
    }

    #[inline]
    pub fn current_frame(&self) -> u32 {
        self.frame
    }

    #[inline]
    pub fn next_frame(&self) -> u32 {
        self.frame + 1
    }

    #[inline]
    pub fn phy_system(&self) -> &PhysicsSystem {
        &self.systems.physics
    }
}

//
// LogicSystems
//

pub struct LogicSystems {
    stopped: bool,
    pub tmpl_db: TmplDatabase,
    pub asset: AssetLoader,
    pub physics: PhysicsSystem,
    pub executor: Box<ScriptExecutor>,
    pub gene: SystemGeneration,
    pub input: SystemInput,
    pub state: SystemState,
    pub save: Option<SystemSave>,
}

impl LogicSystems {
    pub fn new<P: AsRef<Path>>(
        tmpl_db: TmplDatabase,
        asset_path: P,
        save_path: Option<PathBuf>,
    ) -> XResult<LogicSystems> {
        let mut physics = PhysicsSystem::new(
            BroadPhaseLayerInterfaceImpl::new_vbox(BroadPhaseLayerInterfaceImpl),
            ObjectVsBroadPhaseLayerFilterImpl::new_vbox(ObjectVsBroadPhaseLayerFilterImpl),
            ObjectLayerPairFilterImpl::new_vbox(ObjectLayerPairFilterImpl),
        );
        let engine = LogicSystems {
            stopped: false,
            tmpl_db,
            asset: AssetLoader::new(asset_path)?,
            physics,
            executor: ScriptExecutor::new(),
            gene: SystemGeneration::new(),
            input: SystemInput::new(MAX_INPUT_WINDOW),
            state: SystemState::new(),
            save: match save_path {
                Some(save_path) => Some(SystemSave::new(save_path)?),
                None => None,
            },
        };
        Ok(engine)
    }

    // #[inline]
    // pub fn context_assemble(&mut self) -> ContextAssemble<'_> {
    //     return ContextAssemble {
    //         tmpl_db: &self.tmpl_db,
    //         executor: &mut self.executor,
    //     };
    // }

    // #[inline]
    // pub fn context_update(&mut self, frame: u32, synced_frame: u32, new_cap: usize, update_cap: usize) -> ContextUpdate<'_> {
    //     return ContextUpdate::new(self, frame, synced_frame, new_cap, update_cap);
    // }

    fn stop(&mut self) -> XResult<()> {
        if self.stopped {
            return xres!(Unexpected; "system stopped");
        }
        self.stopped = true;
        Ok(())
    }
}

pub struct ContextUpdate<'t> {
    pub systems: &'t mut LogicSystems,
    pub frame: u32,
    pub synced_frame: u32,
}

impl Deref for ContextUpdate<'_> {
    type Target = LogicSystems;

    fn deref(&self) -> &Self::Target {
        self.systems
    }
}

impl DerefMut for ContextUpdate<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.systems
    }
}

impl<'t> ContextUpdate<'t> {
    #[inline]
    pub fn new(systems: &'t mut LogicSystems, frame: u32, synced_frame: u32) -> ContextUpdate<'t> {
        ContextUpdate {
            systems,
            frame,
            synced_frame,
        }
    }

    #[inline]
    pub fn context_assemble(&mut self) -> ContextAssemble<'_> {
        ContextAssemble {
            tmpl_db: &self.systems.tmpl_db,
            executor: &mut self.systems.executor,
        }
    }
}

pub struct ContextRestore {
    pub frame: u32,
    pub(crate) state_set: Arc<StateSet>,
}

impl ContextRestore {
    #[inline]
    pub fn new(state_set: Arc<StateSet>) -> ContextRestore {
        ContextRestore {
            frame: state_set.frame,
            state_set,
        }
    }

    #[inline]
    pub fn find(&self, id: NumID) -> XResult<&dyn StateAny> {
        for state in self.state_set.updates.iter() {
            if state.id == id {
                return Ok(state.as_ref());
            }
        }
        xres!(LogicNotFound, id)
    }

    #[inline]
    pub fn find_as<T: StateAny + 'static>(&self, id: NumID) -> XResult<&T> {
        for state in self.state_set.updates.iter() {
            if state.id == id {
                return state.cast_ref();
            }
        }
        xres!(LogicNotFound, id)
    }
}

//
// LogicGame
//

#[repr(C)]
#[derive(Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateGameInit {
    pub _base: StateAnyBase,
}

extend!(StateGameInit, StateAnyBase);

unsafe impl StateAny for StateGameInit {
    #[inline]
    fn typ(&self) -> StateType {
        assert_eq!(self.typ, StateType::GameInit);
        StateType::GameInit
    }

    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        assert_eq!(self.logic_typ, LogicType::Game);
        LogicType::Game
    }
}

impl ArchivedStateAny for rkyv::Archived<StateGameInit> {
    #[inline]
    fn typ(&self) -> StateType {
        StateType::GameInit
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        LogicType::Game
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
#[archive_attr(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateGameUpdate {
    pub _base: StateAnyBase,
    pub frame: u32,
    pub id_gen_counter: NumID,
}

extend!(StateGameUpdate, StateAnyBase);

unsafe impl StateAny for StateGameUpdate {
    #[inline]
    fn typ(&self) -> StateType {
        assert_eq!(self.typ, StateType::GameUpdate);
        StateType::GameUpdate
    }

    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        assert_eq!(self.logic_typ, LogicType::Game);
        LogicType::Game
    }
}

impl ArchivedStateAny for rkyv::Archived<StateGameUpdate> {
    #[inline]
    fn typ(&self) -> StateType {
        StateType::GameUpdate
    }

    #[inline]
    fn logic_typ(&self) -> LogicType {
        LogicType::Game
    }
}

#[derive(Debug)]
pub struct LogicGame {
    id: NumID,
    frame: u32, // Internal logical restorable frame
    stage: Box<LogicStage>,
    players: HistoryVec<Box<LogicPlayer>>,
    npces: HistoryVec<Box<LogicNpc>>,
}

impl LogicAny for LogicGame {
    #[inline]
    fn id(&self) -> NumID {
        self.id
    }

    #[inline]
    fn typ(&self) -> LogicType {
        LogicType::Game
    }

    #[inline]
    fn spawn_frame(&self) -> u32 {
        0
    }

    #[inline]
    fn death_frame(&self) -> u32 {
        u32::MAX
    }
}

impl LogicGame {
    pub fn new(
        ctx: &mut ContextUpdate<'_>,
        param_stage: ParamStage,
        param_players: Vec<ParamPlayer>,
    ) -> XResult<(Box<LogicGame>, Arc<StateSet>)> {
        let mut state_set = StateSet::new(0, 16, 0);

        let game_init = Arc::new(StateGameInit {
            _base: StateAnyBase::new(GAME_ID, StateType::GameInit, LogicType::Game),
        });
        state_set.inits.push(game_init);

        // new stage
        let (stage, stage_init) = LogicStage::new(ctx, &param_stage)?;
        state_set.inits.push(stage_init);

        // new players
        let mut logic_players = HistoryVec::with_capacity(param_players.len());
        for param_player in param_players {
            let (logic_player, player_init) = LogicPlayer::new(ctx, &param_player)?;
            logic_players.append_new(logic_player);
            state_set.inits.push(player_init);
        }

        // TODO: new ememies
        let logic_enemies = HistoryVec::new();

        let mut game = Box::new(LogicGame {
            id: GAME_ID,
            frame: 0,
            stage,
            players: logic_players,
            npces: logic_enemies,
        });

        state_set.updates = game.collect_states_updates(ctx)?;

        Ok((game, Arc::new(state_set)))
    }

    fn restore(&mut self, ctx: &ContextRestore) -> XResult<()> {
        self.frame = ctx.frame;
        self.stage.restore(ctx)?;

        self.players.restore_when(|player| {
            if player.death_frame() < self.frame {
                Ok(-1)
            } else if player.spawn_frame() > self.frame {
                return Ok(1);
            } else {
                player.restore(ctx)?;
                return Ok(0);
            }
        })?;
        // self.npces.restore(self.frame, |npc| {
        //     return npc.restore(ctx);
        // })?;
        Ok(())
    }

    pub fn update(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<Arc<StateSet>> {
        self.frame = ctx.frame;

        // TODO: Detect hits

        // TODO: Update values

        // TODO: Clear dead objects

        // Apply inputs to player
        for player in self.players.iter_mut_by(|p| p.is_alive()) {
            player.update(ctx)?;
        }

        self.stage.update(ctx)?;

        for npc in self.npces.iter_mut_by(|p| p.is_alive()) {
            npc.update_ai(ctx)?;
        }

        // Collect states
        let mut state_set = StateSet::new(self.frame, 0, 0);
        state_set.updates = self.collect_states_updates(ctx)?;
        Ok(Arc::new(state_set))
    }

    fn collect_states_updates(&mut self, ctx: &mut ContextUpdate<'_>) -> XResult<Vec<Box<dyn StateAny>>> {
        let mut updates: Vec<Box<dyn StateAny>> = Vec::with_capacity(1 + self.players.len() + self.npces.len());
        updates.push(self.stage.state());

        for player in self.players.iter_mut() {
            updates.push(player.state()?);
        }

        for npc in self.npces.iter_mut() {
            updates.push(npc.state());
        }

        updates.push(Box::new(StateGameUpdate {
            _base: StateAnyBase::new(self.id, StateType::GameUpdate, LogicType::Game),
            frame: self.frame,
            id_gen_counter: ctx.gene.counter(),
        }));
        Ok(updates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::{TEST_ASSET_PATH, TEST_TEMPLATE_PATH};
    use crate::utils::{sb, RawEvent, RawKey};

    #[test]
    #[ignore]
    fn test_logic_loop_common() {
        let tmpl_db = TmplDatabase::new(TEST_TEMPLATE_PATH).unwrap();
        let param_stage = ParamStage {
            stage: sb!("Stage.Demo"),
        };
        let param_player = ParamPlayer {
            character: sb!("Character.No1"),
            style: sb!("Style.No1-1"),
            level: 4,
            ..Default::default()
        };
        let (mut ll, _) = LogicLoop::new(tmpl_db, TEST_ASSET_PATH, param_stage, vec![param_player], None).unwrap();
        ll.update(vec![InputPlayerEvents {
            frame: 1,
            player_id: 100,
            events: vec![RawEvent::new_button(RawKey::Attack1, true)],
        }])
        .unwrap();
        // ll.stop().unwrap();
    }
}
