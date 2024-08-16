use std::hash::{BuildHasher, Hasher};
use std::ptr::{self, NonNull};

use crate::utils::{DeterministicState, PRIME_TABLE};

pub const MAX_SYMBOL_SIZE: usize = 1 << 16;

pub(super) trait InnerNode {
    fn hash(&self) -> u64;
    fn as_str(&self) -> &str;
    fn next(&mut self) -> &mut *mut Self;
    fn ref_count(&self) -> u32;
}

pub(super) struct InnerMap<N: InnerNode> {
    nodes: Vec<*mut N>,
    prime: u64,
    prime_pos: usize,
    count: usize,
    state: DeterministicState,
}

impl<N: InnerNode> InnerMap<N> {
    #[inline]
    pub(super) fn new(capacity: usize) -> InnerMap<N> {
        let mut pow = 0;
        let mut num = capacity;
        while num > 1 {
            num >>= 1;
            pow += 1;
        }
        pow = pow.max(5);
        let prime_pos = pow - 5;

        return InnerMap {
            nodes: vec![ptr::null_mut(); PRIME_TABLE[prime_pos] as usize],
            prime: PRIME_TABLE[prime_pos] as u64,
            count: 0,
            prime_pos,
            state: DeterministicState::new(),
        };
    }

    #[inline(always)]
    pub(super) fn hash(&self, string: &str) -> u64 {
        let mut hasher = self.state.build_hasher();
        hasher.write(string.as_bytes());
        return hasher.finish();
    }

    #[inline]
    pub(super) fn find(&self, string: &str, hash: u64) -> Option<NonNull<N>> {
        let pos = (hash % self.prime) as usize;
        let mut node = unsafe { self.nodes.get_unchecked(pos) };
        loop {
            if node.is_null() {
                return None;
            }
            unsafe {
                if string == (**node).as_str() {
                    return Some(NonNull::new_unchecked(*node));
                }
                node = (**node).next();
            }
        }
    }

    #[inline]
    pub(super) fn insert(&mut self, mut node: NonNull<N>) {
        self.try_grow();

        let hash = unsafe { node.as_ref().hash() };
        let pos = (hash % self.prime) as usize;
        unsafe {
            let next = *self.nodes.get_unchecked(pos);
            *self.nodes.get_unchecked_mut(pos) = node.as_ptr();
            *node.as_mut().next() = next;
        }
        self.count += 1;
    }

    // #[inline]
    // pub(super) fn remove(&mut self, mut node: NonNull<N>) -> bool {
    //     let hash = unsafe { node.as_ref().hash() };
    //     let pos = (hash % self.prime) as usize;
    //     let mut iter = unsafe { self.nodes.get_unchecked_mut(pos) };
    //     while !iter.is_null() {
    //         if *iter == node.as_ptr() {
    //             unsafe {
    //                 *iter = *node.as_mut().next();
    //                 *node.as_mut().next() = ptr::null_mut();
    //             }
    //             self.count -= 1;
    //             return true;
    //         }
    //         iter = unsafe { (**iter).next() };
    //     }
    //     return false;
    // }

    #[inline]
    pub(super) fn try_grow(&mut self) {
        if self.count < self.prime as usize {
            return;
        }

        let new_prime_pos = self.prime_pos + 1;
        let new_prime = PRIME_TABLE[new_prime_pos] as u64;
        let mut new_nodes = vec![ptr::null_mut(); new_prime as usize];

        for node in self.nodes.iter() {
            let mut old_next = *node;
            while !old_next.is_null() {
                let node = old_next;
                old_next = unsafe { *(*old_next).next() };

                let hash = unsafe { (*node).hash() };
                let pos = (hash % new_prime) as usize;
                unsafe {
                    let new_next = *new_nodes.get_unchecked(pos);
                    *new_nodes.get_unchecked_mut(pos) = node;
                    *(*node).next() = new_next;
                }
            }
        }

        self.nodes = new_nodes;
        self.prime = new_prime;
        self.prime_pos = new_prime_pos;
    }

    #[inline]
    pub(crate) unsafe fn cleanup<F: Fn(NonNull<N>)>(&mut self, ignore_ref_count: bool, func: F) -> usize {
        let mut count = 0;
        for idx in 0..self.nodes.len() {
            let mut node = unsafe { self.nodes.get_unchecked_mut(idx) };
            while !node.is_null() {
                let next = unsafe { (**node).next() };
                if ignore_ref_count || unsafe { (**node).ref_count() } == 0 {
                    self.count -= 1;
                    count += 1;
                    let free_node = *node;
                    *node = *next;
                    func(NonNull::new_unchecked(free_node));
                } else {
                    node = next;
                }
            }
        }
        return count;
    }

    #[inline(always)]
    pub(crate) fn count(&self) -> usize {
        return self.count;
    }

    #[inline(always)]
    pub(crate) fn capacity(&self) -> usize {
        return self.prime as usize;
    }
}
