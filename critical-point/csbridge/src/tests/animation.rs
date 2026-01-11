#![allow(improper_ctypes_definitions)]

use critical_point_core::logic::{ActionIdleMode, StateActionAny, StateActionBase, StateActionIdle, StateActionType};
use critical_point_core::template::TmplType;
use critical_point_core::utils::{id, sb};

#[no_mangle]
pub extern "C" fn mock_skeleton_animator_state_actions() -> Vec<Box<dyn StateActionAny>> {
    let mut idle = Box::new(StateActionIdle {
        _base: StateActionBase::new(StateActionType::Idle, TmplType::ActionIdle),
        mode: ActionIdleMode::Idle,
        idle_time: 0.0,
        ready_time: 0.0,
        auto_idle_time: 0.0,
        switch_time: 0.0,
    });
    idle._base.id = 111;
    idle._base.tmpl_id = id!("Action.One.Run");
    idle.animations[0].files = sb!("Girl_Run_Empty.*");
    idle.animations[0].animation_id = 1;
    idle.animations[0].ratio = 0.0;
    idle.animations[0].weight = 1.0;
    vec![idle]
}
