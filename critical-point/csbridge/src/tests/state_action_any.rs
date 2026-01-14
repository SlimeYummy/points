#![allow(improper_ctypes_definitions)]

use critical_point_core::animation::RootTrackName;
use critical_point_core::logic::{
    ActionIdleMode, ActionMoveMode, LogicActionStatus, StateActionAnimation, StateActionAny, StateActionBase,
    StateActionGeneral, StateActionIdle, StateActionMove, StateActionType, StateMultiRootMotion, StateRootMotion,
};
use critical_point_core::template::TmplType;
use critical_point_core::utils::{TimeRange, id, sb};
use glam::{Quat, Vec3A};
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
                StateActionAnimation::new(sb!("mock_action_idle_1"), 9999, true, 0.125, 0.333),
                StateActionAnimation::new(sb!("mock_action_idle_2"), 3456, false, 0.6, 0.7),
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
            id: 783,
            tmpl_id: id!("Action.One.Run"),
            typ: StateActionType::Move,
            tmpl_typ: TmplType::ActionMove,
            status: LogicActionStatus::Activing,
            first_frame: 123,
            last_frame: u32::MAX,
            fade_in_weight: 0.77,
            derive_level: 70,
            poise_level: 68,
            animations: [
                StateActionAnimation::new(sb!("mock_action_move_1"), 888, true, 0.02, 0.287),
                StateActionAnimation::new(sb!("mock_action_move_2"), 3456, false, 0.875, 0.46),
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
            pos_track: RootTrackName::Default,
            ratio: 0.0,
            current_pos: Vec3A::new(-5.0, -4.0, -3.0),
            previous_pos: Vec3A::new(-2.0, -1.0, 0.0),
            pos_delta: Vec3A::ONE,
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
            id: 5551,
            tmpl_id: id!("Action.One.Attack^1"),
            typ: StateActionType::General,
            tmpl_typ: TmplType::ActionGeneral,
            status: LogicActionStatus::Activing,
            first_frame: 891,
            last_frame: u32::MAX,
            fade_in_weight: 0.112,
            derive_level: 9,
            poise_level: 13,
            animations: [
                StateActionAnimation::new(sb!("mock_action_gen_1"), 81, true, 0.66, 0.74),
                StateActionAnimation::default(),
                StateActionAnimation::default(),
                StateActionAnimation::default(),
            ],
        },
        current_time: 0.98,
        from_rotation: 1.0,
        to_rotation: 2.0,
        current_rotation: 1.5,
        rotation_time: TimeRange::new(10.0, 20.0),
        root_motion: StateRootMotion {
            pos_track: RootTrackName::Move,
            ratio: 0.9,
            current_pos: Vec3A::new(1.0, 2.0, 3.0),
            previous_pos: Vec3A::new(7.0, 7.0, 7.0),
            pos_delta: Vec3A::new(4.0, 5.0, 6.0),
        },
    }
}
