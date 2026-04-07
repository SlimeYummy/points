use critical_point_csgen::CsOut;
use jolt_physics_rs::{self, PhysicsSystem};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::instance::{assemble_npc, assemble_player, ContextAssemble, InstCharacter};
use crate::logic::{InputPlayerInputs, LogicLoop, PhyContactCollector, StateSet};
use crate::parameter::{verify_npc, verify_player, ContextVerify, ParamGame, ParamNpc, ParamPlayer};
use crate::template::TmplDatabase;
use crate::utils::{xerr, xres, XResult};

#[derive(Debug)]
pub struct EnvPath {
    pub tmpl_path: PathBuf,
    pub asset_path: PathBuf,
}

pub static mut ENV_PATH: EnvPath = EnvPath {
    tmpl_path: PathBuf::new(),
    asset_path: PathBuf::new(),
};

pub struct LogicEngine {
    tmpl_database: TmplDatabase,
    logic_loop: Option<LogicLoop>,
}

#[cfg(feature = "debug-print")]
impl Drop for LogicEngine {
    fn drop(&mut self) {
        log::info!("LogicEngine::drop()");
    }
}

impl LogicEngine {
    pub fn initialize<TP: AsRef<Path>, AP: AsRef<Path>>(tmpl_path: TP, asset_path: AP) -> XResult<()> {
        log::info!(
            "LogicEngine::initialize() tmpl_path={:?} asset_path={:?}",
            tmpl_path.as_ref(),
            asset_path.as_ref()
        );
        unsafe {
            ENV_PATH.tmpl_path = PathBuf::from(tmpl_path.as_ref());
            ENV_PATH.asset_path = PathBuf::from(asset_path.as_ref());
        }

        unsafe {
            crate::utils::init_id_static(&tmpl_path, true)?;
            crate::template::init_database_static(&tmpl_path, true)?;
        };

        jolt_physics_rs::global_initialize();

        log::info!("LogicEngine::initialize() OK");
        Ok(())
    }

    pub fn new() -> XResult<LogicEngine> {
        log::info!("LogicEngine::new()");

        let engine = LogicEngine {
            tmpl_database: TmplDatabase::new(1024 * 1024, 60)?,
            // script_executor: ScriptExecutor::new(),
            logic_loop: None,
        };

        log::info!("LogicEngine::new() OK");
        Ok(engine)
    }

    #[inline]
    pub fn phy_system(&self) -> Option<&PhysicsSystem> {
        self.logic_loop.as_ref().map(|logic_loop| logic_loop.phy_system())
    }

    #[inline]
    pub fn verify_player(&mut self, param: &ParamPlayer) -> XResult<()> {
        let mut ctx = ContextVerify::new(&self.tmpl_database);
        verify_player(&mut ctx, param)
    }

    #[inline]
    pub fn assemble_player(&mut self, param: ParamPlayer) -> XResult<InstCharacter> {
        let mut ctx = ContextAssemble::new(&self.tmpl_database);
        assemble_player(&mut ctx, &param)
    }

    #[inline]
    pub fn verify_npc(&mut self, param: &ParamNpc) -> XResult<()> {
        let mut ctx = ContextVerify::new(&self.tmpl_database);
        verify_npc(&mut ctx, param)
    }

    #[inline]
    pub fn assemble_npc(&mut self, param: ParamNpc) -> XResult<InstCharacter> {
        let mut ctx = ContextAssemble::new(&self.tmpl_database);
        assemble_npc(&mut ctx, &param)
    }

    #[inline]
    pub fn is_game_running(&self) -> bool {
        self.logic_loop.is_some()
    }

    #[inline]
    pub fn current_frame(&self) -> u32 {
        match &self.logic_loop {
            Some(logic_loop) => logic_loop.current_frame(),
            None => 0,
        }
    }

    #[inline]
    pub fn next_frame(&self) -> u32 {
        match &self.logic_loop {
            Some(logic_loop) => logic_loop.next_frame(),
            None => 0,
        }
    }

    pub fn start_game(&mut self, param: ParamGame, save_path: Option<PathBuf>) -> XResult<Arc<StateSet>> {
        log::info!("LogicEngine::new() param={:?} save_path={:?}", &param, &save_path);

        if self.logic_loop.is_some() {
            return xres!(Unexpected; "game already running");
        }

        for p in &param.players {
            self.verify_player(p)?;
        }
        for n in &param.npcs {
            self.verify_npc(n)?;
        }

        let (logic_loop, state_set) = LogicLoop::new(
            self.tmpl_database.clone(),
            #[allow(static_mut_refs)]
            unsafe {
                ENV_PATH.asset_path.clone()
            },
            param,
            save_path,
        )?;
        self.logic_loop = Some(logic_loop);

        log::info!("LogicEngine::start_game() OK");
        Ok(state_set)
    }

    pub fn update_game(&mut self, player_events: Vec<InputPlayerInputs>) -> XResult<Arc<StateSet>> {
        // log::info!("player_events {:?}", player_events);
        let logic_loop = self
            .logic_loop
            .as_mut()
            .ok_or_else(|| xerr!(Unexpected; "game not running"))?;
        logic_loop.update(player_events)
    }

    pub fn stop_game(&mut self) -> XResult<()> {
        log::info!("LogicEngine::stop_game()");

        let logic_loop = self
            .logic_loop
            .as_mut()
            .ok_or_else(|| xerr!(Unexpected; "game not running"))?;
        logic_loop.stop()?;
        self.logic_loop = None;

        log::info!("LogicEngine::stop_game() OK");
        Ok(())
    }
}

#[repr(C)]
#[derive(Debug, Default, CsOut)]
pub struct LogicEngineStatus {
    pub is_game_running: bool,
    pub current_frame: u32,
    pub next_frame: u32,
}
