use libc::c_char;
use std::ffi::{CStr, CString};
use std::ptr;

use cirtical_point_core::utils::{xerrf, Symbol, TmplID, XResult};

use crate::utils::Return;

#[no_mangle]
pub extern "C" fn tmpl_id_create(cstr: *const c_char) -> Return<u64> {
    let res: XResult<TmplID> = (|| {
        let mut string = "";
        if !cstr.is_null() {
            string = unsafe { CStr::from_ptr(cstr) }.to_str()?
        };
        TmplID::new(string)
    })();
    Return::from_result_with(res.map(|v| v.to_u64()), TmplID::INVALID.to_u64())
}

#[no_mangle]
pub extern "C" fn tmpl_id_is_valid(cid: u64) -> bool {
    if cid == TmplID::INVALID.to_u64() {
        return false;
    }
    TmplID::try_from(cid).is_ok()
}

#[no_mangle]
pub extern "C" fn tmpl_id_to_string(cid: u64) -> Return<*mut c_char> {
    let res: XResult<*mut c_char> = (|| {
        let id = TmplID::try_from(cid)?;
        let string = CString::new(id.to_string()).map_err(|e| xerrf!(Unexpected; "{}", e.to_string()))?;
        Ok(string.into_raw())
    })();
    Return::from_result_with(res, ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn tmpl_id_free_string(cstr: *mut c_char) {
    if !cstr.is_null() {
        unsafe { drop(CString::from_raw(cstr)) };
    }
}

#[no_mangle]
pub extern "C" fn symbol_create(cstr: *const c_char) -> usize {
    let str = unsafe { CStr::from_ptr(cstr) }.to_str().unwrap();
    let symbol = Symbol::new(str).unwrap();
    return unsafe { std::mem::transmute::<Symbol, usize>(symbol) };
}
