#![allow(improper_ctypes_definitions)]

use cirtical_point_core::logic::{
    ActionIdleMode, ActionMoveMode, StateAction, StateActionAnimation, StateActionBase, StateActionIdle,
    StateActionMove, StateActionType,
};
use cirtical_point_core::template::TmplType;
use cirtical_point_core::utils::s;
use std::sync::Arc;

#[no_mangle]
pub extern "C" fn mock_box_dyn_state_action() -> Box<dyn StateAction> {
    Box::new(new_state_action_idle())
}

#[no_mangle]
pub extern "C" fn mock_arc_dyn_state_action() -> Arc<dyn StateAction> {
    Arc::new(new_state_action_idle())
}

#[no_mangle]
pub extern "C" fn mock_box_state_action_idle() -> Box<StateActionIdle> {
    Box::new(new_state_action_idle())
}

#[no_mangle]
pub extern "C" fn mock_arc_state_action_idle() -> Arc<StateActionIdle> {
    Arc::new(new_state_action_idle())
}

pub fn new_state_action_idle() -> StateActionIdle {
    StateActionIdle {
        _base: StateActionBase {
            id: 1234,
            tmpl_id: s!("Mock.ActionIdle"),
            typ: StateActionType::Idle,
            tmpl_typ: TmplType::ActionIdle,
            spawn_frame: 555,
            death_frame: u32::MAX,
            enter_progress: 207,
            is_leaving: false,
            event_idx: 7744,
            derive_level: 50,
            antibreak_level: 100,
            body_ratio: 0.667,
            animations: [
                StateActionAnimation {
                    file: s!("mock_action_idle_1.ozz"),
                    animation_id: 9999,
                    ratio: 0.125,
                    weight: 0.333,
                },
                StateActionAnimation {
                    file: s!("mock_action_idle_2.ozz"),
                    animation_id: 3456,
                    ratio: 0.6,
                    weight: 0.7,
                },
                StateActionAnimation::default(),
                StateActionAnimation::default(),
            ],
        },
        mode: ActionIdleMode::Idle,
        idle_progress: 30,
        ready_progress: 40,
        idle_timer: 0,
        switch_progress: 21,
    }
}

#[no_mangle]
pub extern "C" fn mock_box_state_action_move() -> Box<StateActionMove> {
    Box::new(new_state_action_move())
}

#[no_mangle]
pub extern "C" fn mock_arc_state_action_move() -> Arc<StateActionMove> {
    Arc::new(new_state_action_move())
}

pub fn new_state_action_move() -> StateActionMove {
    StateActionMove {
        _base: StateActionBase {
            id: 23893,
            tmpl_id: s!("Mock.ActionMove"),
            typ: StateActionType::Move,
            tmpl_typ: TmplType::ActionMove,
            spawn_frame: 891,
            death_frame: u32::MAX,
            enter_progress: 342,
            is_leaving: false,
            event_idx: 893,
            derive_level: 40,
            antibreak_level: 40,
            body_ratio: 0.629,
            animations: [
                StateActionAnimation {
                    file: s!("mock_action_move_1.ozz"),
                    animation_id: 888,
                    ratio: 0.371,
                    weight: 0.287,
                },
                StateActionAnimation {
                    file: s!("mock_action_move_2.ozz"),
                    animation_id: 3456,
                    ratio: 0.72,
                    weight: 0.46,
                },
                StateActionAnimation::default(),
                StateActionAnimation::default(),
            ],
        },
        mode: ActionMoveMode::Move,
        switch_progress: 10,
        previous_progress: 20,
        current_progress: 30,
    }
}
