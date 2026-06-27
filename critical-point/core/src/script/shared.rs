use std::rc::Rc;
use std::sync::Arc;
use talc::TalcCell;

use crate::script::memory::TalcSource;

/// WASM shared Box
pub type WsBox<T> = Box<T, Rc<TalcCell<TalcSource>>>;

/// WASM shared Vec
pub type WsVec<T> = Vec<T, Rc<TalcCell<TalcSource>>>;

/// WASM shared Rc
pub type WsRc<T> = Rc<T, Rc<TalcCell<TalcSource>>>;

/// WASM shared Arc
pub type WsArc<T> = Arc<T, Rc<TalcCell<TalcSource>>>;

pub trait WsShared<T> {
    fn as_rust_ptr(&self) -> *const T;

    #[inline]
    fn to_wasm_addr(&self, base_ptr: usize) -> u32 {
        (self.as_rust_ptr() as usize - base_ptr) as u32
    }
}

impl<T> WsShared<T> for WsBox<T> {
    #[inline]
    fn as_rust_ptr(&self) -> *const T {
        self.as_ref() as *const T
    }
}

impl<T> WsShared<T> for WsVec<T> {
    #[inline]
    fn as_rust_ptr(&self) -> *const T {
        self.as_ptr()
    }
}

impl<T> WsShared<T> for WsRc<T> {
    #[inline]
    fn as_rust_ptr(&self) -> *const T {
        Rc::as_ptr(self)
    }
}

impl<T> WsShared<T> for WsArc<T> {
    #[inline]
    fn as_rust_ptr(&self) -> *const T {
        Arc::as_ptr(self)
    }
}
