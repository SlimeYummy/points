#![allow(improper_ctypes_definitions)]

use critical_point_core::logic::{
    ActionIdleMode, StateActionAnimation, StateActionAny, StateActionBase, StateActionIdle,
};
use critical_point_core::utils::{id, sb, ActionType};

#[no_mangle]
pub extern "C" fn mock_skeleton_animator_state_actions() -> Vec<Box<dyn StateActionAny>> {
    let mut idle = Box::new(StateActionIdle {
        _base: StateActionBase::new(ActionType::Idle),
        mode: ActionIdleMode::Idle,
        idle_time: 0.0,
        ready_time: 0.0,
        auto_idle_time: 0.0,
        switch_time: 0.0,
    });
    idle._base.id = 111;
    idle._base.tmpl_id = id!("Action.One.Run");
    idle.animations
        .push(StateActionAnimation::new(sb!("Girl_Run_Empty.*"), 1, false, 0.0, 1.0));
    vec![idle]
}
