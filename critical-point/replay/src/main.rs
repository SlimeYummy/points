use anyhow::Result;
use glam::{Quat, Vec3, Vec3A};
use jolt_physics_rs::{
    global_initialize, run_debug_application, CameraState, DebugApplication, DebugKeyboard, RefPhysicsSystem,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

use cirtical_point_core::animation::SkeletonJointMeta;
use cirtical_point_core::engine::{LogicEngine, LogicEngineStatus};
use cirtical_point_core::logic::{PlayerKeyEvents, StateAction, StateAny, StateSet};
use cirtical_point_core::parameter::{ParamPlayer, ParamStage};
use cirtical_point_core::utils::{s, Symbol, XError, XResult};

#[derive(StructOpt, Debug)]
#[structopt(name = "replay")]
struct Opt {
    #[structopt(short, long, default_value = "")]
    config: String,
}

struct Application {
    engine: Box<LogicEngine>,
}

impl DebugApplication for Application {
    fn get_ref_system(&mut self) -> RefPhysicsSystem {
        self.engine.phy_system().unwrap().inner_ref().clone()
    }

    fn render_frame(&mut self, delta: f32, keyboard: &mut DebugKeyboard, camera: &CameraState) -> bool {
        self.engine.update_game(vec![]).unwrap();
        true
    }

    fn get_camera_pivot(&self, heading: f32, pitch: f32) -> Vec3A {
        let fwd = Vec3A::new(pitch.cos() * heading.cos(), pitch.sin(), pitch.cos() * heading.sin());
        // if let Some(chara) = &self.chara_common {
        //     let pos = chara.get_position(false);
        //     let ret = Vec3A::new(pos.x, pos.y + 1.0, pos.z) - 5.0 * fwd;
        //      ret;
        // }
        // if let Some(chara) = &self.chara_virtual {
        //     let pos = chara.get_position();
        //     let ret = Vec3A::new(pos.x, pos.y + 1.0, pos.z) - 5.0 * fwd;
        //      ret;
        // }
        Vec3A::new(1.0, 0.0, 1.0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    pub template_dir: PathBuf,
    pub asset_dir: PathBuf,
    pub stage: ParamStage,
    pub players: Vec<ParamPlayer>,
}

impl Config {
    fn from_path<P: AsRef<Path>>(config_path: P) -> Result<Config> {
        let buf = fs::read_to_string(config_path).unwrap();
        Ok(serde_json::from_str(&buf)?)
    }
}

fn new_application() -> Box<dyn DebugApplication> {
    let app: Result<_> = (|| {
        let opt = Opt::from_args();
        let mut cfg_path = opt.config.clone();
        if cfg_path.is_empty() {
            cfg_path = "./config.json".to_string();
        }
        let cfg = Config::from_path(&cfg_path)?;

        if !cfg.template_dir.is_dir() {
            panic!("template not found: {:?}", cfg.template_dir);
        }
        if !cfg.asset_dir.is_dir() {
            panic!("asset not found: {:?}", cfg.asset_dir);
        }

        let mut engine = Box::new(LogicEngine::new(cfg.template_dir, cfg.asset_dir)?);
        engine.start_game(cfg.stage, cfg.players)?;
        Ok(Box::new(Application { engine }))
    })();
    match app {
        Ok(app) => app,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn main() {
    global_initialize();
    run_debug_application(new_application);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_application() -> Box<dyn DebugApplication> {
        let app: Result<_> = (|| {
            let mut engine = Box::new(LogicEngine::new("D:/project/G1/_/Templates", "D:/project/G1/_")?);
            let config = Config::from_path("config.json")?;
            engine.start_game(config.stage, config.players)?;
            Ok(Box::new(Application { engine }))
        })();
        match app {
            Ok(app) => app,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    #[test]
    fn test() {
        global_initialize();
        run_debug_application(test_application);
    }
}
