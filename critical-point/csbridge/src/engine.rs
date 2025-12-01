#![allow(improper_ctypes_definitions)]

use chrono::Local;
use critical_point_core::{xerrf, xres};
use libc::c_char;
use log::{error, LevelFilter};
use std::backtrace::Backtrace;
use std::ffi::CStr;
use std::sync::Arc;
use std::{mem, panic, ptr};

use critical_point_core::animation::SkeletonJointMeta;
use critical_point_core::engine::{LogicEngine, LogicEngineStatus};
use critical_point_core::logic::{InputPlayerEvents, StateActionAny, StateAny, StateSet};
use critical_point_core::parameter::{ParamPlayer, ParamZone};
use critical_point_core::utils::{Symbol, XError, XResult};

use crate::skeletal::resource::SKELETAL_RESOURCE;
use crate::utils::{as_slice, Return};

#[no_mangle]
pub extern "C" fn engine_initialize(
    tmpl_path: *const c_char,
    asset_path: *const c_char,
    log_file: *const c_char,
    log_level: u32,
) -> Return<()> {
    check_memory_layout();

    let res: XResult<()> = (|| {
        let log_file = unsafe { CStr::from_ptr(log_file) }.to_str()?;
        init_log(log_file, log_level)?;

        catch_panic();

        log::error!("-------------------- Critical Point --------------------");

        let tmpl_path = unsafe { CStr::from_ptr(tmpl_path) }.to_str()?;
        let asset_path = unsafe { CStr::from_ptr(asset_path) }.to_str()?;
        LogicEngine::initialize(tmpl_path, asset_path)?;

        SKELETAL_RESOURCE.write().unwrap().clear_all();
        Ok(())
    })();
    Return::from_result(res)
}

fn init_log(log_file: &str, log_level: u32) -> XResult<()> {
    let log_level: LevelFilter = match log_level {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    if log_level != LevelFilter::Off {
        let mut dispatch = fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{} {}] {}",
                    Local::now().format("%y-%m-%d %H:%M:%S%.3f"),
                    record.level(),
                    message
                ))
            })
            .level(log_level);

        if log_file.is_empty() {
            dispatch = dispatch.chain(std::io::stdout());
        }
        else {
            dispatch = dispatch.chain(fern::log_file(log_file)?);
        }

        if let Err(e) = dispatch.apply() {
            if e.to_string() != "attempted to set a logger after the logging system was already initialized" {
                return Err(XError::from(e.to_string()));
            }
        }
    }

    Ok(())
}

fn catch_panic() {
    panic::set_hook(Box::new(|info| {
        let mut msg = "";
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            msg = s;
        }
        else if let Some(s) = info.payload().downcast_ref::<String>() {
            msg = s;
        }

        error!(
            "Panic!!!!! {} {:?} {:?}",
            msg,
            info.location(),
            Backtrace::force_capture()
        );
    }));
}

#[no_mangle]
pub extern "C" fn engine_create() -> Return<*mut LogicEngine> {
    let res: XResult<*mut LogicEngine> = (|| {
        let engine = Box::new(LogicEngine::new()?);
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
        let player: ParamPlayer = rmp_serde::from_slice(player_buf).map_err(|e| xerrf!(BadArgument; "{}", e))?;
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
    zone_data: *const u8,
    zone_len: u32,
    players_data: *const u8,
    players_len: u32,
) -> Return<Option<Arc<StateSet>>> {
    let res: XResult<Arc<StateSet>> = (|| {
        let engine = as_engine(engine)?;
        let zone_buf = as_slice(zone_data, zone_len, "engine_start_game() zone data is null")?;
        let players_buf = as_slice(players_data, players_len, "engine_start_game() players data is null")?;
        let zone: ParamZone = rmp_serde::from_slice(zone_buf).map_err(|e| xerrf!(BadArgument; "{}", e))?;
        let players: Vec<ParamPlayer> = rmp_serde::from_slice(players_buf).map_err(|e| xerrf!(BadArgument; "{}", e))?;
        engine.start_game(zone, players, None)
    })();
    assert_eq!(unsafe { mem::transmute::<Option<Arc<StateSet>>, usize>(None) }, 0);
    Return::from_result_with(res.map(|s| Some(s)), None)
}

#[no_mangle]
pub extern "C" fn engine_update_game(
    engine: *mut LogicEngine,
    events_data: *const u8,
    events_len: u32,
) -> Return<Option<Arc<StateSet>>> {
    let res: XResult<Arc<StateSet>> = (|| {
        let engine = as_engine(engine)?;
        let events_buf = as_slice(events_data, events_len, "engine_update_game() events data is null")?;
        let events: Vec<InputPlayerEvents> =
            rmp_serde::from_slice(events_buf).map_err(|e| xerrf!(BadArgument; "{}", e))?;
        engine.update_game(events)
    })();
    Return::from_result_with(res.map(|s| Some(s)), None)
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
        return xres!(BadArgument; "engine=null");
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
    let vlayout: VecLayout<Box<dyn StateActionAny>> = unsafe { mem::transmute_copy(&vec) };
    assert_eq!(vlayout.len, vec.len());
    assert_eq!(vlayout.cap, vec.capacity());
    assert_eq!(vlayout.data, vec.as_ptr());

    let vec = Vec::with_capacity(11);
    let vlayout: VecLayout<Arc<dyn StateActionAny>> = unsafe { mem::transmute_copy(&vec) };
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
