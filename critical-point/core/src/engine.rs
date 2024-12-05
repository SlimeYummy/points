use cirtical_point_csgen::CsOut;
use jolt_physics_rs::PhysicsSystem;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::instance::{assemble_player, ContextAssemble, InstPlayer};
use crate::logic::{LogicLoop, PlayerKeyEvents, StateSet};
use crate::parameter::{verify_player, ContextVerify, ParamPlayer, ParamStage};
use crate::script::ScriptExecutor;
use crate::template::TmplDatabase;
use crate::utils::{XError, XResult};

pub struct LogicEngine {
    tmpl_database: TmplDatabase,
    asset_path: PathBuf,
    script_executor: Box<ScriptExecutor>,
    logic_loop: Option<LogicLoop>,
}

#[cfg(debug_assertions)]
impl Drop for LogicEngine {
    fn drop(&mut self) {
        println!("LogicEngine dropped");
    }
}

impl LogicEngine {
    pub fn new<TP, AP>(tmpl_path: TP, asset_path: AP) -> XResult<LogicEngine>
    where
        TP: AsRef<Path>,
        AP: AsRef<Path>,
    {
        let engine = LogicEngine {
            tmpl_database: TmplDatabase::new(tmpl_path)?,
            asset_path: PathBuf::from(asset_path.as_ref()),
            script_executor: ScriptExecutor::new(),
            logic_loop: None,
        };
        Ok(engine)
    }

    #[inline]
    pub fn phy_system(&self) -> Option<&PhysicsSystem> {
        self.logic_loop.as_ref().map(|logic_loop| logic_loop.phy_system())
    }

    pub fn verify_player(&mut self, param: &ParamPlayer) -> XResult<()> {
        let mut ctx = ContextVerify::new(&self.tmpl_database);
        verify_player(&mut ctx, param)
    }

    pub fn assemble_player(&mut self, param: ParamPlayer) -> XResult<InstPlayer> {
        let mut ctx = ContextAssemble::new(&self.tmpl_database, &mut self.script_executor);
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

    pub fn start_game(&mut self, param_stage: ParamStage, param_players: Vec<ParamPlayer>) -> XResult<Arc<StateSet>> {
        if self.logic_loop.is_some() {
            return Err(XError::unexpected("Game already running"));
        }
        let (logic_loop, state_set) =
            LogicLoop::new(self.tmpl_database.clone(), &self.asset_path, param_stage, param_players)?;
        self.logic_loop = Some(logic_loop);
        Ok(state_set)
    }

    pub fn update_game(&mut self, player_keys: Vec<PlayerKeyEvents>) -> XResult<Vec<Arc<StateSet>>> {
        let logic_loop = self
            .logic_loop
            .as_mut()
            .ok_or_else(|| XError::unexpected("Game not running"))?;
        logic_loop.update(player_keys)
    }

    pub fn stop_game(&mut self) -> XResult<()> {
        let logic_loop = self
            .logic_loop
            .as_mut()
            .ok_or_else(|| XError::unexpected("Game not running"))?;
        logic_loop.stop()?;
        self.logic_loop = None;
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
