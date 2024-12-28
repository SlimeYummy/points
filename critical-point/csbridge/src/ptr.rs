#![allow(improper_ctypes_definitions)]

use std::ffi::CStr;
use std::{ptr, mem};
use std::sync::Arc;

use cirtical_point_core::animation::SkeletonMeta;
use cirtical_point_core::logic::{
    StateAction, StateActionIdle, StateAny, StateGameInit, StateGameUpdate, StatePlayerInit, StatePlayerUpdate,
    StateStageInit, StateStageUpdate, StateNpcInit, StateNpcUpdate, StateActionMove, StateSet,
};
use cirtical_point_core::utils::{CastPtr, CastRef, Symbol};

#[no_mangle]
pub extern "C" fn new_symbol(cstr: *const i8) -> Symbol {
    let str = unsafe { CStr::from_ptr(cstr) }.to_str().unwrap();
    return Symbol::new(str).unwrap();
}

macro_rules! box_drop {
    ($fn:ident, $ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $fn(bx: Box<$ty>) {
            mem::drop(bx);
        }
    }
}

macro_rules! arc_clone {
    ($fn:ident, $ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $fn(arc: *const Arc<$ty>) -> Arc<$ty> {
            let arc: &Arc<$ty> = unsafe { &*arc };
            arc.clone()
        }
    }
}

macro_rules! arc_drop {
    ($fn:ident, $ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $fn(arc: Arc<$ty>) {
            mem::drop(arc);
        }
    }
}

macro_rules! box_ref {
    ($fn:ident, $base_ty:ty, $ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $fn(base: *const $base_ty) -> *const $ty {
            let base: &$base_ty = unsafe { &*base };
            match base.cast_ref::<$ty>() {
                Ok(v) => v as *const $ty,
                Err(_) => ptr::null(),
            }
        }
    };
}

type ArcInner = ();

macro_rules! arc_ref {
    ($fn:ident, $base_ty:ty, $ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $fn(base: *const $base_ty) -> *const ArcInner {
            let base_ref: &$base_ty = unsafe { &*base };
            match base_ref.cast_ref::<$ty>() {
                Ok(_) => unsafe { *(base as *const *const ArcInner) },
                Err(_) => ptr::null(),
            }
        }
    };
}

macro_rules! arc_arc {
    ($fn:ident, $base_ty:ty, $ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $fn(base: *const $base_ty) -> Option<Arc<$ty>> {
            let base: &$base_ty = unsafe { &*base };
            match base.cast_to::<$ty>() {
                Ok(v) => Some(v),
                Err(_) => None,
            }
        }
    };
}

//
// StateAny
//

box_drop!(dyn_state_any_box_drop, dyn StateAny);
arc_clone!(dyn_state_any_arc_clone, dyn StateAny);
arc_drop!(dyn_state_any_arc_drop, dyn StateAny);

box_drop!(state_game_init_box_drop, StateGameInit);
arc_clone!(state_game_init_arc_clone, StateGameInit);
arc_drop!(state_game_init_arc_drop, StateGameInit);
box_ref!(state_game_init_box_ref, Box<dyn StateAny>, StateGameInit);
arc_ref!(state_game_init_arc_ref, Arc<dyn StateAny>, StateGameInit);
arc_arc!(state_game_init_arc_arc, Arc<dyn StateAny>, StateGameInit);

box_drop!(state_game_update_box_drop, StateGameUpdate);
arc_clone!(state_game_update_arc_clone, StateGameUpdate);
arc_drop!(state_game_update_arc_drop, StateGameUpdate);
box_ref!(state_game_update_box_ref, Box<dyn StateAny>, StateGameUpdate);
arc_ref!(state_game_update_arc_ref, Arc<dyn StateAny>, StateGameUpdate);
arc_arc!(state_game_update_arc_arc, Arc<dyn StateAny>, StateGameUpdate);

box_drop!(state_stage_init_box_drop, StateStageInit);
arc_clone!(state_stage_init_arc_clone, StateStageInit);
arc_drop!(state_stage_init_arc_drop, StateStageInit);
box_ref!(state_stage_init_box_ref, Box<dyn StateAny>, StateStageInit);
arc_ref!(state_stage_init_arc_ref, Arc<dyn StateAny>, StateStageInit);
arc_arc!(state_stage_init_arc_arc, Arc<dyn StateAny>, StateStageInit);

box_drop!(state_stage_update_box_drop, StateStageUpdate);
arc_clone!(state_stage_update_arc_clone, StateStageUpdate);
arc_drop!(state_stage_update_arc_drop, StateStageUpdate);
box_ref!(state_stage_update_box_ref, Box<dyn StateAny>, StateStageUpdate);
arc_ref!(state_stage_update_arc_ref, Arc<dyn StateAny>, StateStageUpdate);
arc_arc!(state_stage_update_arc_arc, Arc<dyn StateAny>, StateStageUpdate);

box_drop!(state_player_init_box_drop, StatePlayerInit);
arc_clone!(state_player_init_arc_clone, StatePlayerInit);
arc_drop!(state_player_init_arc_drop, StatePlayerInit);
box_ref!(state_player_init_box_ref, Box<dyn StateAny>, StatePlayerInit);
arc_ref!(state_player_init_arc_ref, Arc<dyn StateAny>, StatePlayerInit);
arc_arc!(state_player_init_arc_arc, Arc<dyn StateAny>, StatePlayerInit);

box_drop!(state_player_update_box_drop, StatePlayerUpdate);
arc_clone!(state_player_update_arc_clone, StatePlayerUpdate);
arc_drop!(state_player_update_arc_drop, StatePlayerUpdate);
box_ref!(state_player_update_box_ref, Box<dyn StateAny>, StatePlayerUpdate);
arc_ref!(state_player_update_arc_ref, Arc<dyn StateAny>, StatePlayerUpdate);
arc_arc!(state_player_update_arc_arc, Arc<dyn StateAny>, StatePlayerUpdate);

box_drop!(state_npc_init_box_drop, StateNpcInit);
arc_clone!(state_npc_init_arc_clone, StateNpcInit);
arc_drop!(state_npc_init_arc_drop, StateNpcInit);
box_ref!(state_npc_init_box_ref, Box<dyn StateAny>, StateNpcInit);
arc_ref!(state_npc_init_arc_ref, Arc<dyn StateAny>, StateNpcInit);
arc_arc!(state_npc_init_arc_arc, Arc<dyn StateAny>, StateNpcInit);

box_drop!(state_npc_update_box_drop, StateNpcUpdate);
arc_clone!(state_npc_update_arc_clone, StateNpcUpdate);
arc_drop!(state_npc_update_arc_drop, StateNpcUpdate);
box_ref!(state_npc_update_box_ref, Box<dyn StateAny>, StateNpcUpdate);
arc_ref!(state_npc_update_arc_ref, Arc<dyn StateAny>, StateNpcUpdate);
arc_arc!(state_npc_update_arc_arc, Arc<dyn StateAny>, StateNpcUpdate);

//
// StateAction
//

box_drop!(dyn_state_action_box_drop, dyn StateAction);
arc_clone!(dyn_state_action_arc_clone, dyn StateAction);
arc_drop!(dyn_state_action_arc_drop, dyn StateAction);

box_drop!(state_action_idle_box_drop, StateActionIdle);
arc_clone!(state_action_idle_arc_clone, StateActionIdle);
arc_drop!(state_action_idle_arc_drop, StateActionIdle);
box_ref!(state_action_idle_box_ref, Box<dyn StateAction>, StateActionIdle);
arc_ref!(state_action_idle_arc_ref, Arc<dyn StateAction>, StateActionIdle);
arc_arc!(state_action_idle_arc_arc, Arc<dyn StateAction>, StateActionIdle);

box_drop!(state_action_move_box_drop, StateActionMove);
arc_clone!(state_action_move_arc_clone, StateActionMove);
arc_drop!(state_action_move_arc_drop, StateActionMove);
box_ref!(state_action_move_box_ref, Box<dyn StateAction>, StateActionMove);
arc_ref!(state_action_move_arc_ref, Arc<dyn StateAction>, StateActionMove);
arc_arc!(state_action_move_arc_arc, Arc<dyn StateAction>, StateActionMove);

//
// Others
//

box_drop!(state_set_box_drop, StateSet);
arc_clone!(state_set_arc_clone, StateSet);
arc_drop!(state_set_arc_drop, StateSet);

box_drop!(skeleton_meta_box_drop, SkeletonMeta);
arc_clone!(skeleton_meta_arc_clone, SkeletonMeta);
arc_drop!(skeleton_meta_arc_drop, SkeletonMeta);
