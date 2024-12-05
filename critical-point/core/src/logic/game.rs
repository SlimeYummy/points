use cirtical_point_csgen::CsOut;
use jolt_physics_rs::{BodyInterface, PhysicsSystem};
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::Arc;

use crate::asset::AssetLoader;
use crate::consts::MAX_INPUT_WINDOW;
use crate::instance::ContextAssemble;
use crate::logic::base::{ArchivedStateAny, LogicAny, LogicType, StateAny, StateAnyBase, StateType};
use crate::logic::character::{LogicNpc, LogicPlayer};
use crate::logic::stage::LogicStage;
use crate::logic::system::generation::SystemGeneration;
use crate::logic::system::input::{PlayerKeyEvents, SystemInput};
use crate::logic::system::save::SaveSystem;
use crate::logic::system::state::{StateSet, SystemState};
use crate::parameter::{ParamPlayer, ParamStage};
use crate::script::ScriptExecutor;
use crate::template::TmplDatabase;
use crate::utils::{bubble_sort_by, extend, CastRef, HistoryVec, NumID, XError, XResult};

//
// LogicLoop
//

pub struct LogicLoop {
    systems: LogicSystems,
    game: Box<LogicGame>,
    frame: u32,
}

impl LogicLoop {
    pub fn new<P: AsRef<Path>>(
        tmpl_db: TmplDatabase,
        asset_path: P,
        param_stage: ParamStage,
        param_players: Vec<ParamPlayer>,
        save_path: Option<P>,
    ) -> XResult<(LogicLoop, Arc<StateSet>)> {
        let mut systems = LogicSystems::new(tmpl_db, asset_path, save_path)?;
        let mut ctx = ContextUpdate {
            systems: &mut systems,
            frame: 0,
            synced_frame: 0,
            state_set: StateSet::new(0, 16, 16),
        };
        let game = LogicGame::new(&mut ctx, param_stage, param_players)?;

        let player_ids = game.players.iter().map(|p| p.id()).collect::<Vec<_>>();
        ctx.systems.input.init(&player_ids)?;

        let state_set = Arc::new(ctx.state_set); // take state_set from ctx
        systems.state.init(state_set.clone())?;

        let logic_loop = LogicLoop {
            systems,
            game,
            frame: 0,
        };
        Ok((logic_loop, state_set))
    }

    pub fn update(&mut self, mut players_events: Vec<PlayerKeyEvents>) -> XResult<Vec<Arc<StateSet>>> {
        if self.systems.stopped {
            return Err(XError::unexpected("LogicLoop::update() already stopped"));
        }

        if let Some(save) = self.systems.save.as_mut() {
            save.save_inputs(&players_events)?;
        }

        let systems = &mut self.systems;
        let game = &mut self.game;
        self.frame += 1;

        // Insert new input events
        bubble_sort_by(&mut players_events, |a, b| {
            a.player_id < b.player_id && a.frame < b.frame
        });
        let base_frame = systems
            .input
            .produce(&players_events)?
            .unwrap_or(game.frame)
            .min(game.frame);

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
                state_set: StateSet::new(frame, 0, 1 + game.players.len()),
            };
            game.update(&mut ctx)?;

            let state_set = Arc::new(ctx.state_set);
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
    pub physics: Box<PhysicsSystem>,
    pub body_itf: BodyInterface,
    pub executor: Box<ScriptExecutor>,
    pub gene: SystemGeneration,
    pub input: SystemInput,
    pub state: SystemState,
    pub save: Option<SaveSystem>,
}

impl LogicSystems {
    pub fn new<P: AsRef<Path>>(
        tmpl_db: TmplDatabase,
        asset_path: P,
        save_path: Option<P>,
    ) -> XResult<LogicSystems> {
        let mut physics = PhysicsSystem::new();
        let body_itf = physics.body_interface(false);
        let engine = LogicSystems {
            stopped: false,
            tmpl_db,
            asset: AssetLoader::new(body_itf.clone(), asset_path)?,
            physics,
            body_itf,
            executor: ScriptExecutor::new(),
            gene: SystemGeneration::new(1),
            input: SystemInput::new(MAX_INPUT_WINDOW)?,
            state: SystemState::new(),
            save: match save_path {
                Some(save_path) => Some(SaveSystem::new(save_path)?),
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
            return Err(XError::unexpected("LogicSystems::stop() already stopped"));
        }
        self.stopped = true;
        Ok(())
    }
}

pub struct ContextUpdate<'t> {
    pub systems: &'t mut LogicSystems,
    pub frame: u32,
    pub synced_frame: u32,
    pub(crate) state_set: StateSet,
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
    pub fn new(
        systems: &'t mut LogicSystems,
        frame: u32,
        synced_frame: u32,
        new_cap: usize,
        update_cap: usize,
    ) -> ContextUpdate<'t> {
        ContextUpdate {
            systems,
            frame,
            synced_frame,
            state_set: StateSet::new(frame, new_cap, update_cap),
        }
    }

    #[inline]
    pub fn new_empty(systems: &'t mut LogicSystems) -> ContextUpdate<'t> {
        ContextUpdate {
            systems,
            frame: 0,
            synced_frame: 0,
            state_set: StateSet::new(0, 0, 0),
        }
    }

    #[inline]
    pub fn context_assemble(&mut self) -> ContextAssemble<'_> {
        ContextAssemble {
            tmpl_db: &self.systems.tmpl_db,
            executor: &mut self.systems.executor,
        }
    }

    #[inline]
    pub fn state_init(&mut self, state: Arc<dyn StateAny>) {
        self.state_set.inits.push(state);
    }

    #[inline]
    pub fn state_update(&mut self, state: Box<dyn StateAny>) {
        self.state_set.updates.push(state);
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
        Err(XError::not_found(format!("ContextRestore::find() {}", id)))
    }

    #[inline]
    pub fn find_as<T: StateAny + 'static>(&self, id: NumID) -> XResult<&T> {
        for state in self.state_set.updates.iter() {
            if state.id == id {
                return state.cast_ref();
            }
        }
        Err(XError::not_found(format!("ContextRestore::find() {}", id)))
    }
}

//
// LogicGame
//

#[repr(C)]
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
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
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut)]
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
