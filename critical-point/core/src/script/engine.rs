use log::info;
use std::io::{Cursor, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::{fs, ptr, slice, str};
use talc::{self, TalcCell};
use wasmtime::{Config, Engine, Func, Instance, Linker, Module, Store, TypedFunc, WasmParams, WasmResults};

use crate::consts::{KB, MB};
use crate::script::exports::register_functions;
use crate::script::memory::{TalcSource, new_allocators};
use crate::script::shared::WsShared;
use crate::utils::{XError, XResult, xerr};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct ScriptEngineConfig {
    pub max_size: usize,
    pub stack_size: usize,
    pub host_size: usize,
    pub host_grow_size: usize,
}

impl Default for ScriptEngineConfig {
    #[inline]
    fn default() -> Self {
        Self {
            max_size: 64 * MB,
            stack_size: 512 * KB,
            host_size: 32 * MB - 512 * KB,
            host_grow_size: 256 * KB,
        }
    }
}

#[derive(Debug)]
pub struct ScriptContext {
    base_ptr: usize,
    error_buffer_ptr: u32,
    error_buffer_len: u32,
}

impl ScriptContext {
    #[inline]
    pub fn unpack(&self, v: u64) -> (u32, u32) {
        ((v >> 32) as u32, (v & 0xFFFFFFFF) as u32)
    }

    #[inline]
    pub(crate) unsafe fn get<'t, T: Copy>(&'t self, addr: u32) -> &'t T {
        let ptr = self.base_ptr + addr as usize;
        unsafe { &*(ptr as *const T) }
    }

    #[inline]
    pub(crate) unsafe fn get_mut<'t, T: Copy>(&'t self, addr: u32) -> &'t mut T {
        let ptr = self.base_ptr + addr as usize;
        unsafe { &mut *(ptr as *mut T) }
    }

    #[inline]
    pub(crate) unsafe fn write<T: Copy>(&mut self, addr: u32, value: T) {
        let ptr = self.base_ptr + addr as usize;
        unsafe { ptr::write(ptr as *mut T, value) }
    }

    #[inline]
    pub(crate) unsafe fn slice<'t, T: Copy>(&'t self, addr: u32, len: u32) -> &'t [T] {
        let ptr = self.base_ptr + addr as usize;
        unsafe { slice::from_raw_parts(ptr as *const T, len as usize) }
    }

    #[inline]
    pub(crate) unsafe fn slice_mut<'t, T: Copy>(&'t self, addr: u32, len: u32) -> &'t mut [T] {
        let ptr = self.base_ptr + addr as usize;
        unsafe { slice::from_raw_parts_mut(ptr as *mut T, len as usize) }
    }

    #[inline]
    pub(crate) unsafe fn write_slice<T: Copy>(&mut self, addr: u32, len: u32, slice: &[T]) {
        let ptr = self.base_ptr + addr as usize;
        let len = usize::min(len as usize, slice.len());
        unsafe { ptr::copy_nonoverlapping(slice.as_ptr(), ptr as *mut T, len) }
    }

    #[inline]
    pub(crate) unsafe fn str<'t>(&'t self, addr: u32, len: u32) -> &'t str {
        let buf = unsafe { self.slice(addr, len) };
        unsafe { str::from_utf8_unchecked(buf) }
    }

    pub fn read_error_string(&self, len: u32) -> Option<String> {
        if len == 0 {
            return None;
        }
        let len = u32::min(len, self.error_buffer_len);
        unsafe { Some(self.str(self.error_buffer_ptr, len).to_string()) }
    }

    pub fn read_error(&self, len: u32) -> Option<XError> {
        match self.read_error_string(len) {
            Some(msg) => Some(xerr!(Script).set_msg(msg)),
            None => None,
        }
    }

    pub fn read_result(&self, len: u32) -> XResult<()> {
        match self.read_error_string(len) {
            Some(msg) => Err(xerr!(Script).set_msg(msg)),
            None => Ok(()),
        }
    }

    pub fn write_error(&mut self, err: XError) -> u32 {
        let buf = unsafe { self.slice_mut(self.error_buffer_ptr, self.error_buffer_len) };
        let mut cursor = Cursor::new(buf);
        let msg = err.to_string();
        let _ = cursor.write_all(msg.as_bytes());
        cursor.position() as u32
    }

    #[inline]
    pub fn write_result<T>(&mut self, res: XResult<T>) -> u32 {
        match res {
            Ok(_) => 0,
            Err(e) => self.write_error(e),
        }
    }
}

pub struct ScriptEngine {
    engine: Engine,
    module: Module,
    store: Store<ScriptContext>,
    instance: Instance,

    talc: Rc<TalcCell<TalcSource>>,
}

impl ScriptEngine {
    pub fn new<P: AsRef<Path>>(wasm_path: P, config: ScriptEngineConfig) -> XResult<ScriptEngine> {
        info!("ScriptEngine::new() wasm_path={:?}", wasm_path.as_ref());
        let wasm = fs::read(wasm_path)?;

        let (talc, wasm_creator, base_ptr) = new_allocators(
            config.max_size,
            config.stack_size,
            config.host_size,
            config.host_grow_size,
        )?;

        let mut config = Config::new();
        config.with_host_memory(Arc::new(wasm_creator));

        let engine = Engine::new(&config)?;
        let module = Module::new(&engine, &wasm)?;

        let mut store = Store::new(&engine, ScriptContext {
            base_ptr,
            error_buffer_ptr: 0,
            error_buffer_len: 0,
        });
        let mut linker = Linker::<ScriptContext>::new(&engine);
        register_functions(&mut linker)?;
        let instance = linker.instantiate(&mut store, &module)?;

        let get_error_message = instance.get_typed_func::<(), u64>(&mut store, "get_error_message")?;
        let res = get_error_message.call(&mut store, ())?;
        let (error_buffer_len, error_buffer_ptr) = store.data().unpack(res);

        let ctx = store.data_mut();
        ctx.error_buffer_ptr = error_buffer_ptr;
        ctx.error_buffer_len = error_buffer_len;

        Ok(Self {
            engine,
            module,
            store,
            instance,
            talc: Rc::new(talc),
        })
    }

    #[inline]
    pub fn alloc(&self) -> Rc<TalcCell<TalcSource>> {
        self.talc.clone()
    }

    #[inline]
    pub fn store(&self) -> &Store<ScriptContext> {
        &self.store
    }

    #[inline]
    pub fn store_mut(&mut self) -> &mut Store<ScriptContext> {
        &mut self.store
    }

    #[inline]
    pub fn instance(&self) -> Instance {
        self.instance
    }

    #[inline]
    pub fn get_func(&mut self, name: &str) -> Option<Func> {
        self.instance.get_func(&mut self.store, name)
    }

    #[inline]
    pub fn get_typed_func<Params, Results>(&mut self, name: &str) -> XResult<TypedFunc<Params, Results>>
    where
        Params: WasmParams,
        Results: WasmResults,
    {
        Ok(self.instance.get_typed_func::<Params, Results>(&mut self.store, name)?)
    }

    #[inline]
    pub fn call<Params, Results>(&mut self, func: TypedFunc<Params, Results>, params: Params) -> XResult<Results>
    where
        Params: WasmParams,
        Results: WasmResults,
    {
        Ok(func.call(&mut self.store, params)?)
    }

    #[inline]
    pub fn to_wasm_addr<T, S: WsShared<T>>(&self, p: &S) -> u32 {
        p.to_wasm_addr(self.store.data().base_ptr)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::consts::TEST_WASM_PATH;

    #[test]
    fn test_turning_point_wasm() {
        let mut script = ScriptEngine::new(TEST_WASM_PATH, ScriptEngineConfig::default()).unwrap();

        let func = script.get_typed_func::<(), ()>("test_tmpl_id_api").unwrap();
        script.call(func, ()).unwrap();

        let func = script.get_typed_func::<(), ()>("test_symbol_api").unwrap();
        script.call(func, ()).unwrap();

        let err = XError::from("test error message");
        let msg = err.to_string();
        let len = script.store_mut().data_mut().write_error(err);
        let read_back = script.store().data().read_error_string(len).unwrap();
        assert_eq!(read_back, msg);
    }
}
