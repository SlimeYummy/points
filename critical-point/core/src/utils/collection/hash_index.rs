use std::alloc::Layout;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash, RandomState};
use std::{alloc, fmt, ptr};

use super::hash::{DeterministicState, PRIME_TABLE};

const OCCUPANCY: f64 = 0.6;

#[derive(Debug, Clone)]
struct IndexNode<K, V> {
    hash: u64,
    next: u32,
    key: K,
    value: V,
}

pub struct HashIndex<K, V, S = RandomState> {
    nodes: *mut Option<IndexNode<K, V>>,
    prime: u32,
    prime_pos: i16,
    capacity: u32,
    count: u32,
    state: S,
}

pub type DtHashIndex<K, V> = HashIndex<K, V, DeterministicState>;

unsafe impl<K, V, S: BuildHasher> Send for HashIndex<K, V, S> {}
unsafe impl<K, V, S: BuildHasher> Sync for HashIndex<K, V, S> {}

impl<K, V, S> Default for HashIndex<K, V, S>
where
    K: Clone + Hash + PartialEq,
    V: Clone,
    S: BuildHasher + Default,
{
    fn default() -> HashIndex<K, V, S> {
        HashIndex::new()
    }
}

impl<K, V, S> Drop for HashIndex<K, V, S> {
    fn drop(&mut self) {
        for idx in 0..(self.prime as usize) {
            unsafe { self.nodes.add(idx).drop_in_place() };
        }
        let layout = Layout::array::<Option<IndexNode<K, V>>>(self.prime as usize).unwrap();
        unsafe { alloc::dealloc(self.nodes as *mut u8, layout) };
        self.nodes = ptr::null_mut();
    }
}

impl<K, V, S> HashIndex<K, V, S>
where
    K: Clone + Hash + PartialEq,
    V: Clone,
    S: BuildHasher + Default,
{
    pub fn new() -> HashIndex<K, V, S> {
        HashIndex {
            nodes: ptr::null_mut(),
            prime: 0,
            prime_pos: -1,
            capacity: 0,
            count: 0,
            state: S::default(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        HashIndex::with_capacity_and_hasher(capacity, S::default())
    }
}

impl<K, V, S> HashIndex<K, V, S>
where
    K: Clone + Hash + PartialEq,
    V: Clone,
    S: BuildHasher,
{
    pub fn with_capacity_and_hasher(capacity: usize, state: S) -> Self {
        let mut pow = 0;
        let mut num = ((capacity as f64) / OCCUPANCY).ceil() as u32;
        while num > 1 {
            num >>= 1;
            pow += 1;
        }
        pow = pow.max(5);
        let prime_pos = pow - 5;
        let prime = PRIME_TABLE[prime_pos];

        let layout = Layout::array::<Option<IndexNode<K, V>>>(prime as usize).unwrap();
        let nodes = unsafe { alloc::alloc(layout) as *mut Option<IndexNode<K, V>> };
        for pos in 0..(prime as usize) {
            unsafe { nodes.add(pos).write(None) };
        }

        HashIndex {
            nodes,
            prime,
            prime_pos: prime_pos as i16,
            capacity: ((prime as f64) * OCCUPANCY).ceil() as u32,
            count: 0,
            state,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.try_grow();

        let hash = self.state.hash_one(&key);

        Self::insert_impl(self.nodes, self.prime as usize, IndexNode {
            hash,
            next: u32::MAX,
            key,
            value,
        });
        self.count += 1;
    }

    #[inline]
    fn try_grow(&mut self) {
        if self.count < self.capacity {
            return;
        }

        let new_prime_pos = self.prime_pos + 1;
        let new_prime = PRIME_TABLE[new_prime_pos as usize];
        let layout = Layout::array::<Option<IndexNode<K, V>>>(new_prime as usize).unwrap();
        let new_nodes = unsafe { alloc::alloc(layout) as *mut Option<IndexNode<K, V>> };
        for pos in 0..(new_prime as usize) {
            unsafe { new_nodes.add(pos).write(None) };
        }

        let prime = self.prime as usize;
        let mut offset = 0;
        for pos in 0..prime {
            let node = unsafe { &mut *self.nodes.add(pos) };
            if node.is_none() {
                offset = pos;
                break;
            }
        }

        // Start loop at a None node, to keep the inserted order of nodes
        for pos in offset..(offset + prime) {
            let node: &mut Option<IndexNode<K, V>> = unsafe { &mut *self.nodes.add(pos % prime) };
            if let Some(mut node) = node.take() {
                node.next = u32::MAX;
                Self::insert_impl(new_nodes, new_prime as usize, node);
            }
            else {
                continue;
            }
        }

        let old_layout = Layout::array::<Option<IndexNode<K, V>>>(prime).unwrap();
        unsafe { alloc::dealloc(self.nodes as *mut u8, old_layout) };

        self.nodes = new_nodes;
        self.prime = new_prime;
        self.prime_pos = new_prime_pos;
        self.capacity = ((new_prime as f64) * OCCUPANCY).ceil() as u32;
    }

    #[inline]
    fn insert_impl(nodes: *mut Option<IndexNode<K, V>>, prime: usize, new: IndexNode<K, V>) {
        let mut pos = (new.hash % (prime as u64)) as usize;
        let mut prev = None;
        loop {
            let node: &mut Option<IndexNode<K, V>> = unsafe { &mut *nodes.add(pos) };
            if let Some(node) = node {
                if node.key == new.key {
                    prev = Some(pos);
                    if node.next != u32::MAX {
                        pos = node.next as usize;
                        continue;
                    }
                }
                pos = (pos + 1) % prime;
            }
            else {
                *node = Some(new);
                if let Some(prev) = prev {
                    let prev_node = unsafe { &mut *nodes.add(prev) };
                    prev_node.as_mut().unwrap().next = pos as u32;
                }
                break;
            }
        }
    }

    pub fn find<'a, 'b>(&'a self, key: &'b K) -> Option<IndexValueIter<'a, 'b, K, V, S>> {
        if self.nodes.is_null() {
            return None;
        }

        let hash = self.state.hash_one(key);

        let mut pos = (hash % (self.prime as u64)) as usize;
        loop {
            let node = unsafe { &*self.nodes.add(pos) };
            match node {
                Some(node) => {
                    if node.key == *key {
                        return Some(IndexValueIter { map: self, key, pos });
                    }
                    else {
                        pos = (pos + 1) % (self.prime as usize);
                    }
                }
                None => return None,
            }
        }
    }

    #[inline]
    pub fn find_iter<'a, 'b>(&'a self, key: &'b K) -> IndexValueIter<'a, 'b, K, V, S> {
        match self.find(key) {
            Some(iter) => iter,
            None => IndexValueIter {
                map: self,
                key,
                pos: (u32::MAX as usize),
            },
        }
    }

    #[inline]
    pub fn find_first<'a>(&'a self, key: &K) -> Option<&'a V> {
        self.find(key)?.next()
    }

    #[inline]
    pub fn contain(&self, key: &K) -> bool {
        return self.find(key).is_some();
    }

    #[inline]
    pub fn count(&self, key: &K) -> usize {
        return match self.find(key) {
            Some(iter) => iter.count(),
            None => 0,
        };
    }

    #[inline]
    pub fn iter(&self) -> HashIndexIter<'_, K, V, S> {
        HashIndexIter { map: self, pos: 0 }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count as usize
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity as usize
    }
}

impl<K: Debug, V: Debug, S> fmt::Debug for HashIndex<K, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        for idx in 0..(self.prime as usize) {
            let node = unsafe { &*self.nodes.add(idx) };
            if let Some(node) = node {
                map.entry(&node.key, &node.value);
            }
        }
        map.finish()
    }
}

pub struct IndexValueIter<'a, 'b, K, V, S> {
    map: &'a HashIndex<K, V, S>,
    key: &'b K,
    pos: usize,
}

impl<'a, 'b, K: PartialEq, V, S> Iterator for IndexValueIter<'a, 'b, K, V, S> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos != (u32::MAX as usize) {
            let node = unsafe { &*self.map.nodes.add(self.pos) };
            if let Some(node) = node {
                self.pos = node.next as usize;
                if node.key == *self.key {
                    return Some(&node.value);
                }
            }
        }
        None
    }
}

pub struct HashIndexIter<'a, K, V, S> {
    map: &'a HashIndex<K, V, S>,
    pos: usize,
}

impl<'a, K, V, S> Iterator for HashIndexIter<'a, K, V, S> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < (self.map.prime as usize) {
            let node = unsafe { &*self.map.nodes.add(self.pos) };
            if let Some(node) = node {
                self.pos += 1;
                return Some((&node.key, &node.value));
            }
            self.pos += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hasher;

    use super::*;
    use crate::utils::id::{id, TmplID};

    #[test]
    fn test_insert_find() {
        let mut hi: DtHashIndex<TmplID, u32> = DtHashIndex::new();
        hi.insert(id!("#.Aaa"), 1);
        hi.insert(id!("#.Bbb"), 2);
        hi.insert(id!("#.Aaa"), 3);
        assert_eq!(hi.len(), 3);
        assert_eq!(hi.find(&id!("#.Aaa")).unwrap().copied().collect::<Vec<_>>(), vec![1, 3]);
        assert_eq!(hi.find(&id!("#.Bbb")).unwrap().copied().collect::<Vec<_>>(), vec![2]);

        let all = hi.iter().map(|(k, v)| (k.clone(), *v)).collect::<Vec<_>>();
        assert!(all.contains(&(id!("#.Aaa"), 1)));
        assert!(all.contains(&(id!("#.Bbb"), 2)));
        assert!(all.contains(&(id!("#.Aaa"), 3)));
    }

    #[test]
    fn test_same_key() {
        pub(crate) struct TestHasher;

        impl Hasher for TestHasher {
            fn finish(&self) -> u64 {
                0
            }

            fn write(&mut self, _: &[u8]) {}
        }

        pub(crate) struct TestState;

        impl BuildHasher for TestState {
            type Hasher = TestHasher;

            fn build_hasher(&self) -> TestHasher {
                TestHasher
            }
        }

        let mut hi: HashIndex<TmplID, u32, TestState> = HashIndex::with_capacity_and_hasher(1, TestState);
        hi.insert(id!("#.Xxx"), 1);
        hi.insert(id!("#.Yyy"), 2);
        hi.insert(id!("#.Yyy"), 3);
        hi.insert(id!("#.Xxx"), 4);
        hi.insert(id!("#.Zzz"), 5);
        hi.insert(id!("#.Zzz"), 5);
        hi.insert(id!("#.Xxx"), 6);

        assert_eq!(hi.find(&id!("#.Xxx")).unwrap().copied().collect::<Vec<_>>(), vec![
            1, 4, 6
        ]);
        assert_eq!(hi.find(&id!("#.Yyy")).unwrap().copied().collect::<Vec<_>>(), vec![2, 3]);
        assert_eq!(hi.find(&id!("#.Zzz")).unwrap().copied().collect::<Vec<_>>(), vec![5, 5]);
    }

    #[test]
    fn test_grow() {
        let mut hi: HashIndex<String, u32> = HashIndex::new();
        assert_eq!(hi.capacity(), 0);
        assert_eq!(hi.len(), 0);

        hi.insert("a".into(), 999);
        hi.insert("a".into(), 998);
        hi.insert("a".into(), 997);
        hi.insert("a".into(), 996);
        assert_eq!(hi.capacity(), 32);
        assert_eq!(hi.len(), 4);

        for i in 0..30 {
            hi.insert(format!("n{}", i), i);
        }
        hi.insert("a".into(), 999);
        hi.insert("a".into(), 995);

        assert_eq!(hi.capacity(), 59);
        assert_eq!(hi.len(), 36);
        assert_eq!(hi.find(&"a".into()).unwrap().copied().collect::<Vec<_>>(), vec![
            999, 998, 997, 996, 999, 995
        ]);
    }
}
