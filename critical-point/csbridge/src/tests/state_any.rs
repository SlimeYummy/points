#![allow(improper_ctypes_definitions)]

use critical_point_core::animation::AnimationFileMeta;
use critical_point_core::logic::{
    LogicType, StateAny, StateBase, StateCharaPhysics, StateGameInit, StateGameUpdate, StatePlayerInit,
    StatePlayerUpdate, StateType,
};
use critical_point_core::utils::{id, sb};
use glam::Vec3A;
use glam_ext::Vec2xz;
use std::sync::Arc;

use super::state_action_any::{new_state_action_idle, new_state_action_move};

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
        _base: StateBase {
            id: 123,
            typ: StateType::PlayerInit,
            logic_typ: LogicType::Player,
        },
        skeleton_file: sb!("mock_skeleton.ozz"),
        animation_metas: vec![
            AnimationFileMeta::new(sb!("mock_animation_0.ozz"), false, false),
            AnimationFileMeta::new(sb!("mock_animation_1.ozz"), false, false),
            AnimationFileMeta::new(sb!("mock_animation_2.ozz"), false, false),
        ],
        view_model: sb!("model.vrm"),
        init_position: Vec3A::new(1.0, 2.0, 3.0),
        init_direction: Vec2xz::Z,
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
        _base: StateBase {
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
        _base: StateBase {
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
        _base: StateBase {
            id: 321,
            typ: StateType::PlayerUpdate,
            logic_typ: LogicType::Player,
        },
        physics: StateCharaPhysics {
            velocity: Vec3A::new(4.0, 5.0, 6.0).into(),
            position: Vec3A::new(1.0, 2.0, 3.0).into(),
            direction: Vec2xz::new(0.0, -1.0),
        },
        actions: vec![Box::new(new_state_action_idle()), Box::new(new_state_action_move())],
        // action_events: vec!["Event0".to_string(), "Event1".to_string(), "Event2".to_string()],
        custom_events: vec![
            (id!("Action.One.Attack^1"), sb!("Event0")).into(),
            (id!("Action.One.Attack^1"), sb!("Event1")).into(),
            (id!("Action.One.Attack^1"), sb!("Event2")).into(),
        ],
    }
}
