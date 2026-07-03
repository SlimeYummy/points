use anyhow::Result;

use crate::auto_gen::*;
#[cfg(target_arch = "wasm32")]
use crate::error::HostError;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "host")]
unsafe extern "C" {
    fn tmpl_id_new(str_ptr: u32, str_len: u32, out_ptr: u32) -> u32;
    fn tmpl_id_to_string(id_ptr: u32, out_ptr: u32, out_len: u32) -> u32;
    fn symbol_new(str_ptr: u32, str_len: u32, out_ptr: u32) -> u32;
    fn symbol_to_string(sym_ptr: u32, out_ptr: u32, out_len: u32) -> u32;
    fn symbol_len(sym_ptr: u32) -> u32;
}

impl TmplID {
    #[cfg(target_arch = "wasm32")]
    pub fn new(s: &str) -> Result<TmplID> {
        let mut id = TmplID::INVALID;
        let err = unsafe { tmpl_id_new(s.as_ptr() as u32, s.len() as u32, &raw mut id as *mut _ as u32) };

        HostError::read_result(err as usize)?;
        Ok(id)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(_s: &str) -> Result<TmplID> {
        Err(anyhow::anyhow!("TmplID::new only available in wasm32"))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn to_string(&self) -> String {
        let mut buf = vec![0u8; 192];
        let len =
            unsafe { tmpl_id_to_string(self as *const _ as u32, buf.as_mut_ptr() as u32, buf.len() as u32) } as usize;

        if len > 192 {
            // TmplID's generation mechanism ensures that its length must <= 192.
            return "Invalid.?".to_string();
        }

        buf.truncate(len);
        unsafe { String::from_utf8_unchecked(buf) }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn to_string(&self) -> String {
        unreachable!("TmplID::to_string only available in wasm32")
    }
}

impl Symbol {
    #[cfg(target_arch = "wasm32")]
    pub fn new(s: &str) -> Result<Symbol> {
        let mut sym: Symbol = unsafe { std::mem::zeroed() };
        let err = unsafe { symbol_new(s.as_ptr() as u32, s.len() as u32, &raw mut sym as *mut _ as u32) };

        HostError::read_result(err as usize)?;
        Ok(sym)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(_s: &str) -> Result<Symbol> {
        Err(anyhow::anyhow!("Symbol::new only available in wasm32"))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn len(&self) -> usize {
        unsafe { symbol_len(self as *const _ as u32) as usize }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn len(&self) -> usize {
        unreachable!("Symbol::len only available in wasm32")
    }

    #[cfg(target_arch = "wasm32")]
    pub fn to_string(&self) -> String {
        let len = self.len();
        let mut buf = vec![0u8; len];
        let actual_len =
            unsafe { symbol_to_string(self as *const _ as u32, buf.as_mut_ptr() as u32, buf.len() as u32) } as usize;

        if actual_len != len {
            return "Invalid??".to_string();
        }

        unsafe { String::from_utf8_unchecked(buf) }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn to_string(&self) -> String {
        unreachable!("Symbol::to_string only available in wasm32")
    }
}
