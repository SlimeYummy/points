use enum_iterator::Sequence;
use std::marker::PhantomData;

pub unsafe trait Bitsetable
where
    Self: Copy + PartialEq + Sequence,
{
    const LEN: usize = (Self::CARDINALITY + 7) / 8;
    fn ordinal(&self) -> usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumBitset<E: Bitsetable, const L: usize> {
    bits: [u8; L],
    _phantom: PhantomData<E>,
}

impl<E: Bitsetable, const L: usize> Default for EnumBitset<E, L> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Bitsetable, const L: usize> EnumBitset<E, L> {
    #[inline]
    pub fn new() -> Self {
        debug_assert!(L >= E::LEN);
        Self {
            bits: [0; L],
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn get(&self, insert: E) -> bool {
        let insert = insert.ordinal();
        debug_assert!(insert < E::CARDINALITY);
        self.bits[insert / 8] & (1 << (insert % 8)) != 0
    }

    #[inline]
    pub fn set(&mut self, insert: E, val: bool) {
        let insert = insert.ordinal();
        debug_assert!(insert < E::CARDINALITY);
        if val {
            self.bits[insert / 8] |= 1 << (insert % 8);
        }
        else {
            self.bits[insert / 8] &= !(1 << (insert % 8));
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|v| *v == 0)
    }

    #[inline]
    pub fn iter(&self) -> EnumBitsetIter<E, L> {
        EnumBitsetIter {
            bitset: *self,
            cursor: enum_iterator::first(),
        }
    }

    #[inline]
    pub fn from_slice(list: &[E]) -> EnumBitset<E, L> {
        let mut bitset = EnumBitset::new();
        for item in list {
            bitset.set(*item, true);
        }
        bitset
    }

    #[inline]
    pub fn to_vec(&self) -> Vec<E> {
        let mut list = Vec::new();
        for item in enum_iterator::all() {
            if self.get(item) {
                list.push(item);
            }
        }
        list
    }
}

#[derive(Debug)]
pub struct EnumBitsetIter<E: Bitsetable, const L: usize> {
    bitset: EnumBitset<E, L>,
    cursor: Option<E>,
}

impl<E: Bitsetable, const L: usize> Iterator for EnumBitsetIter<E, L> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        while self.cursor != enum_iterator::last() {
            if self.bitset.get(self.cursor.unwrap()) {
                let value = self.cursor;
                self.cursor = enum_iterator::next(&self.cursor).unwrap();
                return value;
            }
            self.cursor = enum_iterator::next(&self.cursor).unwrap();
        }
        None
    }
}

const _: () = {
    use serde::de::{Deserialize, Deserializer};
    use serde::ser::{Serialize, Serializer};

    impl<E, const L: usize> Serialize for EnumBitset<E, L>
    where
        E: Bitsetable + Serialize,
    {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.to_vec().serialize(serializer)
        }
    }

    impl<'de, E, const L: usize> Deserialize<'de> for EnumBitset<E, L>
    where
        E: Bitsetable + Deserialize<'de>,
    {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<EnumBitset<E, L>, D::Error> {
            let list: Vec<E> = Deserialize::deserialize(deserializer)?;
            Ok(EnumBitset::from_slice(&list))
        }
    }
};

const _: () = {
    use rkyv::rancor::Fallible;
    use rkyv::traits::NoUndef;
    use rkyv::{Archive, Deserialize, Place, Portable, Serialize};

    unsafe impl<E: Bitsetable, const L: usize> NoUndef for EnumBitset<E, L> {}
    unsafe impl<E: Bitsetable, const L: usize> Portable for EnumBitset<E, L> {}

    impl<E: Bitsetable, const L: usize> Archive for EnumBitset<E, L> {
        type Archived = EnumBitset<E, L>;
        type Resolver = ();

        #[inline]
        fn resolve(&self, _: Self::Resolver, out: Place<Self::Archived>) {
            out.write(*self);
        }
    }

    impl<S, E, const L: usize> Serialize<S> for EnumBitset<E, L>
    where
        S: Fallible + ?Sized,
        E: Bitsetable,
    {
        #[inline]
        fn serialize(&self, _: &mut S) -> Result<Self::Resolver, S::Error> {
            Ok(())
        }
    }

    impl<D, E, const L: usize> Deserialize<EnumBitset<E, L>, D> for EnumBitset<E, L>
    where
        D: Fallible + ?Sized,
        E: Bitsetable,
    {
        #[inline]
        fn deserialize(&self, _: &mut D) -> Result<EnumBitset<E, L>, D::Error> {
            Ok(*self)
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_bitset() {}
}
