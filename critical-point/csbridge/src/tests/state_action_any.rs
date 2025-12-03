#![allow(improper_ctypes_definitions)]

use critical_point_core::logic::{
    ActionIdleMode, ActionMoveMode, LogicActionStatus, StateActionAnimation, StateActionAny, StateActionBase,
    StateActionGeneral, StateActionIdle, StateActionMove, StateActionType, StateMultiRootMotion, StateRootMotion,
};
use critical_point_core::template::TmplType;
use critical_point_core::utils::{id, sb};
use glam::{Quat, Vec3};
use glam_ext::Vec2xz;
use std::sync::Arc;

#[no_mangle]
pub extern "C" fn mock_box_dyn_state_action_any() -> Box<dyn StateActionAny> {
    Box::new(new_state_action_idle())
}

#[no_mangle]
pub extern "C" fn mock_arc_dyn_state_action_any() -> Arc<dyn StateActionAny> {
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
            tmpl_id: id!("Action.One.Idle"),
            typ: StateActionType::Idle,
            tmpl_typ: TmplType::ActionIdle,
            status: LogicActionStatus::Activing,
            first_frame: 555,
            last_frame: u32::MAX,
            fade_in_weight: 0.207,
            derive_level: 50,
            poise_level: 100,
            animations: [
                StateActionAnimation {
                    files: sb!("mock_action_idle_1"),
                    animation_id: 9999,
                    ratio: 0.125,
                    weight: 0.333,
                },
                StateActionAnimation {
                    files: sb!("mock_action_idle_2"),
                    animation_id: 3456,
                    ratio: 0.6,
                    weight: 0.7,
                },
                StateActionAnimation::default(),
                StateActionAnimation::default(),
            ],
        },
        mode: ActionIdleMode::Idle,
        idle_time: 3.3,
        ready_time: 4.4,
        auto_idle_time: 1.5,
        switch_time: 0.5,
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
            tmpl_id: id!("Action.One.Run"),
            typ: StateActionType::Move,
            tmpl_typ: TmplType::ActionMove,
            status: LogicActionStatus::Stopping,
            first_frame: 891,
            last_frame: u32::MAX,
            fade_in_weight: 0.342,
            derive_level: 40,
            poise_level: 40,
            animations: [
                StateActionAnimation {
                    files: sb!("mock_action_move_1"),
                    animation_id: 888,
                    ratio: 0.371,
                    weight: 0.287,
                },
                StateActionAnimation {
                    files: sb!("mock_action_move_2"),
                    animation_id: 3456,
                    ratio: 0.72,
                    weight: 0.46,
                },
                StateActionAnimation::default(),
                StateActionAnimation::default(),
            ],
        },
        mode: ActionMoveMode::Move,
        smooth_move_switch: false,
        current_time: 1.5,
        start_anim_idx: 1,
        turn_anim_idx: 2,
        stop_anim_idx: 3,
        root_motion: StateMultiRootMotion {
            local_id: 0,
            ratio: 0.0,
            position: Vec3::new(-5.0, -4.0, -3.0),
            position_delta: Vec3::ONE,
            rotation_cursor: (-Quat::IDENTITY).into(),
            rotation: Quat::IDENTITY.into(),
            rotation_delta: Quat::IDENTITY.into(),
        },
        start_turn_angle_step: Vec2xz::NEG_Z,
        smooth_move_start_speed: 0.5,
        local_fade_in_weight: 1.0,
        anim_offset_time: 0.57,
    }
}

#[no_mangle]
pub extern "C" fn mock_box_state_action_general() -> Box<StateActionGeneral> {
    Box::new(new_state_action_general())
}

#[no_mangle]
pub extern "C" fn mock_arc_state_action_general() -> Arc<StateActionGeneral> {
    Arc::new(new_state_action_general())
}

pub fn new_state_action_general() -> StateActionGeneral {
    StateActionGeneral {
        _base: StateActionBase {
            id: 23893,
            tmpl_id: id!("Action.One.Attack/1"),
            typ: StateActionType::Move,
            tmpl_typ: TmplType::ActionMove,
            status: LogicActionStatus::Activing,
            first_frame: 891,
            last_frame: u32::MAX,
            fade_in_weight: 0.342,
            derive_level: 40,
            poise_level: 40,
            animations: [
                StateActionAnimation {
                    files: sb!("mock_action_move_1"),
                    animation_id: 888,
                    ratio: 0.371,
                    weight: 0.287,
                },
                StateActionAnimation {
                    files: sb!("mock_action_move_2"),
                    animation_id: 3456,
                    ratio: 0.72,
                    weight: 0.46,
                },
                StateActionAnimation::default(),
                StateActionAnimation::default(),
            ],
        },
        current_time: 0.98,
        root_motion: StateRootMotion {
            ratio: 0.9,
            position: Vec3::new(1.0, 2.0, 3.0),
            position_delta: Vec3::new(4.0, 5.0, 6.0),
            rotation_cursor: Quat::IDENTITY.into(),
            rotation: Quat::IDENTITY.into(),
            rotation_delta: (-Quat::IDENTITY).into(),
        },
    }
}
