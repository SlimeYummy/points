#![allow(dead_code)]

use anyhow::anyhow;
use std::alloc::Layout;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::Path;
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::{alloc, fmt, mem, slice, str};

use super::base::{InnerMap, InnerNode, MAX_SYMBOL_SIZE};
use crate::utils::{XError, XResult};

#[repr(C)]
struct SymbolNode {
    next: *mut SymbolNode,
    hash: u64,
    length: u16,
    chars: [u8; 0],
}

impl InnerNode for SymbolNode {
    #[inline(always)]
    fn hash(&self) -> u64 {
        self.hash
    }

    #[inline(always)]
    fn as_str(&self) -> &str {
        let ptr = self.chars.as_ptr();
        unsafe {
            let v = slice::from_raw_parts(ptr, self.length as usize);
            return str::from_utf8_unchecked(v);
        };
    }

    #[inline(always)]
    fn next(&mut self) -> &mut *mut Self {
        &mut self.next
    }

    #[inline(always)]
    fn ref_count(&self) -> u32 {
        1
    }
}

impl SymbolNode {
    #[inline(always)]
    fn size(str_size: usize) -> usize {
        (mem::offset_of!(SymbolNode, chars) + str_size + 1)
    }

    #[inline(always)]
    fn initialize(&mut self, hash: u64, string: &str) {
        self.next = ptr::null_mut();
        self.hash = hash;
        self.length = string.len() as u16;
        let ptr = self.chars.as_mut_ptr();
        unsafe { ptr.copy_from(string.as_ptr(), self.length as usize) };
    }
}

pub struct SymbolCache {
    nodes: InnerMap<SymbolNode>,
    default: *mut SymbolNode,
}

unsafe impl Sync for SymbolCache {}
unsafe impl Send for SymbolCache {}

impl Drop for SymbolCache {
    fn drop(&mut self) {
        unsafe {
            self.nodes.cleanup(true, |node| {
                let layout = Layout::from_size_align_unchecked(
                    SymbolNode::size(node.as_ref().length as usize),
                    mem::align_of::<SymbolNode>(),
                );
                alloc::dealloc(node.as_ptr() as *mut u8, layout);
            });
        }
    }
}

impl SymbolCache {
    fn new(capacity: usize) -> SymbolCache {
        let mut cache = SymbolCache {
            nodes: InnerMap::new(capacity),
            default: ptr::null_mut(),
        };
        cache.default = cache.new_symbol_node("").unwrap().as_ptr();
        cache
    }

    fn preload_from_strings(&mut self, strings: &[&str]) -> anyhow::Result<()> {
        for string in strings.iter() {
            self.new_symbol_node(string)?;
        }
        self.new_symbol_node("")?;
        Ok(())
    }

    fn preload_from_json<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        #[derive(serde::Deserialize)]
        struct JsonStrings<'t>(#[serde(borrow)] Vec<&'t str>);

        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let strings: JsonStrings<'_> = serde_json::from_slice(&buf)?;
        for string in strings.0.iter() {
            self.new_symbol_node(string)?;
        }
        self.new_symbol_node("")?;
        Ok(())
    }

    fn preload_from_rkyv<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        #[derive(rkyv::Archive)]
        #[archive(check_bytes)]
        struct RkyvStrings(Vec<String>);

        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let archive = rkyv::check_archived_root::<RkyvStrings>(&buf[..]).map_err(|e| anyhow!("rkyv: {}", e))?;
        for string in archive.0.iter() {
            self.new_symbol_node(string)?;
        }
        self.new_symbol_node("")?;
        Ok(())
    }

    fn new_symbol_node(&mut self, string: &str) -> XResult<NonNull<SymbolNode>> {
        if string.len() > MAX_SYMBOL_SIZE {
            return Err(XError::SymbolTooLong);
        }

        let hash = self.nodes.hash(string);
        if let Some(node) = self.nodes.find(string, hash) {
            return Ok(node);
        }

        let mut node;
        unsafe {
            let layout =
                Layout::from_size_align_unchecked(SymbolNode::size(string.len()), mem::align_of::<SymbolNode>());
            node = NonNull::new_unchecked(alloc::alloc(layout) as *mut SymbolNode);
            node.as_mut().initialize(hash, string);
        }
        self.nodes.insert(node);
        Ok(node)
    }

    fn find_symbol_node(&self, string: &str) -> XResult<NonNull<SymbolNode>> {
        let hash = self.nodes.hash(string);
        self.nodes.find(string, hash).ok_or(XError::SymbolNotFound)
    }

    fn default_symbol_node(&self) -> NonNull<SymbolNode> {
        unsafe { NonNull::new_unchecked(self.default) }
    }
}

static SYMBOL_CACHE: AtomicPtr<SymbolCache> = AtomicPtr::new(ptr::null_mut());

pub struct Symbol(NonNull<SymbolNode>);

unsafe impl Send for Symbol {}
unsafe impl Sync for Symbol {}

impl Symbol {
    pub fn preload_strings(strings: &[&str]) -> anyhow::Result<()> {
        Symbol::preload_impl(|cache| cache.preload_from_strings(strings))
    }

    pub fn preload_json<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
        Symbol::preload_impl(|cache| cache.preload_from_json(path))
    }

    pub fn preload_rkyv<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
        Symbol::preload_impl(|cache| cache.preload_from_rkyv(path))
    }

    fn preload_impl<F>(preload: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut SymbolCache) -> anyhow::Result<()>,
    {
        if !SYMBOL_CACHE.load(Ordering::SeqCst).is_null() {
            return Err(anyhow!("Symbol cache already preloaded"));
        }

        let mut cache = Box::new(SymbolCache::new(1024 * 8));
        preload(&mut cache)?;

        let ok = SYMBOL_CACHE.compare_exchange(
            ptr::null_mut(),
            cache.as_mut() as *mut _,
            Ordering::SeqCst,
            Ordering::SeqCst,
        );
        if ok.is_err() {
            return Err(anyhow!("Symbol cache already preloaded"));
        }
        mem::forget(cache);
        Ok(())
    }

    #[inline]
    fn symbol_cache() -> Option<&'static SymbolCache> {
        let cache = SYMBOL_CACHE.load(Ordering::Relaxed);
        if cache.is_null() {
            return None;
        }
        Some(unsafe { &*cache })
    }

    #[inline]
    pub const fn max_size() -> usize {
        MAX_SYMBOL_SIZE
    }

    #[inline]
    pub fn node_count() -> usize {
        match Self::symbol_cache() {
            Some(cache) => cache.nodes.count(),
            None => 0,
        }
    }

    #[inline]
    pub fn node_capacity() -> usize {
        match Self::symbol_cache() {
            Some(cache) => cache.nodes.capacity(),
            None => 0,
        }
    }

    #[inline]
    pub fn new(string: &str) -> XResult<Symbol> {
        match Self::symbol_cache() {
            Some(cache) => {
                let node = cache.find_symbol_node(string)?;
                Ok(Symbol(node))
            }
            None => Err(XError::SymbolNotPreloaded),
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        return unsafe { self.0.as_ref().as_str() };
    }

    #[inline]
    pub fn ref_count(&self) -> u32 {
        u32::MAX
    }
}

impl Default for Symbol {
    fn default() -> Symbol {
        Symbol(Self::symbol_cache().unwrap().default_symbol_node())
    }
}

impl Clone for Symbol {
    #[inline]
    fn clone(&self) -> Symbol {
        Symbol(self.0)
    }
}

impl Hash for Symbol {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { self.0.as_ref().hash.hash(state) };
    }
}

impl PartialEq for Symbol {
    #[inline]
    fn eq(&self, other: &Symbol) -> bool {
        self.0 == other.0
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "s{:?}", self.as_str())
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_symbol() {
        let err = Symbol::new("123").unwrap_err();
        assert!(matches!(err, XError::SymbolNotPreloaded));

        let res = Symbol::preload_strings(&[&"x".repeat((1 << 16) + 1)]);
        assert!(res.is_err());

        Symbol::preload_strings(&["aaa", "bbb", &"x".repeat(16), &"x".repeat(32), &"x".repeat(64)]).unwrap();
        assert_eq!(Symbol::node_count(), 6);
        assert_eq!(Symbol::node_capacity(), 12289);

        let s1 = Symbol::new("aaa").unwrap();
        assert_eq!(s1.as_str(), "aaa");

        let s2 = Symbol::new("bbb").unwrap();
        assert_eq!(s2.as_str(), "bbb");

        let s3 = Symbol::new(&"x".repeat(16)).unwrap();
        let s4 = s3.clone();
        assert!(s3 == s4);

        let res = Symbol::new("ccc");
        assert!(res.is_err());
    }
}
