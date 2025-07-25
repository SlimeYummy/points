mod input;

use anyhow::Result;
use chrono::Local;
use cirtical_point_core::consts::{DEFAULT_VIEW_DIR_3D, FPS};
use cirtical_point_core::engine::LogicEngine;
use cirtical_point_core::logic::{InputPlayerEvents, StatePlayerUpdate, StateSet};
use cirtical_point_core::parameter::{ParamPlayer, ParamZone};
use cirtical_point_core::utils::{Castable, NumID};
use glam::{Vec2, Vec3A};
use input::{CharacterType, InputHandler};
use jolt_physics_rs::debug::{run_debug_application, CameraState, DebugApp, DebugKeyboard, DebugMouse};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use structopt::StructOpt;

const F_FPS: f32 = FPS as f32;
const FRAC_FPS: f32 = 1.0 / F_FPS;

const PLAYER_ID: NumID = 100;

#[derive(StructOpt, Debug)]
#[structopt(name = "testbed")]
struct Opt {
    #[structopt(short, long)]
    template: PathBuf,
    #[structopt(short, long)]
    asset: PathBuf,
    #[structopt(short, long)]
    save: Option<PathBuf>,
    #[structopt(short, long, default_value = "./config.json")]
    config: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    pub zone: ParamZone,
    pub players: Vec<ParamPlayer>,
}

impl Config {
    fn from_path<P: AsRef<Path>>(config_path: P) -> Result<Config> {
        let buf = fs::read_to_string(config_path).unwrap();
        Ok(serde_json::from_str(&buf)?)
    }
}

struct Testbed {
    engine: Box<LogicEngine>,
    state_set: Arc<StateSet>,
    input_handler: InputHandler,
    logic_frame: u32,
    current_secs: f32,
    view_rads: Vec2,
}

impl Testbed {
    fn new(opt: &Opt, cfg: Config) -> Testbed {
        let mut engine = Box::new(LogicEngine::new(&opt.asset).unwrap());
        let save_path = match &opt.save {
            Some(p) => Some(p.join(format!("save_{}", Local::now().format("%Y%m%d_%H%M%S")))),
            None => None,
        };
        let state_set = engine.start_game(cfg.zone, cfg.players, save_path).unwrap();
        Testbed {
            engine,
            state_set,
            input_handler: InputHandler::new(CharacterType::Melee),
            logic_frame: 0,
            current_secs: 0.0,
            view_rads: Vec2::ZERO,
        }
    }
}

impl DebugApp for Testbed {
    fn cpp_physics_system(&mut self) -> *mut u8 {
        unsafe { self.engine.phy_system().unwrap().cpp_physics_system() }
    }

    fn update_frame(
        &mut self,
        delta: f32,
        _camera: &CameraState,
        mouse: &mut DebugMouse,
        keyboard: &mut DebugKeyboard,
    ) -> bool {
        self.input_handler.handle(mouse, keyboard, self.view_rads);

        self.current_secs += delta;
        let next_secs = ((self.logic_frame + 1) as f32) / F_FPS;
        // let next_secs = (self.logic_frame + 1) as f32;
        if (next_secs - self.current_secs).abs() >= 0.5 * FRAC_FPS {
            return true;
        }

        // println!("{}", self.engine.phy_system().unwrap().count_ref());

        self.logic_frame += 1;
        let events = self.input_handler.take_events();
        // println!("frame:{} events:{:?}", self.logic_frame, events);
        let mut state_sets = self
            .engine
            .update_game(vec![InputPlayerEvents::new(100, self.logic_frame, events)])
            .unwrap();
        self.state_set = state_sets.pop().unwrap();
        true
    }

    fn get_initial_camera(&mut self, state: &mut CameraState) {
        state.pos = Vec3A::ZERO;
        state.forward = (3.0 * DEFAULT_VIEW_DIR_3D + Vec3A::new(0.0, -2.0, 0.0)).normalize();
    }

    fn get_camera_pivot(&mut self, heading: f32, pitch: f32) -> Vec3A {
        self.view_rads = Vec2::new(heading, pitch);
        // println!("get_camera_pivot heading:{} {}", -heading, Vec2::new(0.0, -1.0).to_angle());

        if self.state_set.updates.len() == 0 {
            return Vec3A::ZERO;
        }

        let player = self
            .state_set
            .updates
            .iter()
            .find(|x| x.id == PLAYER_ID)
            .unwrap()
            .as_ref()
            .cast::<StatePlayerUpdate>()
            .unwrap();

        // println!("{:?}", player.physics.rotation);

        let fwd = Vec3A::new(pitch.cos() * heading.cos(), pitch.sin(), pitch.cos() * heading.sin());
        let pos = player.physics.position;
        Vec3A::new(pos.x, pos.y + 1.0, pos.z) - 3.0 * fwd
    }
}

fn main() {
    std::env::set_current_dir("/project/points/critical-point/demo").unwrap();

    let opt = Opt::from_args();
    // let opt = Opt {
    //     template: PathBuf::from("/project/points/test-tmp/demo-template"),
    //     asset: PathBuf::from("/project/points/test-asset"),
    //     save: None,
    //     config: PathBuf::from("./config.json"),
    // };
    if !opt.template.is_dir() {
        panic!("template not found: {:?}", opt.template);
    }
    if !opt.asset.is_dir() {
        panic!("asset not found: {:?}", opt.asset);
    }
    LogicEngine::initialize(&opt.template).unwrap();

    let cfg = Config::from_path(&opt.config).unwrap();
    let testbed = Box::new(Testbed::new(&opt, cfg));
    run_debug_application(testbed);
}
