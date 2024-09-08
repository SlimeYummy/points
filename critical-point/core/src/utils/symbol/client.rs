#![allow(dead_code)]

use std::alloc::Layout;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicU32, Ordering};
use std::{alloc, fmt, mem, slice, str};

use super::base::{InnerMap, InnerNode, MAX_SYMBOL_SIZE};
use crate::utils::{XError, XResult};

#[repr(C)]
struct SymbolNode {
    next: *mut SymbolNode,
    hash: u64,
    ref_count: AtomicU32,
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
        self.ref_count.load(Ordering::Relaxed)
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
        self.ref_count = AtomicU32::new(0);
        self.length = string.len() as u16;
        let ptr = self.chars.as_mut_ptr();
        unsafe { ptr.copy_from(string.as_ptr(), self.length as usize) };
    }

    #[inline(always)]
    fn inc_ref(&self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::Relaxed)
    }

    #[inline(always)]
    fn dec_ref(&self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::Release)
    }
}

pub struct SymbolCache {
    nodes: InnerMap<SymbolNode>,
    default: *mut SymbolNode,
}

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
        unsafe { (*cache.default).inc_ref() };
        cache
    }

    fn default_symbol_node(&self) -> NonNull<SymbolNode> {
        unsafe { NonNull::new_unchecked(self.default) }
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

    fn try_cleanup(&mut self) {
        unsafe {
            self.nodes.cleanup(false, |node| {
                let layout = Layout::from_size_align_unchecked(
                    SymbolNode::size(node.as_ref().length as usize),
                    mem::align_of::<SymbolNode>(),
                );
                alloc::dealloc(node.as_ptr() as *mut u8, layout);
            });
        }
    }
}

thread_local! {
    static SYMBOL_CACHE: RefCell<SymbolCache> = RefCell::new(SymbolCache::new(2048));
}

pub struct Symbol(NonNull<SymbolNode>);

unsafe impl Send for Symbol {}
unsafe impl Sync for Symbol {}

impl Symbol {
    #[inline]
    pub const fn max_size() -> usize {
        MAX_SYMBOL_SIZE
    }

    #[inline]
    pub fn node_count() -> usize {
        return SYMBOL_CACHE.with(|cache| cache.borrow().nodes.count());
    }

    #[inline]
    pub fn node_capacity() -> usize {
        return SYMBOL_CACHE.with(|cache| cache.borrow().nodes.capacity());
    }

    #[inline]
    pub fn new(string: &str) -> XResult<Symbol> {
        return SYMBOL_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            let node = cache.new_symbol_node(string)?;
            unsafe { node.as_ref().inc_ref() };
            Ok(Symbol(node))
        });
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        return unsafe { self.0.as_ref().as_str() };
    }

    #[inline]
    pub fn ref_count(&self) -> u32 {
        return unsafe { self.0.as_ref().ref_count() };
    }

    #[inline]
    pub fn try_cleanup() {
        SYMBOL_CACHE.with(|cache| cache.borrow_mut().try_cleanup());
    }
}

impl Default for Symbol {
    fn default() -> Symbol {
        return SYMBOL_CACHE.with(|cache| {
            let node = cache.borrow().default_symbol_node();
            unsafe { node.as_ref().inc_ref() };
            Symbol(node)
        });
    }
}

impl Clone for Symbol {
    #[inline]
    fn clone(&self) -> Symbol {
        unsafe { self.0.as_ref().inc_ref() };
        Symbol(self.0)
    }
}

impl Drop for Symbol {
    fn drop(&mut self) {
        unsafe { self.0.as_ref().dec_ref() };
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
    fn test_symbol_client_alloc_size() {
        assert_eq!(SymbolNode::size(0), 23);
        assert_eq!(SymbolNode::size(1), 24);
        assert_eq!(SymbolNode::size(2), 25);

        assert!(Symbol::new("").is_ok());
        assert!(Symbol::new("1").is_ok());
        assert!(Symbol::new(&"x".repeat((1 << 16) + 1)).is_err());
    }

    #[test]
    fn test_symbol_client_ref_count() {
        let s1 = Symbol::new("hello").unwrap();
        assert_eq!(unsafe { s1.0.as_ref().ref_count() }, 1);

        {
            let s2: Symbol = Symbol::new("hello").unwrap();
            assert_eq!(unsafe { s1.0.as_ref().ref_count() }, 2);
            assert!(s1 == s2);

            let s3 = s2.clone();
            assert_eq!(unsafe { s1.0.as_ref().ref_count() }, 3);
            assert!(s1 == s3);
        }
        assert_eq!(unsafe { s1.0.as_ref().ref_count() }, 1);

        let s4 = s1;
        assert_eq!(unsafe { s4.0.as_ref().ref_count() }, 1);

        Symbol::try_cleanup();
        assert_eq!(Symbol::node_count(), 2);

        {
            let _ = Symbol::new("world").unwrap();
        }

        assert_eq!(Symbol::node_count(), 3);
        Symbol::try_cleanup();
        assert_eq!(Symbol::node_count(), 2);
    }

    #[test]
    fn test_symbol_client_cache_grow() {
        let mut cache: SymbolCache = SymbolCache::new(0);
        assert_eq!(cache.nodes.capacity(), 53);
        for idx in 0..54 {
            let str = format!("qqq{}", idx);
            cache.new_symbol_node(&str).unwrap();
        }
        assert_eq!(cache.nodes.capacity(), 97);
        assert_eq!(cache.nodes.count(), 55);

        let a = cache.new_symbol_node("qqq9").unwrap();
        let b = cache.new_symbol_node("qqq9").unwrap();
        assert!(a == b);
        assert_eq!(cache.nodes.count(), 55);

        let a = cache.new_symbol_node("qqq53").unwrap();
        let b = cache.new_symbol_node("qqq53").unwrap();
        assert!(a == b);
        assert_eq!(cache.nodes.count(), 55);
    }
}
