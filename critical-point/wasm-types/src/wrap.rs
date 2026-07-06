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
    chara_value_ptr: *const WsCharaValue,
    ai_tasks_ptr: *mut WsAiTask,
    ai_tasks_len: u32,
    f: F,
) -> u64
where
    F: FnOnce(&WsGameGlobal, &WsCharaValue, &mut HostBuffer<WsAiTask>) -> Result<()>,
{
    let global = unsafe { &*(global_ptr as *const WsGameGlobal) };
    let chara_value = unsafe { &*(chara_value_ptr as *const WsCharaValue) };
    let mut ai_tasks = unsafe { HostBuffer::new(ai_tasks_ptr, ai_tasks_len) };

    match f(global, chara_value, &mut ai_tasks) {
        Ok(()) => (0u32, ai_tasks.len() as u32).pack(),
        Err(err) => (HostError::write_error(err), 0u32).pack(),
    }
}
