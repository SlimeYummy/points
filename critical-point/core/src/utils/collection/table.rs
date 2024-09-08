use std::alloc::Layout;
use std::marker::PhantomData;
use std::ops::Index;
use std::{alloc, fmt, mem, ptr, slice};

#[derive(Debug, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
struct Header<K> {
    offset: u32,
    key: K,
}

pub struct Table<K, V> {
    header_len: u32,
    value_len: u32,
    headers: *mut Header<K>,
    values: *mut V,
}

impl<K, V> Default for Table<K, V> {
    fn default() -> Self {
        Table::alloc(0, 0)
    }
}

impl<K, V> Drop for Table<K, V> {
    fn drop(&mut self) {
        for i in 0..self.header_len {
            unsafe { ptr::drop_in_place(self.headers.add(i as usize)) };
        }
        for i in 0..self.value_len {
            unsafe { ptr::drop_in_place(self.values.add(i as usize)) };
        }
        let data = self.headers as *mut u8;
        let (size, _, align) = Table::<K, V>::size(self.header_len as usize, self.value_len as usize);
        unsafe { alloc::dealloc(data, Layout::from_size_align(size, align).unwrap()) };
        self.headers = ptr::null_mut();
        self.values = ptr::null_mut();
    }
}

impl<K, V> Table<K, V> {
    #[inline]
    fn size(header_len: usize, value_len: usize) -> (usize, usize, usize) {
        let header_size = mem::size_of::<Header<K>>() * header_len;
        let header_align = mem::align_of::<Header<K>>();

        let value_size = mem::size_of::<V>() * value_len;
        let value_align = mem::align_of::<V>();
        let value_mask = value_align - 1;

        let offset = (header_size + value_mask) & !(value_mask);
        let size = offset + value_size;
        let align = usize::max(header_align, value_align);
        (size, offset, align)
    }

    fn alloc(header_len: usize, value_len: usize) -> Table<K, V> {
        let (size, offset, align) = Table::<K, V>::size(header_len, value_len);
        let data = unsafe { alloc::alloc(Layout::from_size_align(size, align).unwrap()) };
        Table {
            header_len: header_len as u32,
            value_len: value_len as u32,
            headers: data as *mut Header<K>,
            values: unsafe { data.add(offset) as *mut V },
        }
    }

    #[inline]
    unsafe fn init_header(&mut self, idx: usize, key: K, offset: usize) {
        self.headers.add(idx).write(Header {
            offset: u32::min(offset as u32, self.value_len),
            key,
        });
    }

    #[inline]
    unsafe fn init_value(&mut self, idx: usize, val: V) {
        self.values.add(idx).write(val);
    }

    #[inline]
    fn headers_buf(&self) -> &[Header<K>] {
        return unsafe { slice::from_raw_parts(self.headers, self.header_len as usize) };
    }

    #[inline]
    fn values_buf(&self) -> &[V] {
        return unsafe { slice::from_raw_parts(self.values, self.value_len as usize) };
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.header_len as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn key(&self, idx: usize) -> Option<&K> {
        if idx < self.len() {
            return Some(unsafe { &(*self.headers.add(idx)).key });
        }
        None
    }

    #[inline]
    pub fn xkey(&self, idx: usize) -> &K {
        return self.key(idx).expect("index out of bounds");
    }

    #[inline]
    pub fn values(&self, idx: usize) -> Option<&[V]> {
        if idx < self.len() {
            let header = unsafe { &(*self.headers.add(idx)) };
            let start = header.offset as usize;
            let end = if idx + 1 < self.header_len as usize {
                unsafe { (*self.headers.add(idx + 1)).offset }
            } else {
                self.value_len
            } as usize;
            return Some(unsafe { slice::from_raw_parts(self.values.add(start), end - start) });
        }
        None
    }

    #[inline]
    pub fn xvalues(&self, idx: usize) -> &[V] {
        return self.values(idx).expect("index out of bounds");
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Option<(&K, &[V])> {
        if idx < self.len() {
            let header = unsafe { &(*self.headers.add(idx)) };
            let start = header.offset as usize;
            let end = if idx + 1 < self.header_len as usize {
                unsafe { (*self.headers.add(idx + 1)).offset }
            } else {
                self.value_len
            } as usize;
            let values = unsafe { slice::from_raw_parts(self.values.add(start), end - start) };
            return Some((&header.key, values));
        }
        None
    }

    #[inline]
    pub fn xget(&self, idx: usize) -> (&K, &[V]) {
        return self.get(idx).expect("index out of bounds");
    }

    #[inline]
    pub fn iter(&self) -> TableIter<'_, K, V> {
        TableIter { table: self, cursor: 0 }
    }

    #[inline]
    pub fn key_iter(&self) -> TableKeyIter<'_, K, V> {
        TableKeyIter { table: self, cursor: 0 }
    }

    #[inline]
    pub fn values_iter(&self) -> TableValuesIter<'_, K, V> {
        TableValuesIter { table: self, cursor: 0 }
    }

    #[inline]
    pub fn find(&self, key: &K) -> Option<&[V]>
    where
        K: PartialEq,
    {
        return self.iter().find(|(k, _)| *k == key).map(|(_, v)| v);
    }
}

impl<K, V> Index<usize> for Table<K, V> {
    type Output = [V];

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        return self.values(idx).expect("index out of bounds");
    }
}

pub struct TableIter<'t, K, V> {
    table: &'t Table<K, V>,
    cursor: usize,
}

impl<'t, K, V> Iterator for TableIter<'t, K, V> {
    type Item = (&'t K, &'t [V]);

    fn next(&mut self) -> Option<(&'t K, &'t [V])> {
        let value = self.table.get(self.cursor);
        self.cursor += 1;
        value
    }
}

pub struct TableKeyIter<'t, K, V> {
    table: &'t Table<K, V>,
    cursor: usize,
}

impl<'t, K, V> Iterator for TableKeyIter<'t, K, V> {
    type Item = &'t K;

    fn next(&mut self) -> Option<&'t K> {
        let value = self.table.key(self.cursor);
        self.cursor += 1;
        value
    }
}

pub struct TableValuesIter<'t, K, V> {
    table: &'t Table<K, V>,
    cursor: usize,
}

impl<'t, K, V> Iterator for TableValuesIter<'t, K, V> {
    type Item = &'t [V];

    fn next(&mut self) -> Option<&'t [V]> {
        let value = self.table.values(self.cursor);
        self.cursor += 1;
        value
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Table<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = f.debug_map();
        for (key, values) in self.iter() {
            out.key(key);
            out.value(&values);
        }
        out.finish()
    }
}

impl<'t, K, V> IntoIterator for &'t Table<K, V> {
    type Item = (&'t K, &'t [V]);
    type IntoIter = TableIter<'t, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        return self.iter();
    }
}

//
// serde
//

const _: () = {
    use serde::de::{
        Deserialize, DeserializeOwned, DeserializeSeed, Deserializer, Error, MapAccess, SeqAccess, Visitor,
    };

    impl<'de, K: DeserializeOwned, V: DeserializeOwned> Deserialize<'de> for Table<K, V> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            return deserializer.deserialize_any(TableVisitor(PhantomData));
        }
    }

    struct TableVisitor<K, V>(PhantomData<(K, V)>);

    impl<'de, K: DeserializeOwned, V: DeserializeOwned> Visitor<'de> for TableVisitor<K, V> {
        type Value = Table<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(r#"expecting {"key":[value, ...]} or [{"k":key,"v":[value, ...]}]"#)
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Table<K, V>, A::Error> {
            let mut headers = Vec::new();
            let mut values = Vec::new();

            while let Some(key) = map.next_key()? {
                headers.push(Header {
                    key,
                    offset: values.len() as u32,
                });
                map.next_value_seed(VecVisitor(&mut values))?;
            }

            let mut table = Table::alloc(headers.len(), values.len());
            for (idx, header) in headers.into_iter().enumerate() {
                unsafe { table.init_header(idx, header.key, header.offset as usize) };
            }
            for (idx, value) in values.into_iter().enumerate() {
                unsafe { table.init_value(idx, value) };
            }
            Ok(table)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Table<K, V>, A::Error> {
            let mut headers = Vec::new();
            let mut values = Vec::new();

            loop {
                let offset = values.len() as u32;
                let key = seq.next_element_seed(SeqItemVisitor {
                    values: &mut values,
                    _phantom: PhantomData,
                })?;
                if key.is_none() {
                    break;
                }
                headers.push(Header {
                    key: key.unwrap(),
                    offset,
                });
            }

            let mut table = Table::alloc(headers.len(), values.len());
            for (idx, header) in headers.into_iter().enumerate() {
                unsafe { table.init_header(idx, header.key, header.offset as usize) };
            }
            for (idx, value) in values.into_iter().enumerate() {
                unsafe { table.init_value(idx, value) };
            }
            Ok(table)
        }
    }

    struct VecVisitor<'t, T>(&'t mut Vec<T>);

    impl<'de, 't, T: DeserializeOwned> DeserializeSeed<'de> for VecVisitor<'t, T> {
        type Value = ();

        fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
            return deserializer.deserialize_seq(self);
        }
    }

    impl<'de, 't, T: DeserializeOwned> Visitor<'de> for VecVisitor<'t, T> {
        type Value = ();

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(r#"expecting [value, ...]"#)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
            while let Some(elem) = seq.next_element::<T>()? {
                self.0.push(elem);
            }
            Ok(())
        }
    }

    struct SeqItemVisitor<'t, K, V> {
        values: &'t mut Vec<V>,
        _phantom: PhantomData<K>,
    }

    impl<'de, 't, K: DeserializeOwned, V: DeserializeOwned> DeserializeSeed<'de> for SeqItemVisitor<'t, K, V> {
        type Value = K;

        fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
            return deserializer.deserialize_map(self);
        }
    }

    impl<'de, 't, K: DeserializeOwned, V: DeserializeOwned> Visitor<'de> for SeqItemVisitor<'t, K, V> {
        type Value = K;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(r#"expecting {"k":key, "v":[value, ...]}"#)
        }

        fn visit_map<A: MapAccess<'de>>(self, mut seq: A) -> Result<K, A::Error> {
            let mut key = None;
            while let Some(k) = seq.next_key()? {
                match k {
                    "k" => key = Some(seq.next_value()?),
                    "v" => seq.next_value_seed(VecVisitor(self.values))?,
                    _ => {}
                }
            }
            key.ok_or_else(|| Error::custom("missing key"))
        }
    }
};

//
// rkyv
//

const _: () = {
    use rkyv::ser::{ScratchSpace, Serializer};
    use rkyv::vec::{ArchivedVec, VecResolver};
    use rkyv::{out_field, Archive, Archived, Deserialize, Fallible, Serialize};

    pub struct ArchivedTable<K: Archive, V: Archive> {
        headers: ArchivedVec<Archived<Header<K>>>,
        values: ArchivedVec<Archived<V>>,
    }

    impl<K: Archive, V: Archive> Archive for Table<K, V> {
        type Archived = ArchivedTable<K, V>;
        type Resolver = (VecResolver, VecResolver);

        unsafe fn resolve(&self, pos: usize, (hr, vr): Self::Resolver, out: *mut Self::Archived) {
            let (fp, fo) = out_field!(out.headers);
            ArchivedVec::resolve_from_slice(self.headers_buf(), pos + fp, hr, fo);
            let (fp, fo) = out_field!(out.values);
            ArchivedVec::resolve_from_slice(self.values_buf(), pos + fp, vr, fo);
        }
    }

    impl<S, K, V> Serialize<S> for Table<K, V>
    where
        S: Serializer + ScratchSpace + ?Sized,
        K: Serialize<S>,
        V: Serialize<S>,
    {
        fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
            let hr = ArchivedVec::serialize_from_slice(self.headers_buf(), serializer);
            let vr = ArchivedVec::serialize_from_slice(self.values_buf(), serializer);
            Ok((hr?, vr?))
        }
    }

    impl<D, K, V> Deserialize<Table<K, V>, D> for ArchivedTable<K, V>
    where
        D: Fallible + ?Sized,
        K: Archive,
        Archived<K>: Deserialize<K, D>,
        V: Archive,
        Archived<V>: Deserialize<V, D>,
    {
        fn deserialize(&self, deserializer: &mut D) -> Result<Table<K, V>, D::Error> {
            let mut table = Table::alloc(self.headers.len(), self.values.len());
            for (idx, archived) in self.headers.iter().enumerate() {
                let header: Header<K> = archived.deserialize(deserializer)?;
                unsafe { table.init_header(idx, header.key, header.offset as usize) };
            }
            for (idx, archived) in self.values.iter().enumerate() {
                let value: V = archived.deserialize(deserializer)?;
                unsafe { table.init_value(idx, value) };
            }
            Ok(table)
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{s, Symbol};
    use anyhow::Result;

    #[test]
    fn test_table_basic() {
        let tb1: Table<u32, f32> = Table::alloc(0, 0);
        assert_eq!(tb1.len(), 0);
        assert_eq!(tb1.key(0), None);
        assert_eq!(tb1.values(0), None);
        assert_eq!(tb1.get(0), None);

        let mut tb2: Table<u32, f32> = Table::alloc(2, 5);
        unsafe {
            tb2.init_header(0, 123, 0);
            tb2.init_value(0, 10.0);
            tb2.init_value(1, 20.0);
            tb2.init_value(2, 30.0);
            tb2.init_header(1, 456, 3);
            tb2.init_value(3, 21.0);
            tb2.init_value(4, 22.0);
        }
        assert_eq!(tb2.len(), 2);
        assert_eq!(tb2.xkey(0), &123);
        assert_eq!(tb2.xkey(1), &456);
        assert_eq!(tb2.xvalues(0), &[10.0f32, 20.0f32, 30.0f32]);
        assert_eq!(tb2.xvalues(1), &[21.0f32, 22.0f32]);
        assert_eq!(tb2.key_iter().copied().collect::<Vec<u32>>(), vec![123, 456]);
        assert_eq!(
            tb2.values_iter().map(|x| x.to_vec()).collect::<Vec<Vec<f32>>>(),
            vec![vec![10.0, 20.0, 30.0], vec![21.0, 22.0]]
        );
        assert_eq!(
            tb2.iter()
                .map(|(k, v)| (*k, v.to_vec()))
                .collect::<Vec<(u32, Vec<f32>)>>(),
            vec![(123, vec![10.0, 20.0, 30.0]), (456, vec![21.0, 22.0])]
        );

        let mut tb2: Table<Symbol, Symbol> = Table::alloc(2, 3);
        let lab1 = s!("abc");
        let lab2 = s!("def");
        let val1 = s!("111");
        let val2 = s!("222");
        let val3 = s!("333");
        unsafe {
            tb2.init_header(0, lab1.clone(), 0);
            tb2.init_value(0, val1.clone());
            tb2.init_header(1, lab2.clone(), 1);
            tb2.init_value(1, val2.clone());
            tb2.init_value(2, val3.clone());
        }
        assert_eq!(lab1.ref_count(), 2);
        assert_eq!(lab2.ref_count(), 2);
        assert_eq!(val1.ref_count(), 2);
        assert_eq!(val2.ref_count(), 2);
        assert_eq!(val3.ref_count(), 2);
        assert_eq!(tb2.xkey(0), &lab1);
        assert_eq!(tb2.xvalues(1), &[val2.clone(), val3.clone()]);
        mem::drop(tb2);
        assert_eq!(lab1.ref_count(), 1);
        assert_eq!(lab2.ref_count(), 1);
        assert_eq!(val1.ref_count(), 1);
        assert_eq!(val2.ref_count(), 1);
        assert_eq!(val3.ref_count(), 1);
    }

    #[test]
    fn test_table_serde() {
        use serde_json;

        let json = r#"[
            {"k": 123, "v": [5.0, 6.0, 7.0]},
            {"k": 789, "v": [12.0, 13.0]}
        ]"#;
        let tb1: Table<u16, f64> = serde_json::from_str(json).unwrap();
        assert_eq!(tb1.len(), 2);
        assert_eq!(tb1.key_iter().copied().collect::<Vec<u16>>(), vec![123, 789]);
        assert_eq!(
            tb1.values_iter().map(|x| x.to_vec()).collect::<Vec<Vec<f64>>>(),
            vec![vec![5.0, 6.0, 7.0], vec![12.0, 13.0]]
        );

        let json = r#"{
            "k1": ["aaa", "bbb"],
            "k2": ["xx"],
            "k3": ["xx", "yy"]
        }"#;
        let tb2: Table<Symbol, Symbol> = serde_json::from_str(json).unwrap();
        assert_eq!(tb2.len(), 3);
        assert_eq!(
            tb2.key_iter().cloned().collect::<Vec<Symbol>>(),
            vec![s!("k1"), s!("k2"), s!("k3")]
        );
        assert_eq!(
            tb2.values_iter().map(|x| x.to_vec()).collect::<Vec<Vec<Symbol>>>(),
            vec![vec![s!("aaa"), s!("bbb")], vec![s!("xx")], vec![s!("xx"), s!("yy")]]
        );
    }

    #[test]
    fn test_table_rkyv() {
        use rkyv::ser::serializers::AllocSerializer;
        use rkyv::ser::Serializer;
        use rkyv::{Deserialize, Infallible, Serialize};

        fn test_rkyv<K, V>(table: Table<K, V>) -> Result<()>
        where
            K: fmt::Debug + PartialEq + Serialize<AllocSerializer<4096>>,
            K::Archived: Deserialize<K, Infallible>,
            V: fmt::Debug + PartialEq + Serialize<AllocSerializer<4096>>,
            V::Archived: Deserialize<V, Infallible>,
        {
            let mut serializer = rkyv::ser::serializers::AllocSerializer::<4096>::default();
            serializer.serialize_value(&table)?;
            let buffer = serializer.into_serializer().into_inner();
            let archived = unsafe { rkyv::archived_root::<Table<K, V>>(&buffer) };
            let mut deserializer = rkyv::Infallible;
            let result: Table<K, V> = archived.deserialize(&mut deserializer)?;
            if table.headers_buf() != result.headers_buf() {
                return Err(anyhow::anyhow!("rkyv test not equal"));
            }
            if table.values_buf() != result.values_buf() {
                return Err(anyhow::anyhow!("rkyv test not equal"));
            }
            Ok(())
        }

        let tb1: Table<u64, u8> = Table::alloc(0, 0);
        test_rkyv(tb1).unwrap();

        let mut tb2: Table<u64, [f32; 2]> = Table::alloc(3, 5);
        unsafe {
            tb2.init_header(0, 31, 0);
            tb2.init_header(1, 41, 0);
            tb2.init_value(0, [1.0, 2.0]);
            tb2.init_value(1, [3.0, 4.0]);
            tb2.init_value(2, [5.0, 6.0]);
            tb2.init_header(2, 51, 3);
            tb2.init_value(3, [7.0, 8.0]);
            tb2.init_value(4, [9.0, 10.0]);
        }
        test_rkyv(tb2).unwrap();
    }
}
