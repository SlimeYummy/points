#![allow(improper_ctypes_definitions)]

use cirtical_point_core::logic::{ActionIdleMode, StateAction, StateActionBase, StateActionIdle, StateActionType};
use cirtical_point_core::template::TmplType;
use cirtical_point_core::utils::asb;

#[no_mangle]
pub extern "C" fn mock_skeleton_animator_state_actions() -> Vec<Box<dyn StateAction>> {
    let mut idle = Box::new(StateActionIdle {
        _base: StateActionBase::new(StateActionType::Idle, TmplType::ActionIdle),
        mode: ActionIdleMode::Idle,
        idle_progress: 0,
        ready_progress: 0,
        idle_timer: 0,
        switch_progress: 0,
    });
    idle._base.id = 111;
    idle._base.tmpl_id = asb!("idle");
    idle.animations[0].file = asb!("view_anim.ozz");
    idle.animations[0].animation_id = 1;
    idle.animations[0].ratio = 0.0;
    idle.animations[0].weight = 1.0;
    vec![idle]
}
