use super::list::*;
use std::fmt;
use std::ops::Index;

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct KvList<K, V>(List<(K, V)>);

impl<K, V> Default for KvList<K, V> {
    fn default() -> Self {
        return KvList(List::default());
    }
}

impl<K, V> KvList<K, V> {
    #[inline]
    pub fn len(&self) -> usize {
        return self.0.len();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        return self.0.is_empty();
    }

    #[inline]
    pub fn key(&self, idx: usize) -> Option<&K> {
        return self.get(idx).map(|(k, _)| k);
    }

    #[inline]
    pub fn xkey(&self, idx: usize) -> &K {
        return self.key(idx).expect("index out of bounds");
    }

    #[inline]
    pub fn value(&self, idx: usize) -> Option<&V> {
        return self.get(idx).map(|(_, v)| v);
    }

    #[inline]
    pub fn xvalue(&self, idx: usize) -> &V {
        return self.value(idx).expect("index out of bounds");
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Option<&(K, V)> {
        return self.0.get(idx);
    }

    #[inline]
    pub fn xget(&self, idx: usize) -> &(K, V) {
        return self.get(idx).expect("index out of bounds");
    }

    #[inline]
    pub fn as_slice(&self) -> &[(K, V)] {
        return self.0.as_slice();
    }

    #[inline]
    pub fn iter(&self) -> ListIter<'_, (K, V)> {
        return self.0.iter();
    }

    #[inline]
    pub fn key_iter(&self) -> KvListKeyIter<'_, K, V> {
        return KvListKeyIter { list: self, cursor: 0 };
    }

    #[inline]
    pub fn value_iter(&self) -> KvListValueIter<'_, K, V> {
        return KvListValueIter { list: self, cursor: 0 };
    }

    #[inline]
    pub fn find(&self, key: &K) -> Option<&V>
    where
        K: PartialEq,
    {
        return self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v);
    }
}

impl<K, V> Index<usize> for KvList<K, V> {
    type Output = (K, V);

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        return self.get(idx).expect("index out of bounds");
    }
}

pub struct KvListKeyIter<'t, K, V> {
    list: &'t KvList<K, V>,
    cursor: usize,
}

impl<'t, K, V> Iterator for KvListKeyIter<'t, K, V> {
    type Item = &'t K;

    fn next(&mut self) -> Option<&'t K> {
        let value = self.list.key(self.cursor);
        self.cursor += 1;
        return value;
    }
}

pub struct KvListValueIter<'t, K, V> {
    list: &'t KvList<K, V>,
    cursor: usize,
}

impl<'t, K, V> Iterator for KvListValueIter<'t, K, V> {
    type Item = &'t V;

    fn next(&mut self) -> Option<&'t V> {
        let value = self.list.value(self.cursor);
        self.cursor += 1;
        return value;
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for KvList<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = f.debug_list();
        for pair in self.iter() {
            out.entry(&pair);
        }
        return out.finish();
    }
}

impl<'t, K, V> IntoIterator for &'t KvList<K, V> {
    type Item = &'t (K, V);
    type IntoIter = ListIter<'t, (K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        return self.iter();
    }
}

//
// serde
//

const _: () = {
    use serde::de::{DeserializeOwned, Deserializer, MapAccess, SeqAccess, Visitor};
    use serde::Deserialize;
    use std::marker::PhantomData;

    impl<'de, K: DeserializeOwned, V: DeserializeOwned> Deserialize<'de> for KvList<K, V> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            return deserializer.deserialize_any(KvListVisitor(PhantomData));
        }
    }

    struct KvListVisitor<K, V>(PhantomData<(K, V)>);

    impl<'de, K: DeserializeOwned, V: DeserializeOwned> Visitor<'de> for KvListVisitor<K, V> {
        type Value = KvList<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            return formatter.write_str(r#"expecting {"key":value} or [{"k":key,"v":value}]"#);
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<KvList<K, V>, A::Error> {
            let mut vec = Vec::new();
            while let Some(key) = map.next_key()? {
                vec.push((key, map.next_value()?));
            }

            let mut list = List::alloc(vec.len());
            for (i, elem) in vec.into_iter().enumerate() {
                unsafe { list.init(i, elem) };
            }
            return Ok(KvList(list));
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<KvList<K, V>, A::Error> {
            #[derive(Deserialize)]
            struct Helper<K, V> {
                k: K,
                v: V,
            }

            let mut vec = Vec::new();
            while let Some(Helper { k, v }) = seq.next_element()? {
                vec.push((k, v));
            }

            let mut list = List::alloc(vec.len());
            for (i, elem) in vec.into_iter().enumerate() {
                unsafe { list.init(i, elem) };
            }
            return Ok(KvList(list));
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{s, Symbol};

    #[test]
    fn test_kvlist_basic() {
        let mut lt = KvList(List::alloc(3));
        unsafe {
            lt.0.init(0, (s!("xx"), 10));
            lt.0.init(1, (s!("yy"), 20));
            lt.0.init(2, (s!("zz"), 30));
        };
        assert_eq!(lt.len(), 3);
        assert_eq!(
            lt.key_iter().map(|x| x.clone()).collect::<Vec<Symbol>>(),
            vec![s!("xx"), s!("yy"), s!("zz")]
        );
        assert_eq!(lt.value_iter().map(|x| *x).collect::<Vec<i32>>(), vec![10, 20, 30]);
    }

    #[test]
    fn test_kvlist_serde() {
        use serde_json;

        let lt1: KvList<String, f64> = serde_json::from_str("[]").unwrap();
        assert_eq!(lt1.as_slice(), &[] as &[(String, f64)]);

        let json = r#"{
            "k1": [1,2,3],
            "k2": [4,5,6]
        }"#;
        let lt2: KvList<Symbol, [u8; 3]> = serde_json::from_str(json).unwrap();
        assert_eq!(
            lt2.as_slice(),
            &[(s!("k1"), [1u8, 2u8, 3u8]), (s!("k2"), [4u8, 5u8, 6u8])]
        );

        let json = r#"[
            {"k":234, "v":"abc"},
            {"k":345, "v":"def"},
            {"k":987, "v":"def"}
        ]"#;
        let lt3: KvList<u32, Symbol> = serde_json::from_str(json).unwrap();
        assert_eq!(lt3.as_slice(), &[(234, s!("abc")), (345, s!("def")), (987, s!("def"))]);
    }
}
