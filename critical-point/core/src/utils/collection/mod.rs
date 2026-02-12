mod algorithm;
mod enum_bitset;
mod hash_index;
mod history_queue;
mod history_vec;
mod prime_table;
mod table;

pub use algorithm::*;
pub use enum_bitset::*;
pub use hash_index::*;
pub use history_queue::*;
pub use history_vec::*;
pub(crate) use prime_table::*;
pub use table::*;

pub use arrayvec::{
    ArrayString, ArrayVec, CapacityError as ArrayVecCapacityError, Drain as ArrayVecDrain, IntoIter as ArrayVecIntoIter,
};
pub use smallvec::{smallvec, Drain as SmallVecDrain, IntoIter as SmallVecIntoIter, SmallVec, ToSmallVec};
pub use thin_vec::{thin_vec, Drain as ThinVecDrain, IntoIter as ThinVecIntoIter, Splice as ThinVecSplice, ThinVec};

/// A deterministic hash map.
pub type DtHashMap<K, V> = rustc_hash::FxHashMap<K, V>;

/// A deterministic hash set.
pub type DtHashSet<K> = rustc_hash::FxHashSet<K>;
