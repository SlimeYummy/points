use libc::c_char;
use std::cell::RefCell;
use std::ffi::CString;
use std::ptr;

use cirtical_point_core::utils::{XError, XResult};

thread_local! {
    static ERR_MSG: RefCell<CString> = RefCell::new(CString::new("").unwrap());
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Return<T> {
    pub value: T,
    pub err_msg: *const c_char,
}

impl<T: Default> Default for Return<T> {
    fn default() -> Self {
        Return {
            value: Default::default(),
            err_msg: ptr::null(),
        }
    }
}

impl<T> Return<T> {
    pub fn from_result(res: XResult<T>) -> Return<T>
    where
        T: Default,
    {
        Return::from_result_with(res, Default::default())
    }

    pub fn from_result_with(res: XResult<T>, def_value: T) -> Return<T> {
        match res {
            Ok(v) => Return {
                value: v,
                err_msg: ptr::null(),
            },
            Err(e) => {
                let mut ret = Return {
                    value: def_value,
                    err_msg: ptr::null(),
                };
                ERR_MSG.with(|msg| {
                    *msg.borrow_mut() = CString::new(e.to_string()).unwrap();
                    ret.err_msg = msg.borrow().as_ptr();
                });
                ret
            }
        }
    }
}

pub fn as_slice<'t, T>(ptr: *const T, len: u32, err_msg: &'static str) -> XResult<&'t [T]> {
    if ptr.is_null() {
        Err(XError::bad_argument(err_msg))
    } else {
        Ok(unsafe { std::slice::from_raw_parts(ptr, len as usize) })
    }
}
