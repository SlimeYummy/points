#![allow(improper_ctypes_definitions)]

use cirtical_point_core::xerror;
use libc::c_char;
use std::ffi::CStr;
use std::sync::Arc;
use std::{mem, ptr};

use cirtical_point_core::animation::SkeletonJointMeta;
use cirtical_point_core::engine::{LogicEngine, LogicEngineStatus};
use cirtical_point_core::logic::{InputPlayerEvents, StateAction, StateAny, StateSet};
use cirtical_point_core::parameter::{ParamPlayer, ParamStage};
use cirtical_point_core::utils::{Symbol, XResult};

use crate::utils::{as_slice, Return};

#[no_mangle]
pub extern "C" fn engine_create(tmpl_path: *const c_char, asset_path: *const c_char) -> Return<*mut LogicEngine> {
    check_memory_layout();

    let res: XResult<*mut LogicEngine> = (|| {
        let tmpl_path = unsafe { CStr::from_ptr(tmpl_path) }.to_str()?;
        let asset_path = unsafe { CStr::from_ptr(asset_path) }.to_str()?;
        let engine = Box::new(LogicEngine::new(tmpl_path, asset_path)?);
        Ok(Box::into_raw(engine))
    })();
    Return::from_result_with(res, ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn engine_destroy(engine: *mut LogicEngine) {
    if !engine.is_null() {
        unsafe { drop(Box::from_raw(engine)) };
    }
}

#[no_mangle]
pub extern "C" fn engine_verify_player(
    engine: *mut LogicEngine,
    player_data: *const u8,
    player_len: u32,
) -> Return<()> {
    let res: XResult<()> = (|| {
        let engine = as_engine(engine)?;
        let player_buf = as_slice(player_data, player_len, "engine_verify_player() player data is null")?;
        let player: ParamPlayer = rmp_serde::from_slice(player_buf).map_err(|e| xerror!(BadArgument, e))?;
        engine.verify_player(&player)
    })();
    Return::from_result(res)
}

// TODO: assemble_player

#[no_mangle]
pub extern "C" fn engine_get_game_status(engine: *mut LogicEngine) -> Return<LogicEngineStatus> {
    let res: XResult<LogicEngineStatus> = (|| {
        let engine = as_engine(engine)?;
        Ok(LogicEngineStatus {
            is_game_running: engine.is_game_running(),
            current_frame: engine.current_frame(),
            next_frame: engine.next_frame(),
        })
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn engine_start_game(
    engine: *mut LogicEngine,
    stage_data: *const u8,
    stage_len: u32,
    players_data: *const u8,
    players_len: u32,
) -> Return<Option<Arc<StateSet>>> {
    let res: XResult<Arc<StateSet>> = (|| {
        let engine = as_engine(engine)?;
        let stage_buf = as_slice(stage_data, stage_len, "engine_start_game() stage data is null")?;
        let players_buf = as_slice(players_data, players_len, "engine_start_game() players data is null")?;
        let stage: ParamStage = rmp_serde::from_slice(stage_buf).map_err(|e| xerror!(BadArgument, e))?;
        let players: Vec<ParamPlayer> = rmp_serde::from_slice(players_buf).map_err(|e| xerror!(BadArgument, e))?;
        engine.start_game(stage, players, None)
    })();
    assert_eq!(unsafe { mem::transmute::<Option<Arc<StateSet>>, usize>(None) }, 0);
    Return::from_result_with(res.map(|s| Some(s)), None)
}

#[no_mangle]
pub extern "C" fn engine_update_game(
    engine: *mut LogicEngine,
    events_data: *const u8,
    events_len: u32,
) -> Return<Vec<Arc<StateSet>>> {
    let res: XResult<Vec<Arc<StateSet>>> = (|| {
        let engine = as_engine(engine)?;
        let events_buf = as_slice(events_data, events_len, "engine_update_game() events data is null")?;
        let events: Vec<InputPlayerEvents> = rmp_serde::from_slice(events_buf).map_err(|e| xerror!(BadArgument, e))?;
        engine.update_game(events)
    })();
    Return::from_result(res)
}

#[no_mangle]
pub extern "C" fn vec_arc_state_set_drop(vec: Vec<Arc<StateSet>>) {
    mem::drop(vec);
}

#[no_mangle]
pub extern "C" fn engine_stop_game(engine: *mut LogicEngine) -> Return<()> {
    let res: XResult<()> = (|| {
        let engine = as_engine(engine)?;
        engine.stop_game()
    })();
    Return::from_result(res)
}

fn as_engine<'t>(engine: *mut LogicEngine) -> XResult<&'t mut LogicEngine> {
    if engine.is_null() {
        return Err(xerror!(BadArgument, "engine=null"));
    }
    Ok(unsafe { &mut *engine })
}

fn check_memory_layout() {
    #[repr(C)]
    #[derive(Debug)]
    struct VecLayout<T> {
        cap: usize,
        data: *const T,
        len: usize,
    }

    let vec = Vec::with_capacity(5);
    let vlayout: VecLayout<u8> = unsafe { mem::transmute_copy(&vec) };
    assert_eq!(vlayout.len, vec.len());
    assert_eq!(vlayout.cap, vec.capacity());
    assert_eq!(vlayout.data, vec.as_ptr());

    let vec = Vec::with_capacity(5);
    let vlayout: VecLayout<u16> = unsafe { mem::transmute_copy(&vec) };
    assert_eq!(vlayout.len, vec.len());
    assert_eq!(vlayout.cap, vec.capacity());
    assert_eq!(vlayout.data, vec.as_ptr());

    let vec = Vec::with_capacity(5);
    let vlayout: VecLayout<u32> = unsafe { mem::transmute_copy(&vec) };
    assert_eq!(vlayout.len, vec.len());
    assert_eq!(vlayout.cap, vec.capacity());
    assert_eq!(vlayout.data, vec.as_ptr());

    let vec = Vec::with_capacity(5);
    let vlayout: VecLayout<usize> = unsafe { mem::transmute_copy(&vec) };
    assert_eq!(vlayout.len, vec.len());
    assert_eq!(vlayout.cap, vec.capacity());
    assert_eq!(vlayout.data, vec.as_ptr());

    let vec = Vec::with_capacity(7);
    let vlayout: VecLayout<f32> = unsafe { mem::transmute_copy(&vec) };
    assert_eq!(vlayout.len, vec.len());
    assert_eq!(vlayout.cap, vec.capacity());
    assert_eq!(vlayout.data, vec.as_ptr());

    let vec = Vec::with_capacity(7);
    let vlayout: VecLayout<f64> = unsafe { mem::transmute_copy(&vec) };
    assert_eq!(vlayout.len, vec.len());
    assert_eq!(vlayout.cap, vec.capacity());
    assert_eq!(vlayout.data, vec.as_ptr());

    let vec = Vec::with_capacity(7);
    let vlayout: VecLayout<Symbol> = unsafe { mem::transmute_copy(&vec) };
    assert_eq!(vlayout.len, vec.len());
    assert_eq!(vlayout.cap, vec.capacity());
    assert_eq!(vlayout.data, vec.as_ptr());

    let vec = Vec::with_capacity(10);
    let vlayout: VecLayout<Box<dyn StateAny>> = unsafe { mem::transmute_copy(&vec) };
    assert_eq!(vlayout.len, vec.len());
    assert_eq!(vlayout.cap, vec.capacity());
    assert_eq!(vlayout.data, vec.as_ptr());

    let vec = Vec::with_capacity(10);
    let vlayout: VecLayout<Arc<dyn StateAny>> = unsafe { mem::transmute_copy(&vec) };
    assert_eq!(vlayout.len, vec.len());
    assert_eq!(vlayout.cap, vec.capacity());
    assert_eq!(vlayout.data, vec.as_ptr());

    let vec = Vec::with_capacity(11);
    let vlayout: VecLayout<Box<dyn StateAction>> = unsafe { mem::transmute_copy(&vec) };
    assert_eq!(vlayout.len, vec.len());
    assert_eq!(vlayout.cap, vec.capacity());
    assert_eq!(vlayout.data, vec.as_ptr());

    let vec = Vec::with_capacity(11);
    let vlayout: VecLayout<Arc<dyn StateAction>> = unsafe { mem::transmute_copy(&vec) };
    assert_eq!(vlayout.len, vec.len());
    assert_eq!(vlayout.cap, vec.capacity());
    assert_eq!(vlayout.data, vec.as_ptr());

    let vec = Vec::with_capacity(12);
    let vlayout: VecLayout<SkeletonJointMeta> = unsafe { mem::transmute_copy(&vec) };
    assert_eq!(vlayout.len, vec.len());
    assert_eq!(vlayout.cap, vec.capacity());
    assert_eq!(vlayout.data, vec.as_ptr());

    let vec = Vec::with_capacity(13);
    let vlayout: VecLayout<Arc<StateSet>> = unsafe { mem::transmute_copy(&vec) };
    assert_eq!(vlayout.len, vec.len());
    assert_eq!(vlayout.cap, vec.capacity());
    assert_eq!(vlayout.data, vec.as_ptr());

    // slice

    #[repr(C)]
    struct SliceLayout {
        data: *const i32,
        len: usize,
    }
    let slice: &[i32] = &[123, 456, 789];
    let slayout: SliceLayout = unsafe { mem::transmute_copy(&slice) };
    assert_eq!(slayout.data, slice.as_ptr());
    assert_eq!(slayout.len, slice.len());

    // string

    #[repr(C)]
    struct StringLayout {
        len: usize,
        data: *const u8,
        cap: usize,
    }
    let s = "hello".to_string();
    let slayout: StringLayout = unsafe { mem::transmute_copy(&s) };
    assert_eq!(slayout.len, s.len());
    assert_eq!(slayout.cap, s.capacity());
    assert_eq!(slayout.data, s.as_ptr());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_memory_layout() {
        check_memory_layout();
    }
}
