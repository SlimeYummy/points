#![allow(improper_ctypes_definitions)]

use cirtical_point_core::logic::{
    LogicType, StateAny, StateAnyBase, StateCharaPhysics, StateGameInit, StateGameUpdate, StatePlayerInit,
    StatePlayerUpdate, StateType,
};
use cirtical_point_core::utils::s;
use glam::{Quat, Vec3};
use std::sync::Arc;

use super::state_action::{new_state_action_idle, new_state_action_move};

#[no_mangle]
pub extern "C" fn mock_box_dyn_state_any() -> Box<dyn StateAny> {
    Box::new(new_state_player_init())
}

#[no_mangle]
pub extern "C" fn mock_arc_dyn_state_any() -> Arc<dyn StateAny> {
    Arc::new(new_state_player_init())
}

#[no_mangle]
pub extern "C" fn mock_box_state_player_init() -> Box<StatePlayerInit> {
    Box::new(new_state_player_init())
}

#[no_mangle]
pub extern "C" fn mock_arc_state_player_init() -> Arc<StatePlayerInit> {
    Arc::new(new_state_player_init())
}

fn new_state_player_init() -> StatePlayerInit {
    StatePlayerInit {
        _base: StateAnyBase {
            id: 123,
            typ: StateType::PlayerInit,
            logic_typ: LogicType::Player,
        },
        skeleton_file: s!("mock_skeleton.ozz"),
        animation_files: vec![
            s!("mock_animation_0.ozz"),
            s!("mock_animation_1.ozz"),
            s!("mock_animation_2.ozz"),
        ],
        view_model: "model.vrm".to_string(),
    }
}

#[no_mangle]
pub extern "C" fn mock_box_state_game_init() -> Box<StateGameInit> {
    Box::new(new_state_game_init())
}

#[no_mangle]
pub extern "C" fn mock_arc_state_game_init() -> Arc<StateGameInit> {
    Arc::new(new_state_game_init())
}

fn new_state_game_init() -> StateGameInit {
    StateGameInit {
        _base: StateAnyBase {
            id: 4455,
            typ: StateType::GameInit,
            logic_typ: LogicType::Game,
        },
    }
}

#[no_mangle]
pub extern "C" fn mock_box_state_game_update() -> Box<StateGameUpdate> {
    Box::new(new_state_game_update())
}

#[no_mangle]
pub extern "C" fn mock_arc_state_game_update() -> Arc<StateGameUpdate> {
    Arc::new(new_state_game_update())
}

fn new_state_game_update() -> StateGameUpdate {
    StateGameUpdate {
        _base: StateAnyBase {
            id: 4477,
            typ: StateType::GameUpdate,
            logic_typ: LogicType::Game,
        },
        frame: 900,
        id_gen_counter: 42,
    }
}

#[no_mangle]
pub extern "C" fn mock_box_state_player_update() -> Box<StatePlayerUpdate> {
    Box::new(new_state_player_update())
}

#[no_mangle]
pub extern "C" fn mock_arc_state_player_update() -> Arc<StatePlayerUpdate> {
    Arc::new(new_state_player_update())
}

fn new_state_player_update() -> StatePlayerUpdate {
    StatePlayerUpdate {
        _base: StateAnyBase {
            id: 321,
            typ: StateType::PlayerUpdate,
            logic_typ: LogicType::Player,
        },
        physics: StateCharaPhysics {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
        },
        actions: vec![Box::new(new_state_action_idle()), Box::new(new_state_action_move())],
    }
}
