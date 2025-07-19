use cirtical_point_csgen::CsOut;
use jolt_physics_rs::{global_initialize, PhysicsSystem};
use log::{debug, info};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::instance::{assemble_player, ContextAssemble, InstPlayer};
use crate::logic::{InputPlayerEvents, LogicLoop, StateSet};
use crate::parameter::{verify_player, ContextVerify, ParamPlayer, ParamZone};
use crate::template::TmplDatabase;
use crate::utils::{xerr, xres, XResult};

pub struct LogicEngine {
    tmpl_database: TmplDatabase,
    asset_path: PathBuf,
    logic_loop: Option<LogicLoop>,
}

#[cfg(feature = "debug-print")]
impl Drop for LogicEngine {
    fn drop(&mut self) {
        debug!("LogicEngine::drop()");
    }
}

impl LogicEngine {
    pub fn initialize<P: AsRef<Path>>(tmpl_path: P) -> XResult<()> {
        env_logger::init();
        info!("LogicEngine::initialize() tmpl_path={:?}", tmpl_path.as_ref());

        unsafe {
            crate::utils::init_id_static(&tmpl_path)?;
            crate::template::init_database_static(&tmpl_path)?;
        };
        global_initialize();

        info!("LogicEngine::initialize() OK");
        Ok(())
    }

    pub fn new<P: AsRef<Path>>(asset_path: P) -> XResult<LogicEngine> {
        info!("LogicEngine::new() asset_path={:?}", asset_path.as_ref());

        let engine = LogicEngine {
            tmpl_database: TmplDatabase::new(1024 * 1024, 60)?,
            asset_path: PathBuf::from(asset_path.as_ref()),
            // script_executor: ScriptExecutor::new(),
            logic_loop: None,
        };

        info!("LogicEngine::new() OK");
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
    pub fn assemble_player(&mut self, param: ParamPlayer) -> XResult<InstPlayer> {
        let mut ctx = ContextAssemble::new(&self.tmpl_database);
        assemble_player(&mut ctx, &param)
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

    pub fn start_game(
        &mut self,
        param_zone: ParamZone,
        param_players: Vec<ParamPlayer>,
        save_path: Option<PathBuf>,
    ) -> XResult<Arc<StateSet>> {
        info!(
            "LogicEngine::new() param_zone={:?} param_players={:?} save_path={:?}",
            &param_zone, &param_players, &save_path
        );

        if self.logic_loop.is_some() {
            return xres!(Unexpected; "game already running");
        }

        let (logic_loop, state_set) = LogicLoop::new(
            self.tmpl_database.clone(),
            &self.asset_path,
            param_zone,
            param_players,
            save_path,
        )?;
        self.logic_loop = Some(logic_loop);

        info!("LogicEngine::start_game() OK");
        Ok(state_set)
    }

    pub fn update_game(&mut self, player_events: Vec<InputPlayerEvents>) -> XResult<Vec<Arc<StateSet>>> {
        let logic_loop = self
            .logic_loop
            .as_mut()
            .ok_or_else(|| xerr!(Unexpected; "game not running"))?;
        logic_loop.update(player_events)
    }

    pub fn stop_game(&mut self) -> XResult<()> {
        info!("LogicEngine::stop_game()");

        let logic_loop = self
            .logic_loop
            .as_mut()
            .ok_or_else(|| xerr!(Unexpected; "game not running"))?;
        logic_loop.stop()?;
        self.logic_loop = None;

        info!("LogicEngine::stop_game() OK");
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
