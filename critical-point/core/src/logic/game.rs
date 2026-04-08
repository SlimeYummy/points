use critical_point_csgen::CsOut;
use glam::Vec3A;
use jolt_physics_rs::PhysicsSystem;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::slice;
use std::sync::Arc;

use crate::asset::AssetLoader;
use crate::consts::{FPS, MAX_INPUT_WINDOW, SPF};
use crate::instance::ContextAssemble;
use crate::logic::base::{impl_state, LogicAny, LogicType, StateAny, StateBase, StateType};
use crate::logic::character::LogicCharacter;
use crate::logic::physics::{
    PhyBroadPhaseLayerInterface, PhyContactCollector, PhyHitCharacterEvent, PhyObjectLayerPairFilter,
    PhyObjectVsBroadPhaseLayerFilter,
};
use crate::logic::system::generation::{StateGeneration, SystemGeneration};
use crate::logic::system::input::{InputFrameInputs, InputPlayerInputs, SystemInput};
use crate::logic::system::save::SystemSave;
use crate::logic::system::state::{StateSet, SystemState};
use crate::logic::zone::LogicZone;
use crate::parameter::ParamGame;
// use crate::script::ScriptExecutor;
use crate::template::TmplDatabase;
use crate::utils::{bubble_sort_by, extend, force_mut, xres, Castable, HistoryVec, NumID, Symbol, XResult};

//
// LogicLoop
//

pub struct LogicLoop {
    systems: LogicSystems,
    game: Option<Box<LogicGame>>,
    frame: u32, // The current game frame for library user's side
    local_mode: bool,
}

impl Drop for LogicLoop {
    fn drop(&mut self) {
        self.game = None; // Ensure game is dropped before systems (especially PhysicsSystem)

        #[cfg(feature = "debug-print")]
        log::debug!("LogicLoop::drop()");
    }
}

impl LogicLoop {
    pub fn new<P: AsRef<Path>>(
        tmpl_db: TmplDatabase,
        asset_path: P,
        param: ParamGame,
        save_path: Option<PathBuf>,
    ) -> XResult<(LogicLoop, Arc<StateSet>)> {
        let local_mode = param.local_mode;
        if local_mode && param.players.len() != 1 {
            return xres!(BadArgument; "local mode only supports one player");
        }

        let mut systems = LogicSystems::new(tmpl_db, asset_path, save_path)?;
        systems.input.init(param.players.len())?;

        let mut ctx = ContextUpdate::new(&mut systems, 0, 0);
        let (game, state_set) = LogicGame::new(&mut ctx, param)?;
        systems.state.init(state_set.clone())?;

        systems.physics.optimize_broad_phase();

        let logic_loop = LogicLoop {
            systems,
            game: Some(game),
            frame: 0,
            local_mode,
        };
        Ok((logic_loop, state_set))
    }

    pub fn update(&mut self, player_events: Vec<InputPlayerInputs>) -> XResult<Arc<StateSet>> {
        if self.systems.stopped {
            return xres!(Unexpected; "system stopped");
        }

        println!("--------------------{}--------------------", self.frame);

        if self.local_mode {
            self.update_local(player_events)
        }
        else {
            self.update_online(player_events)
        }
    }

    fn update_local(&mut self, player_events: Vec<InputPlayerInputs>) -> XResult<Arc<StateSet>> {
        if player_events.len() != 1 {
            return xres!(BadArgument; "local mode must have one InputPlayerInputs per frame");
        }

        let systems = &mut self.systems;
        let game = self.game.as_mut().unwrap();
        self.frame += 1;

        if let Some(save) = systems.save.as_mut() {
            let player_events = InputFrameInputs::new(self.frame, &player_events);
            save.save_input(player_events)?;
        }

        let base_frame = systems.input.produce(&player_events)?;
        assert_eq!(base_frame, game.frame);

        let synced_frame = systems.input.synced_frame();
        assert_eq!(synced_frame, self.frame);

        let mut cl = PhyContactCollector::new_vpair(PhyContactCollector::new(game));
        systems
            .physics
            .update_with_listeners::<_, ()>(SPF, 1, Some(&mut cl), None)?;

        let mut ctx = ContextUpdate::new(systems, game.frame + 1, synced_frame);
        let state_set = game.update(&mut ctx)?;

        systems.state.append(state_set.clone())?;
        let ret_state = systems.state[game.frame].clone();
        assert_eq!(self.frame, game.frame);

        systems.gene.update(game.frame);

        systems.input.confirm()?;
        let state_sets = systems.state.confirm(systems.input.synced_frame())?;
        if let Some(save) = self.systems.save.as_mut() {
            save.save_states(state_sets)?;
        }

        Ok(ret_state)
    }

    fn update_online(&mut self, mut player_events: Vec<InputPlayerInputs>) -> XResult<Arc<StateSet>> {
        let systems = &mut self.systems;
        let game = self.game.as_mut().unwrap();
        self.frame += 1;

        if let Some(save) = systems.save.as_mut() {
            let player_events = InputFrameInputs::new(self.frame, &player_events);
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
            debug_assert_eq!(game.frame, base_frame);
        }

        // Update frame to current
        while game.frame < self.frame {
            let frame = game.frame + 1;
            let synced_frame = systems.input.synced_frame();
            let mut ctx = ContextUpdate::new(systems, frame, synced_frame);
            let state_set = game.update(&mut ctx)?;
            // systems.physics.update(delta);
            systems.state.append(state_set.clone())?;

            systems.gene.update(frame);
        }
        let ret_state = systems.state[game.frame].clone();

        systems.input.confirm()?;
        let state_sets = systems.state.confirm(systems.input.synced_frame())?;
        if let Some(save) = self.systems.save.as_mut() {
            save.save_states(state_sets)?;
        }

        Ok(ret_state)
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
    pub(crate) tmpl_db: TmplDatabase,
    pub(crate) asset: AssetLoader,
    pub(crate) gene: SystemGeneration,
    pub(crate) input: SystemInput,
    pub(crate) state: SystemState,
    pub(crate) save: Option<SystemSave>,
    pub(crate) physics: PhysicsSystem,
}

#[cfg(feature = "debug-print")]
impl Drop for LogicSystems {
    fn drop(&mut self) {
        log::debug!("LogicSystems::drop()");
    }
}

impl LogicSystems {
    pub(crate) fn new<P: AsRef<Path>>(
        tmpl_db: TmplDatabase,
        asset_path: P,
        save_path: Option<PathBuf>,
    ) -> XResult<LogicSystems> {
        let physics = PhysicsSystem::new(
            PhyBroadPhaseLayerInterface::new_vbox(PhyBroadPhaseLayerInterface),
            PhyObjectVsBroadPhaseLayerFilter::new_vbox(PhyObjectVsBroadPhaseLayerFilter),
            PhyObjectLayerPairFilter::new_vbox(PhyObjectLayerPairFilter),
        );

        let system = LogicSystems {
            stopped: false,
            tmpl_db,
            asset: AssetLoader::new(asset_path)?,
            physics,
            // executor: ScriptExecutor::new(),
            gene: SystemGeneration::new(),
            input: SystemInput::new(MAX_INPUT_WINDOW),
            state: SystemState::new(),
            save: match save_path {
                Some(save_path) => Some(SystemSave::new(save_path)?),
                None => None,
            },
        };
        Ok(system)
    }

    fn stop(&mut self) -> XResult<()> {
        if self.stopped {
            return xres!(Unexpected; "system stopped");
        }
        self.stopped = true;
        Ok(())
    }
}

//
// Contexts
//

pub struct ContextUpdate<'t> {
    pub(crate) systems: &'t mut LogicSystems,
    pub(crate) frame: u32,
    pub(crate) synced_frame: u32,
    pub(crate) time: f32,
    pub(crate) synced_time: f32,
    pub(crate) hit_events: &'t [HitCharacterEvent],
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
    pub(crate) fn new(systems: &'t mut LogicSystems, frame: u32, synced_frame: u32) -> ContextUpdate<'t> {
        ContextUpdate {
            systems,
            frame,
            synced_frame,
            time: frame as f32 / FPS, // TODO: The error between time and accumulation time
            synced_time: synced_frame as f32 / FPS,
            hit_events: &[],
        }
    }

    #[inline]
    pub(crate) fn context_assemble(&mut self) -> ContextAssemble<'_> {
        ContextAssemble {
            tmpl_db: &self.systems.tmpl_db,
            // executor: &mut self.systems.executor,
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
                return state.cast();
            }
        }
        xres!(LogicNotFound, id)
    }
}

pub struct ContextHitGenerate<'t, E> {
    pub(crate) frame: u32,
    pub(crate) time: f32,
    pub(crate) events: &'t mut Vec<E>,
}

impl<'t, E> ContextHitGenerate<'t, E> {
    #[inline]
    pub(crate) fn new(frame: u32, events: &'t mut Vec<E>) -> ContextHitGenerate<'t, E> {
        ContextHitGenerate {
            frame,
            time: frame as f32 / FPS,
            events,
        }
    }

    #[inline]
    pub(crate) fn context_update(&mut self, idx: usize) -> ContextHitUpdate<'_, E> {
        ContextHitUpdate::new(self.frame, &mut self.events[idx])
    }
}

pub struct ContextHitUpdate<'t, E> {
    pub(crate) frame: u32,
    pub(crate) time: f32,
    pub(crate) event: &'t mut E,
}

impl<'t, E> ContextHitUpdate<'t, E> {
    #[inline]
    pub(crate) fn new(frame: u32, event: &'t mut E) -> ContextHitUpdate<'t, E> {
        ContextHitUpdate {
            frame,
            time: frame as f32 / FPS,
            event,
        }
    }
}

//
// LogicGame
//

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateGameInit {
    pub _base: StateBase,
}

extend!(StateGameInit, StateBase);

impl_state!(StateGameInit, Game, GameInit, "GameInit");

#[repr(C)]
#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Ref)]
pub struct StateGameUpdate {
    pub _base: StateBase,
    pub frame: u32,
    pub gene: StateGeneration,
    pub hit_events: Vec<HitCharacterEvent>,
}

extend!(StateGameUpdate, StateBase);

impl_state!(StateGameUpdate, Game, GameUpdate, "GameUpdate");

#[derive(Debug)]
pub struct LogicGame {
    id: NumID,
    frame: u32, // Internal logical restorable frame, always equal to ctx.frame
    zone: Box<LogicZone>,
    characters: HistoryVec<Box<LogicCharacter>>,
    hit_events: Vec<HitCharacterEvent>,
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
    fn new(ctx: &mut ContextUpdate, param: ParamGame) -> XResult<(Box<LogicGame>, Arc<StateSet>)> {
        let mut state_set = StateSet::new(0, 16, 0);

        let game_init = Arc::new(StateGameInit {
            _base: StateBase::new(NumID::GAME, StateType::GameInit, LogicType::Game),
        });
        state_set.inits.push(game_init);

        // new zone
        let (zone, zone_init) = LogicZone::new(ctx, &param.zone)?;
        state_set.inits.push(zone_init);

        // new players & npcs
        let mut logic_characters = HistoryVec::with_capacity(param.players.len() + param.npcs.len());

        for param_player in param.players {
            let (logic_player, player_init) = LogicCharacter::new_player(ctx, &param_player)?;
            logic_characters.append_new(logic_player);
            state_set.inits.push(player_init);
        }

        for param_npc in param.npcs {
            let (logic_npc, npc_init) = LogicCharacter::new_npc(ctx, &param_npc)?;
            logic_characters.append_new(logic_npc);
            state_set.inits.push(npc_init);
        }

        let mut game = Box::new(LogicGame {
            id: NumID::GAME,
            frame: 0,
            zone,
            characters: logic_characters,
            hit_events: Vec::with_capacity(32),
        });

        state_set.updates = game.collect_states_updates(ctx)?;

        Ok((game, Arc::new(state_set)))
    }

    fn restore(&mut self, ctx: &ContextRestore) -> XResult<()> {
        self.frame = ctx.frame;
        self.zone.restore(ctx)?;

        self.characters.restore_when(|chara| {
            if chara.death_frame() < self.frame {
                Ok(-1)
            }
            else if chara.spawn_frame() > self.frame {
                return Ok(1);
            }
            else {
                chara.restore(ctx)?;
                return Ok(0);
            }
        })?;
        Ok(())
    }

    fn update(&mut self, ctx: &mut ContextUpdate) -> XResult<Arc<StateSet>> {
        self.frame = ctx.frame;

        let ptr = self.hit_events.as_ptr();
        let len = self.hit_events.len();
        // Safety: steal a reference, we will reset ctx.hit_events after use.
        ctx.hit_events = unsafe { slice::from_raw_parts(ptr, len) };

        // Update character hit & value
        for chara in self.characters.iter_mut_by(|p| p.is_alive()) {
            chara.update_hit(ctx)?;
            chara.update_value(ctx)?;
        }

        // TODO: Create new objects
        // TODO: Clear dead objects

        // Update character action & physics
        for chara in self.characters.iter_mut_by(|p| p.is_alive()) {
            chara.update_action(ctx)?;
            chara.update_physics(ctx)?;
        }

        self.zone.update(ctx)?;

        ctx.hit_events = &[];

        // Collect states
        let mut state_set = StateSet::new(self.frame, 0, 0);
        state_set.updates = self.collect_states_updates(ctx)?;

        // Clean up hit events
        self.hit_events.clear();
        for chara in self.characters.iter_mut_by(|p| p.is_alive()) {
            chara.update_clean_up();
        }

        Ok(Arc::new(state_set))
    }

    fn collect_states_updates(&mut self, ctx: &mut ContextUpdate) -> XResult<Vec<Box<dyn StateAny>>> {
        let mut updates: Vec<Box<dyn StateAny>> = Vec::with_capacity(1 + self.characters.len());
        updates.push(Box::new(StateGameUpdate {
            _base: StateBase::new(self.id, StateType::GameUpdate, LogicType::Game),
            frame: self.frame,
            gene: ctx.gene.state(),
            hit_events: self.hit_events.drain(..).collect(),
        }));

        updates.push(self.zone.state());

        for chara in self.characters.iter_mut() {
            updates.push(chara.state()?);
        }
        Ok(updates)
    }

    pub(crate) fn on_hit_character<'t>(&mut self, phy_event: &PhyHitCharacterEvent<'t>) -> XResult<()> {
        let Some(src) = self.characters.iter().position(|c| c.id() == phy_event.src_chara_id)
        else {
            log::warn!("Src Character not found ({})", phy_event.src_chara_id);
            return Ok(());
        };
        let Some(dst) = self.characters.iter().position(|c| c.id() == phy_event.dst_chara_id)
        else {
            log::warn!("Dst Character not found ({})", phy_event.dst_chara_id);
            return Ok(());
        };

        let mut ctx = ContextHitGenerate::new(self.frame, &mut self.hit_events);
        let src_chara = unsafe { force_mut(&self.characters[src]) };
        let dst_chara = unsafe { force_mut(&self.characters[dst]) };
        src_chara.before_hit(dst_chara, &mut ctx, phy_event)?;

        Ok(())
    }
}

//
// Hit Event
//

#[repr(C)]
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    CsOut,
)]
#[rkyv(derive(Debug))]
#[cs_attr(Value)]
pub struct HitCharacterEvent {
    pub src_chara_id: NumID,
    pub dst_chara_id: NumID,
    pub group: Symbol,
    pub box_index: u16,
    pub group_index: u16,
    pub box_hit_times: u16,
    pub group_hit_times: u16,
    // Normal for this collision, direction along which to move dst_chara out of collision along the shortest path.
    pub collision_normal: Vec3A,
    // The average position of all collision points
    pub collision_point_average: Vec3A,
    // The vector pointing from the src_chara position to the dst_chara position.
    pub character_vector: Vec3A,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_ASSET_PATH;
    use crate::parameter::{ParamNpc, ParamPlayer, ParamZone};
    use crate::utils::{id, RawInput, RawKey};

    #[ctor::ctor]
    fn test_init_jolt_physics() {
        jolt_physics_rs::global_initialize();
    }

    #[test]
    fn test_logic_loop_common() {
        let tmpl_db = TmplDatabase::new(10240, 150).unwrap();
        let param = ParamGame {
            zone: ParamZone { zone: id!("Zone.Demo") },
            players: vec![ParamPlayer {
                character: id!("Character.One"),
                style: id!("Style.One^1"),
                level: 4,
                ..Default::default()
            }],
            npcs: vec![ParamNpc {
                character: id!("NpcCharacter.NpcInstance^1"),
                level: 2,
                ..Default::default()
            }],
            local_mode: false,
        };
        let (mut ll, _) = LogicLoop::new(tmpl_db, TEST_ASSET_PATH, param, None).unwrap();
        ll.update(vec![InputPlayerInputs {
            frame: 1,
            player_id: NumID::MIN_PLAYER,
            inputs: vec![RawInput::new_button(RawKey::Attack1, true)],
        }])
        .unwrap();
        // ll.update(vec![]).unwrap();
        // ll.update(vec![]).unwrap();
        // // ll.update(vec![]).unwrap();
        // ll.stop().unwrap();
    }
}
