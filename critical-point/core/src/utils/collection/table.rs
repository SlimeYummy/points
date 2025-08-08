use rkyv::Archive;
use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice;

//
// Table KV
//

#[derive(Debug, Default, Clone, Copy, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TableKv<K, V> {
    pub k: K,
    pub v: V,
}

impl<K, V> TableKv<K, V> {
    #[inline]
    pub fn new(k: K, v: V) -> TableKv<K, V> {
        TableKv { k, v }
    }
}

impl<K, V> From<(K, V)> for TableKv<K, V> {
    #[inline]
    fn from(pair: (K, V)) -> TableKv<K, V> {
        TableKv::new(pair.0, pair.1)
    }
}

impl<K, V> From<TableKv<K, V>> for (K, V) {
    #[inline]
    fn from(kv: TableKv<K, V>) -> (K, V) {
        (kv.k, kv.v)
    }
}

const _: () = {
    use serde::de::value::{MapAccessDeserializer, SeqAccessDeserializer};
    use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
    use serde::ser::{Serialize, SerializeTuple, Serializer};
    use serde::Deserialize;

    impl<'de, K, V> Deserialize<'de> for TableKv<K, V>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<TableKv<K, V>, D::Error> {
            deserializer.deserialize_any(TableKvVisitor(PhantomData))
        }
    }

    pub struct TableKvVisitor<'de, K, V>(PhantomData<(&'de (), K, V)>);

    impl<'de, K, V> Visitor<'de> for TableKvVisitor<'de, K, V>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        type Value = TableKv<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(r#"[key, value] or {"key": key, "value": value}"#)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let (k, v) = <(K, V)>::deserialize(SeqAccessDeserializer::new(&mut seq))?;
            Ok(TableKv { k, v })
        }

        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Helper<K, V> {
                k: K,
                v: V,
            }
            let Helper { k, v } = Helper::deserialize(MapAccessDeserializer::new(map))?;
            Ok(TableKv { k, v })
        }
    }

    impl<K: Serialize, V: Serialize> Serialize for TableKv<K, V> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut seq = serializer.serialize_tuple(2)?;
            seq.serialize_element(&self.k)?;
            seq.serialize_element(&self.v)?;
            seq.end()
        }
    }
};

impl<K: Archive, V: Archive> From<ArchivedTableKv<K, V>> for (K::Archived, V::Archived) {
    #[inline]
    fn from(kv: ArchivedTableKv<K, V>) -> (K::Archived, V::Archived) {
        (kv.k, kv.v)
    }
}

impl<K, V> fmt::Debug for ArchivedTableKv<K, V>
where
    K: rkyv::Archive,
    V: rkyv::Archive,
    K::Archived: Debug,
    V::Archived: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArchivedTableKv")
            .field("k", &self.k)
            .field("v", &self.v)
            .finish()
    }
}

//
// Table
//

#[derive(PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Table<K, V>(Vec<TableKv<K, V>>);

impl<K, V> Default for Table<K, V> {
    #[inline]
    fn default() -> Self {
        Table(Vec::new())
    }
}

impl<K, V> Deref for Table<K, V> {
    type Target = Vec<TableKv<K, V>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> DerefMut for Table<K, V> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K, V> Table<K, V> {
    #[inline]
    pub fn new() -> Table<K, V> {
        Table(Vec::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Table<K, V> {
        Table(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn key(&self, idx: usize) -> Option<&K> {
        self.0.get(idx).map(|kv| &kv.k)
    }

    #[inline]
    pub fn key_mut(&mut self, idx: usize) -> Option<&mut K> {
        self.0.get_mut(idx).map(|kv| &mut kv.k)
    }

    #[inline]
    pub fn key_x(&self, idx: usize) -> &K {
        self.0.get(idx).map(|kv| &kv.k).unwrap()
    }

    #[inline]
    pub fn key_x_mut(&mut self, idx: usize) -> &mut K {
        self.0.get_mut(idx).map(|kv| &mut kv.k).unwrap()
    }

    #[inline]
    pub fn value(&self, idx: usize) -> Option<&V> {
        self.0.get(idx).map(|kv| &kv.v)
    }

    #[inline]
    pub fn value_mut(&mut self, idx: usize) -> Option<&mut V> {
        self.0.get_mut(idx).map(|kv| &mut kv.v)
    }

    #[inline]
    pub fn value_x(&self, idx: usize) -> &V {
        self.0.get(idx).map(|kv| &kv.v).unwrap()
    }

    #[inline]
    pub fn value_x_mut(&mut self, idx: usize) -> &mut V {
        self.0.get_mut(idx).map(|kv| &mut kv.v).unwrap()
    }

    #[inline]
    pub fn push2(&mut self, k: K, v: V) {
        self.0.push(TableKv::new(k, v));
    }

    #[inline]
    pub fn insert2(&mut self, idx: usize, k: K, v: V) {
        self.0.insert(idx, TableKv::new(k, v));
    }
}

impl<K, V> Index<usize> for Table<K, V> {
    type Output = TableKv<K, V>;

    #[inline]
    fn index(&self, idx: usize) -> &TableKv<K, V> {
        self.0.get(idx).unwrap()
    }
}

impl<K, V> IndexMut<usize> for Table<K, V> {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut TableKv<K, V> {
        self.0.get_mut(idx).unwrap()
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Table<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<K, V> Table<K, V> {
    #[inline]
    pub fn keys(&self) -> TableKeysIter<'_, K, V> {
        TableKeysIter(self.0.iter())
    }

    #[inline]
    pub fn keys_mut(&mut self) -> TableKeysIterMut<'_, K, V> {
        TableKeysIterMut(self.0.iter_mut())
    }

    #[inline]
    pub fn values(&self) -> TableValuesIter<'_, K, V> {
        TableValuesIter(self.0.iter())
    }

    #[inline]
    pub fn values_mut(&mut self) -> TableValuesIterMut<'_, K, V> {
        TableValuesIterMut(self.0.iter_mut())
    }
}

pub struct TableKeysIter<'t, K, V>(slice::Iter<'t, TableKv<K, V>>);

impl<'t, K, V> Iterator for TableKeysIter<'t, K, V> {
    type Item = &'t K;

    #[inline]
    fn next(&mut self) -> Option<&'t K> {
        self.0.next().map(|kv| &kv.k)
    }
}

pub struct TableKeysIterMut<'t, K, V>(slice::IterMut<'t, TableKv<K, V>>);

impl<'t, K, V> Iterator for TableKeysIterMut<'t, K, V> {
    type Item = &'t mut K;

    #[inline]
    fn next(&mut self) -> Option<&'t mut K> {
        self.0.next().map(|kv| &mut kv.k)
    }
}

pub struct TableValuesIter<'t, K, V>(slice::Iter<'t, TableKv<K, V>>);

impl<'t, K, V> Iterator for TableValuesIter<'t, K, V> {
    type Item = &'t V;

    #[inline]
    fn next(&mut self) -> Option<&'t V> {
        self.0.next().map(|kv| &kv.v)
    }
}

pub struct TableValuesIterMut<'t, K, V>(slice::IterMut<'t, TableKv<K, V>>);

impl<'t, K, V> Iterator for TableValuesIterMut<'t, K, V> {
    type Item = &'t mut V;

    #[inline]
    fn next(&mut self) -> Option<&'t mut V> {
        self.0.next().map(|kv| &mut kv.v)
    }
}

impl<K: PartialEq, V> Table<K, V> {
    #[inline]
    pub fn find(&self, key: &K) -> Option<&V> {
        self.0.iter().find(|kv| kv.k == *key).map(|kv| &kv.v)
    }

    #[inline]
    pub fn find_mut(&mut self, key: &K) -> Option<&mut V> {
        self.0.iter_mut().find(|kv| kv.k == *key).map(|kv| &mut kv.v)
    }

    #[inline]
    pub fn index_of(&self, key: &K) -> Option<usize> {
        self.0.iter().position(|kv| kv.k == *key)
    }

    #[inline]
    pub fn find_kv(&self, key: &K) -> Option<&TableKv<K, V>> {
        self.0.iter().find(|kv| kv.k == *key)
    }

    #[inline]
    pub fn find_kv_mut(&mut self, key: &K) -> Option<&mut TableKv<K, V>> {
        self.0.iter_mut().find(|kv| kv.k == *key)
    }
}

const _: () = {
    use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
    use serde::ser::{Serialize, Serializer};

    impl<'de, K, V> Deserialize<'de> for Table<K, V>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_any(TableVisitor(PhantomData))
        }
    }

    struct TableVisitor<K, V>(PhantomData<(K, V)>);

    impl<'de, K, V> Visitor<'de> for TableVisitor<K, V>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        type Value = Table<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(r#"expecting {"key":value, ...} or [[key,value], ...]"#)
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Table<K, V>, A::Error> {
            let mut table = Table::with_capacity(map.size_hint().unwrap_or(0));
            while let Some((key, val)) = map.next_entry()? {
                table.push2(key, val);
            }
            Ok(table)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Table<K, V>, A::Error> {
            let mut table = Table::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some((key, val)) = seq.next_element()? {
                table.push2(key, val);
            }
            Ok(table)
        }
    }

    impl<K: Serialize, V: Serialize> Serialize for Table<K, V> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.0.serialize(serializer)
        }
    }
};

//
// Archived Table
//

impl<K: Archive, V: Archive> Deref for ArchivedTable<K, V> {
    type Target = rkyv::vec::ArchivedVec<ArchivedTableKv<K, V>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K: Archive, V: Archive> ArchivedTable<K, V> {
    #[inline]
    pub fn key(&self, idx: usize) -> Option<&K::Archived> {
        self.0.get(idx).map(|kv| &kv.k)
    }

    #[inline]
    pub fn key_x(&self, idx: usize) -> &K::Archived {
        self.0.get(idx).map(|kv| &kv.k).unwrap()
    }

    #[inline]
    pub fn value(&self, idx: usize) -> Option<&V::Archived> {
        self.0.get(idx).map(|kv| &kv.v)
    }

    #[inline]
    pub fn value_x(&self, idx: usize) -> &V::Archived {
        self.0.get(idx).map(|kv| &kv.v).unwrap()
    }
}

impl<K: Archive, V: Archive> Index<usize> for ArchivedTable<K, V> {
    type Output = ArchivedTableKv<K, V>;

    #[inline]
    fn index(&self, idx: usize) -> &ArchivedTableKv<K, V> {
        self.0.get(idx).unwrap()
    }
}

impl<K, V> fmt::Debug for ArchivedTable<K, V>
where
    K: rkyv::Archive,
    V: rkyv::Archive,
    K::Archived: Debug,
    V::Archived: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<K: Archive, V: Archive> ArchivedTable<K, V> {
    #[inline]
    pub fn keys(&self) -> ArchivedTableKeysIter<'_, K, V> {
        ArchivedTableKeysIter(self.0.iter())
    }

    #[inline]
    pub fn values(&self) -> ArchivedTableValuesIter<'_, K, V> {
        ArchivedTableValuesIter(self.0.iter())
    }
}

pub struct ArchivedTableKeysIter<'t, K: Archive, V: Archive>(slice::Iter<'t, ArchivedTableKv<K, V>>);

impl<'t, K: Archive, V: Archive> Iterator for ArchivedTableKeysIter<'t, K, V> {
    type Item = &'t K::Archived;

    #[inline]
    fn next(&mut self) -> Option<&'t K::Archived> {
        self.0.next().map(|kv| &kv.k)
    }
}

pub struct ArchivedTableValuesIter<'t, K: Archive, V: Archive>(slice::Iter<'t, ArchivedTableKv<K, V>>);

impl<'t, K: Archive, V: Archive> Iterator for ArchivedTableValuesIter<'t, K, V> {
    type Item = &'t V::Archived;

    #[inline]
    fn next(&mut self) -> Option<&'t V::Archived> {
        self.0.next().map(|kv| &kv.v)
    }
}

impl<K, V> ArchivedTable<K, V>
where
    K: rkyv::Archive,
    V: rkyv::Archive,
    K::Archived: PartialEq,
{
    #[inline]
    pub fn find(&self, key: &K::Archived) -> Option<&V::Archived> {
        self.0.iter().find(|kv| kv.k == *key).map(|kv| &kv.v)
    }

    #[inline]
    pub fn index_of(&self, key: &K::Archived) -> Option<usize> {
        self.0.iter().position(|kv| kv.k == *key)
    }

    #[inline]
    pub fn find_kv(&self, key: &K::Archived) -> Option<&ArchivedTableKv<K, V>> {
        self.0.iter().find(|kv| kv.k == *key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_serde() {
        use serde_json;

        let json = r#"[
            [123, [5.0, 6.0, 7.0]],
            [789, [12.0, 13.0]]
        ]"#;
        let tb1: Table<u16, Vec<f64>> = serde_json::from_str(json).unwrap();
        assert_eq!(tb1.len(), 2);
        assert_eq!(tb1.keys().copied().collect::<Vec<u16>>(), vec![123, 789]);
        assert_eq!(tb1.values().map(|x| x.to_vec()).collect::<Vec<Vec<f64>>>(), vec![
            vec![5.0, 6.0, 7.0],
            vec![12.0, 13.0]
        ]);
        let text = serde_json::to_string(&tb1).unwrap();
        let tb1x: Table<u16, Vec<f64>> = serde_json::from_str(&text).unwrap();
        assert_eq!(tb1x, tb1);

        let json = r#"{
            "k1": ["aaa", "bbb"],
            "k2": ["xx"],
            "k3": ["xx", "yy"]
        }"#;
        let tb2: Table<String, Vec<String>> = serde_json::from_str(json).unwrap();
        assert_eq!(tb2.len(), 3);
        assert_eq!(tb2.keys().cloned().collect::<Vec<String>>(), vec![
            "k1".to_string(),
            "k2".to_string(),
            "k3".to_string()
        ]);
        assert_eq!(tb2.values().map(|x| x.to_vec()).collect::<Vec<Vec<String>>>(), vec![
            vec!["aaa".to_string(), "bbb".to_string()],
            vec!["xx".to_string()],
            vec!["xx".to_string(), "yy".to_string()]
        ]);
        let text = serde_json::to_string(&tb2).unwrap();
        let tb2x: Table<String, Vec<String>> = serde_json::from_str(&text).unwrap();
        assert_eq!(tb2x, tb2);
    }

    #[test]
    fn test_table_rkyv() {
        use rkyv::rancor::Error;

        let tb1: Table<u8, String> = Table::default();
        let buf = rkyv::to_bytes::<Error>(&tb1).unwrap();
        let archived1 = unsafe { rkyv::access_unchecked::<ArchivedTable<u8, String>>(&buf) };
        let tb1x: Table<u8, String> = rkyv::deserialize::<_, Error>(archived1).unwrap();
        assert_eq!(tb1, tb1x);

        let mut tb2: Table<u64, [f32; 2]> = Table::new();
        tb2.push2(0, [1.0, 2.0]);
        tb2.push2(0, [3.0, 4.0]);
        tb2.push2(0, [5.0, 6.0]);
        let buf = rkyv::to_bytes::<Error>(&tb2).unwrap();
        let archived2 = unsafe { rkyv::access_unchecked::<ArchivedTable<u64, [f32; 2]>>(&buf) };
        let tb2x: Table<u64, [f32; 2]> = rkyv::deserialize::<_, Error>(archived2).unwrap();
        assert_eq!(tb2, tb2x);
    }
}
