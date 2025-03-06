#![allow(dead_code)]

use std::alloc::Layout;
use std::cell::{Cell, RefCell};
use std::hash::{Hash, Hasher};
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicU32, Ordering};
use std::{alloc, fmt, mem, slice, str};

use super::base::{InnerMap, InnerNode, MAX_SYMBOL_SIZE};
use crate::utils::{xres, XResult};

#[repr(C)]
struct SymbolNode {
    next: *mut SymbolNode,
    hash: u64,
    ref_count: Cell<u32>,
    ref_count_mt: AtomicU32, // multi-thread reference count
    length: u16,
    cache_id: u8,
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
        self.ref_count.get() + self.ref_count_mt.load(Ordering::Relaxed)
    }
}

impl SymbolNode {
    #[inline(always)]
    fn size(str_size: usize) -> usize {
        (mem::offset_of!(SymbolNode, chars) + str_size + 1)
    }

    #[inline(always)]
    fn initialize(&mut self, cache_id: u8, hash: u64, string: &str) {
        self.next = ptr::null_mut();
        self.hash = hash;
        self.ref_count = Cell::new(0);
        self.ref_count_mt = AtomicU32::new(0);
        self.length = string.len() as u16;
        self.cache_id = cache_id;
        let ptr = self.chars.as_mut_ptr();
        unsafe {
            ptr.copy_from(string.as_ptr(), self.length as usize);
            ptr.add(self.length as usize).write(0);
        }
    }

    #[inline(always)]
    fn inc_ref(&self) -> u32 {
        self.ref_count.set(self.ref_count.get() + 1);
        self.ref_count.get()
    }

    #[inline(always)]
    fn dec_ref(&self) -> u32 {
        self.ref_count.set(self.ref_count.get() - 1);
        self.ref_count.get()
    }

    #[inline(always)]
    fn inc_ref_mt(&self) -> u32 {
        self.ref_count_mt.fetch_add(1, Ordering::Relaxed)
    }

    #[inline(always)]
    fn dec_ref_mt(&self) -> u32 {
        self.ref_count_mt.fetch_sub(1, Ordering::Release)
    }
}

thread_local! {
    static SYMBOL_CACHE: RefCell<SymbolCache> = RefCell::new(SymbolCache::new(2048));
}

static CACHE_COUNTER: AtomicU32 = AtomicU32::new(0);

pub struct SymbolCache {
    nodes: InnerMap<SymbolNode>,
    cache_id: u8,
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
        let cache_id = CACHE_COUNTER.fetch_add(1, Ordering::Relaxed);
        if cache_id > u8::MAX as u32 {
            panic!("Too many SymbolCache created");
        }

        let mut cache = SymbolCache {
            nodes: InnerMap::new(capacity),
            cache_id: cache_id as u8,
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
            return xres!(SymbolTooLong);
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
            node.as_mut().initialize(self.cache_id, hash, string);
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

macro_rules! symbol_methods {
    ($symbol:path) => {
        impl $symbol {
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

        impl Default for $symbol {
            fn default() -> $symbol {
                return SYMBOL_CACHE.with(|cache| {
                    let node = cache.borrow().default_symbol_node();
                    unsafe { node.as_ref().inc_ref() };
                    $symbol(node)
                });
            }
        }

        impl Hash for $symbol {
            #[inline]
            fn hash<H: Hasher>(&self, state: &mut H) {
                unsafe { self.0.as_ref().hash.hash(state) };
            }
        }

        impl PartialEq for $symbol {
            #[inline]
            fn eq(&self, other: &$symbol) -> bool {
                self.0 == other.0
            }
        }

        impl fmt::Debug for $symbol {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "s{:?}", self.as_str())
            }
        }

        impl fmt::Display for $symbol {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }
    };
}

/// Symbol can be used in current thread
pub struct Symbol(NonNull<SymbolNode>);

symbol_methods!(Symbol);

impl Symbol {
    #[inline]
    pub fn new(string: &str) -> XResult<Symbol> {
        return SYMBOL_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            let node = cache.new_symbol_node(string)?;
            unsafe { node.as_ref().inc_ref() };
            Ok(Symbol(node))
        });
    }

    pub fn from(symbol: &ASymbol) -> Symbol {
        SYMBOL_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            if unsafe { symbol.0.as_ref().cache_id } == cache.cache_id {
                unsafe { symbol.0.as_ref().inc_ref() };
                Symbol(symbol.0)
            } else {
                let node = cache.new_symbol_node(symbol.as_str()).unwrap(); // Impossible to meet SymbolTooLong
                unsafe { node.as_ref().inc_ref() };
                Symbol(node)
            }
        })
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

impl PartialEq<ASymbol> for Symbol {
    #[inline]
    fn eq(&self, other: &ASymbol) -> bool {
        self.0 == other.0
    }
}

/// ASymbol can be send to other thread
pub struct ASymbol(NonNull<SymbolNode>);

unsafe impl Send for ASymbol {}
unsafe impl Sync for ASymbol {}

symbol_methods!(ASymbol);

impl ASymbol {
    #[inline]
    pub fn new(string: &str) -> XResult<ASymbol> {
        return SYMBOL_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            let node = cache.new_symbol_node(string)?;
            unsafe { node.as_ref().inc_ref_mt() };
            Ok(ASymbol(node))
        });
    }

    #[inline]
    pub fn from(symbol: &Symbol) -> ASymbol {
        unsafe { symbol.0.as_ref().inc_ref_mt() };
        ASymbol(symbol.0)
    }
}

impl Clone for ASymbol {
    #[inline]
    fn clone(&self) -> ASymbol {
        unsafe { self.0.as_ref().inc_ref_mt() };
        ASymbol(self.0)
    }
}

impl Drop for ASymbol {
    fn drop(&mut self) {
        unsafe { self.0.as_ref().dec_ref_mt() };
    }
}

impl PartialEq<Symbol> for ASymbol {
    #[inline]
    fn eq(&self, other: &Symbol) -> bool {
        self.0 == other.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_client_alloc_size() {
        assert_eq!(SymbolNode::size(0), 28);
        assert_eq!(SymbolNode::size(1), 29);
        assert_eq!(SymbolNode::size(2), 30);

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
            assert_eq!(unsafe { s1.0.as_ref().ref_count_mt.load(Ordering::Relaxed) }, 0);
            assert!(s1 == s2);

            let s3 = s2.clone();
            assert_eq!(unsafe { s1.0.as_ref().ref_count() }, 3);
            assert_eq!(unsafe { s1.0.as_ref().ref_count_mt.load(Ordering::Relaxed) }, 0);
            assert!(s1 == s3);

            let a1 = ASymbol::from(&s3);
            assert_eq!(unsafe { s1.0.as_ref().ref_count() }, 4);
            assert_eq!(unsafe { s1.0.as_ref().ref_count_mt.load(Ordering::Relaxed) }, 1);
            assert!(s1 == a1);

            let a2: ASymbol = a1.clone();
            assert_eq!(unsafe { s1.0.as_ref().ref_count() }, 5);
            assert_eq!(unsafe { s1.0.as_ref().ref_count_mt.load(Ordering::Relaxed) }, 2);
            assert!(a1 == a2);
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

        let a3 = ASymbol::new("async");
        assert_eq!(Symbol::node_count(), 3);

        mem::drop(a3);
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
