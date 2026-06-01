use critical_point_macros::wasm_struct;
use rustc_hash::FxHasher;
use std::hash::Hasher;
use std::hint::{likely, unlikely};
use std::{mem, ptr, slice, str};

use crate::consts::KB;
use crate::utils::collection::PRIME_TABLE;
use crate::utils::error::{XResult, xres};
use crate::utils::macros::ifelse;

//
// SymbolNode
//

#[repr(C)]
struct SymbolNode {
    next: *mut SymbolNode,
    hash: u64,
    length: u16,
    chars: [u8; 1], // Flexible array member, we use [u8; 1] here to simplify EMPTY_NODE initialization.
}

unsafe impl Sync for SymbolNode {}

impl SymbolNode {
    const MAX_SIZE: usize = 512;
    const ALIGN_SIZE: usize = mem::align_of::<SymbolNode>();

    #[inline(always)]
    fn size(str_size: usize) -> usize {
        (mem::offset_of!(SymbolNode, chars) + str_size + 1)
    }

    #[inline(always)]
    fn initialize(&mut self, hash: u64, string: &str) {
        self.next = ptr::null_mut();
        self.hash = hash;
        debug_assert!(string.len() < Self::MAX_SIZE as usize); // string.len() should be checked in alloc_node()
        self.length = string.len() as u16;
        let ptr = self.chars.as_mut_ptr();
        unsafe {
            ptr.copy_from(string.as_ptr(), self.length as usize);
            ptr.add(self.length as usize).write(0);
        }
    }

    #[inline(always)]
    pub(super) fn as_str(&self) -> &str {
        let ptr = self.chars.as_ptr();
        unsafe {
            let v = slice::from_raw_parts(ptr, self.length as usize);
            str::from_utf8_unchecked(v)
        }
    }

    #[inline(always)]
    pub(super) fn length(&self) -> usize {
        self.length as usize
    }

    #[inline(always)]
    fn to_str_ptr(node: *const SymbolNode) -> *const u8 {
        debug_assert!(!node.is_null(), "SymbolNode pointer is null");
        unsafe { &(*node).chars as *const u8 }
    }

    #[inline(always)]
    unsafe fn from_str_ptr(ptr: *const u8) -> *const SymbolNode {
        debug_assert!(!ptr.is_null(), "Symbol pointer is null");
        let node = unsafe { ptr.sub(mem::offset_of!(SymbolNode, chars)) };
        debug_assert!(node as usize % mem::align_of::<SymbolNode>() == 0);
        node as *const SymbolNode
    }
}

//
// SymbolCache
//

pub(super) struct SymbolCache {
    arenas: Vec<Box<[u8]>>,
    arena_size: usize,
    arena_ptr: usize,

    nodes: Vec<*mut SymbolNode>,
    prime: u64,
    prime_pos: usize,
    count: usize,
    tmp: Vec<*mut SymbolNode>,
}

unsafe impl Send for SymbolCache {}
unsafe impl Sync for SymbolCache {}

impl SymbolCache {
    #[inline]
    const fn new(capacity: usize, arena_size: usize) -> SymbolCache {
        let mut pow = 0;
        let mut num = capacity;
        while num > 1 {
            num >>= 1;
            pow += 1;
        }
        pow = ifelse!(pow < 5, 5, pow);
        let prime_pos = pow - 5;

        SymbolCache {
            arenas: vec![],
            arena_size,
            arena_ptr: 0,

            nodes: vec![],
            prime: PRIME_TABLE[prime_pos] as u64,
            count: 1, // +1 for empty node
            prime_pos,
            tmp: Vec::new(),
        }
    }

    fn init(&mut self) {
        self.arenas = vec![vec![0; self.arena_size].into_boxed_slice()];
        self.nodes = vec![ptr::null_mut(); self.prime as usize];
        self.tmp.reserve(16);
    }

    fn find(&self, string: &str, hash: u64) -> Option<*const SymbolNode> {
        let pos = (hash % self.prime) as usize;
        let mut node = unsafe { self.nodes.get_unchecked(pos) };
        loop {
            if node.is_null() {
                return None;
            }
            unsafe {
                let node_ref = &**node;
                if node_ref.hash == hash && string == node_ref.as_str() {
                    return Some(*node);
                }
                node = &node_ref.next;
            }
        }
    }

    fn insert(&mut self, string: &str, hash: u64) -> XResult<*const SymbolNode> {
        let node = self.alloc_node(string)?;
        unsafe { (*node).initialize(hash, string) };

        self.try_grow();

        let hash = unsafe { (*node).hash };
        let pos = (hash % self.prime) as usize;
        unsafe {
            let next = *self.nodes.get_unchecked(pos);
            *self.nodes.get_unchecked_mut(pos) = node;
            (*node).next = next;
        }
        self.count += 1;

        Ok(node)
    }

    #[inline]
    fn alloc_node(&mut self, string: &str) -> XResult<*mut SymbolNode> {
        let node_size = SymbolNode::size(string.len());
        if unlikely(node_size > SymbolNode::MAX_SIZE) {
            return xres!(InvalidSymbol; "too long");
        }

        self.arena_ptr = (self.arena_ptr + SymbolNode::ALIGN_SIZE - 1) & !(SymbolNode::ALIGN_SIZE - 1);
        if unlikely(self.arena_ptr + node_size > self.arena_size) {
            self.arenas.push(vec![0; self.arena_size].into_boxed_slice());
            self.arena_ptr = 0;
        }
        let arena_len = self.arenas.len();
        let arena = unsafe { self.arenas.get_unchecked_mut(arena_len - 1) };

        let node = unsafe { arena.as_mut_ptr().add(self.arena_ptr) as *mut SymbolNode };
        self.arena_ptr += node_size;
        Ok(node)
    }

    #[inline]
    fn capacity(&self) -> usize {
        // 75% load factor
        self.prime as usize * 3 / 4
    }

    #[inline]
    fn try_grow(&mut self) {
        if likely(self.count < self.capacity()) {
            return;
        }

        let new_prime_pos = self.prime_pos + 1;
        let new_prime = PRIME_TABLE[new_prime_pos] as u64;
        let mut new_nodes = vec![ptr::null_mut(); new_prime as usize];

        self.tmp.clear();
        for node in self.nodes.iter() {
            let mut current = *node;
            while !current.is_null() {
                self.tmp.push(current);
                current = unsafe { (*current).next };
            }

            // Insert in reverse order to preserve original traversal order
            for node in self.tmp.iter().rev().cloned() {
                let hash = unsafe { (*node).hash };
                let pos = (hash % new_prime) as usize;
                unsafe {
                    (*node).next = *new_nodes.get_unchecked(pos);
                    *new_nodes.get_unchecked_mut(pos) = node;
                }
            }
            self.tmp.clear();
        }

        self.nodes = new_nodes;
        self.prime = new_prime;
        self.prime_pos = new_prime_pos;
    }

    unsafe fn clean_up(&mut self) {
        while self.arenas.len() > 1 {
            self.arenas.pop();
        }
        self.arenas[0].fill(0);
        self.arena_ptr = 0;

        self.nodes.fill(ptr::null_mut());
        self.count = 1; // +1 for empty node
    }
}

//
// Statics
//

#[cfg(not(feature = "server-side"))]
static SYMBOL_CACHE: std::sync::Mutex<SymbolCache> = std::sync::Mutex::new(SymbolCache::new(512, 16 * KB));
#[cfg(feature = "server-side")]
static SYMBOL_CACHE: std::sync::RwLock<SymbolCache> = std::sync::RwLock::new(SymbolCache::new(512, 16 * KB));

#[ctor::ctor]
fn init_symbol_cache() {
    #[cfg(not(feature = "server-side"))]
    SYMBOL_CACHE.lock().unwrap().init();
    #[cfg(feature = "server-side")]
    SYMBOL_CACHE.write().unwrap().init();
}

static EMPTY_NODE: SymbolNode = SymbolNode {
    next: ptr::null_mut(),
    hash: 0xF456D26876D72D91,
    length: 0,
    chars: [0],
};

//
// Symbol
//

#[repr(transparent)]
#[wasm_struct(8, 8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Symbol(#[wasm_hide(8, 8)] *const u8);

unsafe impl Send for Symbol {}
unsafe impl Sync for Symbol {}

impl Symbol {
    #[inline(always)]
    fn hash(string: &str) -> u64 {
        let mut hasher = FxHasher::default();
        hasher.write(string.as_bytes());
        hasher.finish()
    }

    #[cfg(not(feature = "server-side"))]
    pub fn new(string: &str) -> XResult<Symbol> {
        let mut cache = SYMBOL_CACHE.lock().unwrap();

        if unlikely(string.is_empty()) {
            return Ok(Symbol(SymbolNode::to_str_ptr(&EMPTY_NODE)));
        }

        let hash = Self::hash(string);
        if let Some(node) = cache.find(string, hash) {
            return Ok(Symbol(SymbolNode::to_str_ptr(node)));
        }

        let node = cache.insert(string, hash)?;
        Ok(Symbol(SymbolNode::to_str_ptr(node)))
    }

    #[cfg(feature = "server-side")]
    #[inline]
    pub fn new(string: &str) -> XResult<Symbol> {
        let cache = SYMBOL_CACHE.read().unwrap();

        if unlikely(string.is_empty()) {
            return Ok(Symbol(SymbolNode::to_str_ptr(&EMPTY_NODE)));
        }

        let hash = Self::hash(string);
        if let Some(node) = cache.find(string, hash) {
            return Ok(Symbol(SymbolNode::to_str_ptr(node)));
        }

        drop(cache);
        let mut cache = SYMBOL_CACHE.write().unwrap();

        let node = cache.insert(string, hash)?;
        Ok(Symbol(SymbolNode::to_str_ptr(node)))
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { (*SymbolNode::from_str_ptr(self.0)).as_str() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { (*SymbolNode::from_str_ptr(self.0)).length() }
    }

    #[inline]
    pub fn precomputed_hash(&self) -> u64 {
        unsafe { (*SymbolNode::from_str_ptr(self.0)).hash }
    }

    pub fn count_capacity_memory() -> (usize, usize, usize) {
        #[cfg(not(feature = "server-side"))]
        let cache = SYMBOL_CACHE.lock().unwrap();
        #[cfg(feature = "server-side")]
        let cache = SYMBOL_CACHE.read().unwrap();

        let count = cache.count;
        let capacity = cache.capacity();
        let memory = cache.arena_size * cache.arenas.len();
        (count, capacity, memory)
    }

    /// # Safety
    ///
    /// **This is an extremely dangerous function.** Calling this method will:
    /// 1. Deallocate all arena memory except the first arena
    /// 2. Invalidate ALL existing `Symbol` instances (including those held by callers)
    /// 3. Reset the hash table to empty state
    ///
    /// After calling this function, ANY access to previously created `Symbol` instances
    /// will result in **use-after-free** (reading from deallocated memory), which causes:
    /// - Undefined behavior
    /// - Potential security vulnerabilities
    /// - Data corruption
    /// - Program crashes
    ///
    /// ## When to use
    ///
    /// This function should ONLY be called when:
    /// - You are certain NO `Symbol` instances exist outside the cache
    /// - You are performing a complete reset of the application state
    /// - All previous `Symbol` holders have been dropped
    ///
    /// ## Example of WRONG usage
    ///
    /// ```ignore
    /// let s = Symbol::new("hello").unwrap();
    /// unsafe { Symbol::clean_up() }; // DANGER!
    /// let _ = s.as_str(); // USE-AFTER-FREE!
    /// ```
    ///
    /// The caller is responsible for ensuring no `Symbol` instances survive across this call.
    pub unsafe fn clean_up() {
        #[cfg(not(feature = "server-side"))]
        let mut cache = SYMBOL_CACHE.lock().unwrap();
        #[cfg(feature = "server-side")]
        let mut cache = SYMBOL_CACHE.write().unwrap();

        unsafe { cache.clean_up() };
    }
}

impl Default for Symbol {
    fn default() -> Self {
        Symbol(SymbolNode::to_str_ptr(&EMPTY_NODE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_cache_empty() {
        let h0 = Symbol::hash("");
        assert_eq!(h0, EMPTY_NODE.hash);
        assert_eq!(Symbol::default().precomputed_hash(), h0);
        assert_eq!(Symbol::default().as_str(), "");
        assert_eq!(Symbol::default().len(), 0);
        assert!(Symbol::default() == Symbol::new("").unwrap());
    }

    #[test]
    fn test_symbol_cache_alloc_size() {
        assert_eq!(SymbolNode::size(0), 19);
        assert_eq!(SymbolNode::size(1), 20);
        assert_eq!(SymbolNode::size(2), 21);

        assert!(Symbol::new("").is_ok());
        assert!(Symbol::new("1").is_ok());
        assert!(Symbol::new(&"x".repeat(513 - 18)).is_err());
    }

    #[test]
    fn test_symbol_cache_grow() {
        const COUNT: usize = 40; // 54 * 3 / 4

        let mut cache: SymbolCache = SymbolCache::new(0, 1 * KB);
        cache.init();
        assert_eq!(cache.nodes.capacity(), 53);
        for idx in 0..COUNT {
            let str = format!("qqq{}", idx);
            cache.insert(&str, Symbol::hash(&str)).unwrap();
        }
        assert_eq!(cache.nodes.capacity(), 97);
        assert_eq!(cache.count, COUNT + 1);

        let a = cache.find("qqq9", Symbol::hash("qqq9")).unwrap();
        let b = cache.find("qqq9", Symbol::hash("qqq9")).unwrap();
        assert!(a == b);
        assert_eq!(cache.count, COUNT + 1);

        let a = cache.find("qqq39", Symbol::hash("qqq39")).unwrap();
        let b = cache.find("qqq39", Symbol::hash("qqq39")).unwrap();
        assert!(a == b);
        assert_eq!(cache.count, COUNT + 1);
    }
}
