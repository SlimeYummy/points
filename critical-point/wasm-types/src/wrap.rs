use anyhow::Result;

use crate::auto_gen::*;
use crate::error::HostError;
use crate::host_buffer::HostBuffer;

pub trait PackReturn<A> {
    fn pack(self) -> u64;
}

const _: () = {
    impl PackReturn<(usize, usize)> for (usize, usize) {
        #[inline]
        fn pack(self) -> u64 {
            (self.0 as u64) << 32 | (self.1 as u64)
        }
    }

    impl PackReturn<(usize, u32)> for (usize, u32) {
        #[inline]
        fn pack(self) -> u64 {
            (self.0 as u64) << 32 | (self.1 as u64)
        }
    }

    impl<T> PackReturn<(usize, *const T)> for (usize, *const T) {
        #[inline]
        fn pack(self) -> u64 {
            (self.0 as u64) << 32 | (self.1 as u64)
        }
    }

    impl<T> PackReturn<(usize, *mut T)> for (usize, *mut T) {
        #[inline]
        fn pack(self) -> u64 {
            (self.0 as u64) << 32 | (self.1 as u64)
        }
    }

    impl<T> PackReturn<(*const T, *const T)> for (*const T, *const T) {
        #[inline]
        fn pack(self) -> u64 {
            (self.0 as u64) << 32 | (self.1 as u64)
        }
    }

    impl<T> PackReturn<(*const T, *mut T)> for (*const T, *mut T) {
        #[inline]
        fn pack(self) -> u64 {
            (self.0 as u64) << 32 | (self.1 as u64)
        }
    }

    impl PackReturn<(u32, u32)> for (u32, u32) {
        #[inline]
        fn pack(self) -> u64 {
            (self.0 as u64) << 32 | (self.1 as u64)
        }
    }
};

#[inline(always)]
pub fn wrap_ai_brain_execute<F>(
    global_ptr: *const WsGameGlobal,
    chara_ctrl_ptr: *const WsCharaControl,
    chara_phy_ptr: *const WsCharaPhysics,
    chara_val_ptr: *const WsCharaValue,
    tgt_phy_ptr: *const WsCharaPhysics,
    tgt_val_ptr: *const WsCharaValue,
    ai_do_list_ptr: *mut WsAiDo,
    ai_do_list_len: u32,
    f: F,
) -> u64
where
    F: FnOnce(
        &WsGameGlobal,
        &WsCharaControl,
        &WsCharaPhysics,
        &WsCharaValue,
        Option<&WsCharaPhysics>,
        Option<&WsCharaValue>,
        &mut HostBuffer<WsAiDo>,
    ) -> Result<()>,
{
    let global = unsafe { &*(global_ptr as *const WsGameGlobal) };
    let chara_ctrl = unsafe { &*(chara_ctrl_ptr as *const WsCharaControl) };
    let chara_phy = unsafe { &*(chara_phy_ptr as *const WsCharaPhysics) };
    let chara_val = unsafe { &*(chara_val_ptr as *const WsCharaValue) };
    let tgt_phy = if tgt_phy_ptr.is_null() {
        None
    }
    else {
        Some(unsafe { &*tgt_phy_ptr })
    };
    let tgt_val = if tgt_val_ptr.is_null() {
        None
    }
    else {
        Some(unsafe { &*tgt_val_ptr })
    };
    let mut ai_tasks = unsafe { HostBuffer::new(ai_do_list_ptr, ai_do_list_len) };

    match f(
        global,
        chara_ctrl,
        chara_phy,
        chara_val,
        tgt_phy,
        tgt_val,
        &mut ai_tasks,
    ) {
        Ok(()) => (0u32, ai_tasks.len() as u32).pack(),
        Err(err) => (HostError::write_error(err), 0u32).pack(),
    }
}

#[inline(always)]
pub fn wrap_ai_routine_if<F>(
    global_ptr: *const WsGameGlobal,
    chara_ctrl_ptr: *const WsCharaControl,
    chara_phy_ptr: *const WsCharaPhysics,
    chara_val_ptr: *const WsCharaValue,
    tgt_phy_ptr: *const WsCharaPhysics,
    tgt_val_ptr: *const WsCharaValue,
    f: F,
) -> u64
where
    F: FnOnce(
        &WsGameGlobal,
        &WsCharaControl,
        &WsCharaPhysics,
        &WsCharaValue,
        Option<&WsCharaPhysics>,
        Option<&WsCharaValue>,
    ) -> Result<bool>,
{
    let global = unsafe { &*(global_ptr as *const WsGameGlobal) };
    let chara_ctrl = unsafe { &*(chara_ctrl_ptr as *const WsCharaControl) };
    let chara_phy = unsafe { &*(chara_phy_ptr as *const WsCharaPhysics) };
    let chara_val = unsafe { &*(chara_val_ptr as *const WsCharaValue) };
    let tgt_phy = if tgt_phy_ptr.is_null() {
        None
    }
    else {
        Some(unsafe { &*tgt_phy_ptr })
    };
    let tgt_val = if tgt_val_ptr.is_null() {
        None
    }
    else {
        Some(unsafe { &*tgt_val_ptr })
    };

    match f(global, chara_ctrl, chara_phy, chara_val, tgt_phy, tgt_val) {
        Ok(res) => (0u32, if res { 1u32 } else { 0u32 }).pack(),
        Err(err) => (HostError::write_error(err), 0u32).pack(),
    }
}
