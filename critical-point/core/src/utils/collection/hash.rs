use ahash::{AHasher, RandomState};
use byteorder::{ByteOrder, NativeEndian};
use std::hash::{BuildHasher, Hasher};

pub(crate) const PRIME_TABLE: [u32; 26] = [
    53,         // 2^5
    97,         // 2^6
    193,        // 2^7
    389,        // 2^8
    769,        // 2^9
    1543,       // 2^10
    3079,       // 2^11
    6151,       // 2^12
    12289,      // 2^13
    24593,      // 2^14
    49157,      // 2^15
    98317,      // 2^16
    196613,     // 2^17
    393241,     // 2^18
    786433,     // 2^19
    1572869,    // 2^20
    3145739,    // 2^21
    6291469,    // 2^22
    12582917,   // 2^23
    25165843,   // 2^24
    50331653,   // 2^25
    100663319,  // 2^26
    201326611,  // 2^27
    402653189,  // 2^28
    805306457,  // 2^29
    1610612741, // 2^30
];

#[derive(Debug, Default)]
pub struct DeterministicState(RandomState);

impl DeterministicState {
    pub const fn new() -> DeterministicState {
        return DeterministicState(RandomState::with_seeds(15668197, 11003, 94686217, 206347));
    }
}

impl BuildHasher for DeterministicState {
    type Hasher = AHasher;

    fn build_hasher(&self) -> Self::Hasher {
        return self.0.build_hasher();
    }
}

/// A deterministic hash map.
pub type DtHashMap<K, V> = std::collections::HashMap<K, V, DeterministicState>;

/// A deterministic hash set.
pub type DtHashSet<K> = std::collections::HashSet<K, DeterministicState>;

#[derive(Debug, Default)]
pub struct IdentityHasher {
    hash: u64,
}

impl Hasher for IdentityHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        if bytes.len() == 8 {
            self.hash = NativeEndian::read_u64(bytes);
        }
    }

    #[inline]
    fn finish(&self) -> u64 {
        return self.hash;
    }
}

#[derive(Debug, Default)]
pub struct IdentityState;

impl IdentityState {
    pub const fn new() -> IdentityState {
        return IdentityState;
    }
}

impl BuildHasher for IdentityState {
    type Hasher = IdentityHasher;

    fn build_hasher(&self) -> IdentityHasher {
        return IdentityHasher::default();
    }
}
