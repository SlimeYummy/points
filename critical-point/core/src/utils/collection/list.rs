use std::alloc::Layout;
use std::marker::PhantomData;
use std::ops::Index;
use std::{alloc, fmt, ptr, slice};

pub struct List<V> {
    len: u32,
    data: *mut V,
}

impl<V> Default for List<V> {
    fn default() -> Self {
        return List::alloc(0);
    }
}

impl<V> Drop for List<V> {
    fn drop(&mut self) {
        for i in 0..self.len() {
            unsafe { ptr::drop_in_place(self.data.add(i)) };
        }
        let data = self.data as *mut u8;
        unsafe { alloc::dealloc(data, Layout::array::<V>(self.len()).unwrap()) };
        self.data = ptr::null_mut();
    }
}

impl<V> List<V> {
    #[inline]
    pub(super) fn alloc(len: usize) -> List<V> {
        let data = unsafe { alloc::alloc(Layout::array::<V>(len).unwrap()) };
        return List {
            len: len as u32,
            data: data as *mut V,
        };
    }

    #[inline]
    pub(super) unsafe fn init(&mut self, idx: usize, val: V) {
        self.data.add(idx).write(val);
    }

    #[inline]
    pub fn len(&self) -> usize {
        return self.len as usize;
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        return self.len() == 0;
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Option<&V> {
        if idx < self.len() {
            return Some(unsafe { &*self.data.add(idx) });
        }
        return None;
    }

    #[inline]
    pub fn xget(&self, idx: usize) -> &V {
        return self.get(idx).expect("index out of bounds");
    }

    #[inline]
    pub fn as_slice(&self) -> &[V] {
        return unsafe { slice::from_raw_parts(self.data, self.len()) };
    }

    #[inline]
    pub fn iter(&self) -> ListIter<V> {
        return ListIter {
            list: self,
            cursor: 0,
        };
    }

    #[inline]
    pub fn contains(&self, value: &V) -> bool
    where
        V: PartialEq,
    {
        return self.iter().any(|x| x == value);
    }
}

impl<V> Index<usize> for List<V> {
    type Output = V;

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        return self.get(idx).expect("index out of bounds");
    }
}

#[derive(Debug)]
pub struct ListIter<'t, V> {
    list: &'t List<V>,
    cursor: usize,
}

impl<'t, V> Iterator for ListIter<'t, V> {
    type Item = &'t V;

    fn next(&mut self) -> Option<&'t V> {
        let value = self.list.get(self.cursor);
        self.cursor += 1;
        return value;
    }
}

impl<L: fmt::Debug> fmt::Debug for List<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return f.debug_list().entries(self.iter()).finish();
    }
}

//
// serde
//

const _: () = {
    use serde::de::{Deserialize, DeserializeOwned, Deserializer, SeqAccess, Visitor};

    impl<'de, V: DeserializeOwned> Deserialize<'de> for List<V> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            return deserializer.deserialize_seq(ListVisitor(PhantomData));
        }
    }

    struct ListVisitor<V>(PhantomData<V>);

    impl<'de, V: DeserializeOwned> Visitor<'de> for ListVisitor<V> {
        type Value = List<V>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            return formatter.write_str("expecting [...]");
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<List<V>, A::Error> {
            let mut vec = Vec::new();
            while let Some(elem) = seq.next_element::<V>()? {
                vec.push(elem);
            }

            let mut list = List::alloc(vec.len());
            for (i, elem) in vec.into_iter().enumerate() {
                unsafe { list.init(i, elem) };
            }
            return Ok(list);
        }
    }
};

//
// rkyv
//

pub type ArchivedList<V> = rkyv::vec::ArchivedVec<rkyv::Archived<V>>;

const _: () = {
    use rkyv::ser::{ScratchSpace, Serializer};
    use rkyv::vec::{ArchivedVec, VecResolver};
    use rkyv::{Archive, Archived, Deserialize, Fallible, Serialize};

    impl<V: Archive> Archive for List<V> {
        type Archived = ArchivedList<V>;
        type Resolver = VecResolver;

        unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
            ArchivedVec::resolve_from_slice(self.as_slice(), pos, resolver, out);
        }
    }

    impl<S, V> Serialize<S> for List<V>
    where
        S: Serializer + ScratchSpace + ?Sized,
        V: Serialize<S>,
    {
        fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
            return ArchivedVec::serialize_from_slice(self.as_slice(), serializer);
        }
    }

    impl<D, V> Deserialize<List<V>, D> for ArchivedList<V>
    where
        D: Fallible + ?Sized,
        V: Archive,
        Archived<V>: Deserialize<V, D>,
    {
        fn deserialize(&self, deserializer: &mut D) -> Result<List<V>, D::Error> {
            let mut list = List::alloc(self.len());
            for (idx, archived) in self.iter().enumerate() {
                let value: V = archived.deserialize(deserializer)?;
                unsafe { list.init(idx, value) };
            }
            return Ok(list);
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{s, Symbol};
    use anyhow::Result;
    use std::mem;

    #[test]
    fn test_list_basic() {
        let lt1: List<i32> = List::alloc(0);
        assert_eq!(lt1.len(), 0);
        assert_eq!(lt1.get(0), None);
        assert_eq!(lt1.as_slice(), &[] as &[i32]);

        let mut lt2 = List::alloc(3);
        unsafe {
            lt2.init(0, 10);
            lt2.init(1, 20);
            lt2.init(2, 30);
        };
        assert_eq!(lt2.len(), 3);
        assert_eq!(lt2.get(0), Some(&10));
        assert_eq!(lt2.get(1), Some(&20));
        assert_eq!(lt2.get(2), Some(&30));
        assert_eq!(lt2.get(3), None);
        assert_eq!(lt2.as_slice(), &[10, 20, 30]);
        assert_eq!(
            lt2.iter().map(|x| *x).collect::<Vec<i32>>(),
            vec![10, 20, 30]
        );

        let mut lt3 = List::alloc(2);
        let s1 = s!("abc");
        let s2 = s!("def");
        unsafe {
            lt3.init(0, s1.clone());
            lt3.init(1, s2.clone());
        }
        assert_eq!(s1.ref_count(), 2);
        assert_eq!(s2.ref_count(), 2);
        assert_eq!(lt3[0], s1);
        assert_eq!(*lt3.xget(1), s2);
        mem::drop(lt3);
        assert_eq!(s1.ref_count(), 1);
        assert_eq!(s2.ref_count(), 1);
    }

    #[test]
    fn test_list_serde() {
        use serde_json;

        let lt1: List<f64> = serde_json::from_str("[]").unwrap();
        assert_eq!(lt1.as_slice(), &[] as &[f64]);

        let lt2: List<[u8; 3]> = serde_json::from_str("[[1,2,3],[4,5,6]]").unwrap();
        assert_eq!(lt2.as_slice(), &[[1u8, 2u8, 3u8], [4u8, 5u8, 6u8]]);

        let lt3: List<Symbol> = serde_json::from_str(r#"["abc","def","def"]"#).unwrap();
        assert_eq!(lt3.as_slice(), &[s!("abc"), s!("def"), s!("def")]);
    }

    #[test]
    fn test_list_rkyv() {
        use rkyv::ser::serializers::AllocSerializer;
        use rkyv::ser::Serializer;
        use rkyv::{Deserialize, Infallible, Serialize};

        fn test_rkyv<V>(list: List<V>) -> Result<()>
        where
            V: PartialEq + Serialize<AllocSerializer<4096>>,
            V::Archived: Deserialize<V, Infallible>,
        {
            let mut serializer = rkyv::ser::serializers::AllocSerializer::<4096>::default();
            serializer.serialize_value(&list)?;
            let buffer = serializer.into_serializer().into_inner();
            let archived = unsafe { rkyv::archived_root::<List<V>>(&buffer) };
            let mut deserializer = rkyv::Infallible::default();
            let result: List<V> = archived.deserialize(&mut deserializer)?;
            if list.as_slice() != result.as_slice() {
                return Err(anyhow::anyhow!("rkyv test not equal"));
            }
            return Ok(());
        }

        let lt1: List<u32> = List::alloc(0);
        test_rkyv(lt1).unwrap();

        let mut lt2 = List::alloc(3);
        unsafe {
            lt2.init(0, 10);
            lt2.init(1, 20);
            lt2.init(2, 30);
        };
        test_rkyv(lt2).unwrap();

        let mut lt3 = List::alloc(2);
        unsafe {
            lt3.init(0, s!("abc"));
            lt3.init(1, s!("def"));
        }
        test_rkyv(lt3).unwrap();
    }
}
