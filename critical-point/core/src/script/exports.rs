use wasmtime::{Caller, Linker};

use crate::script::engine::ScriptContext;
use crate::utils::{Symbol, TmplID, XError, XResult};

//
// TmplID
//

fn tmpl_id_new(mut caller: Caller<'_, ScriptContext>, str_ptr: u32, str_len: u32, tmpl_id_ptr: u32) -> u32 {
    let ctx = caller.data_mut();
    let id_str = unsafe { ctx.str(str_ptr, str_len) };
    let res = TmplID::new(id_str);
    if let Ok(id) = res {
        unsafe { ctx.write(tmpl_id_ptr, id) };
    }
    ctx.write_result(res)
}

fn tmpl_id_to_string(mut caller: Caller<'_, ScriptContext>, tmpl_id_ptr: u32, buf_ptr: u32, buf_len: u32) -> u32 {
    let ctx = caller.data_mut();
    let id: TmplID = unsafe { *ctx.get(tmpl_id_ptr) };
    let s = id.to_string();
    let s_len = s.len();
    unsafe { ctx.write_slice(buf_ptr, buf_len, s.as_bytes()) };
    s_len as u32
}

//
// Symbol
//

fn symbol_new(mut caller: Caller<'_, ScriptContext>, str_ptr: u32, str_len: u32, symbol_ptr: u32) -> u32 {
    let ctx = caller.data_mut();
    let sym_str = unsafe { ctx.str(str_ptr, str_len) };
    let res = Symbol::new(sym_str);
    if let Ok(sym) = res {
        unsafe { ctx.write(symbol_ptr, sym) };
    }
    ctx.write_result(res)
}

fn symbol_len(caller: Caller<'_, ScriptContext>, symbol_ptr: u32) -> u32 {
    let ctx = caller.data();
    let sym: Symbol = unsafe { *ctx.get(symbol_ptr) };
    sym.len() as u32
}

fn symbol_to_string(mut caller: Caller<'_, ScriptContext>, symbol_ptr: u32, buf_ptr: u32, buf_len: u32) -> u32 {
    let ctx = caller.data_mut();
    let sym: Symbol = unsafe { *ctx.get(symbol_ptr) };
    let s = sym.as_str().to_string();
    let s_len = s.len();
    unsafe { ctx.write_slice(buf_ptr, buf_len, s.as_bytes()) };
    s_len as u32
}

//
// Register functions
//

pub fn register_functions(linker: &mut Linker<ScriptContext>) -> XResult<()> {
    linker
        .func_wrap("host", "tmpl_id_new", tmpl_id_new)
        .map_err(|e| XError::from(e))?;

    linker
        .func_wrap("host", "tmpl_id_to_string", tmpl_id_to_string)
        .map_err(|e| XError::from(e))?;

    linker
        .func_wrap("host", "symbol_new", symbol_new)
        .map_err(|e| XError::from(e))?;

    linker
        .func_wrap("host", "symbol_len", symbol_len)
        .map_err(|e| XError::from(e))?;

    linker
        .func_wrap("host", "symbol_to_string", symbol_to_string)
        .map_err(|e| XError::from(e))?;

    Ok(())
}
