use rustc_hash::FxBuildHasher;
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};

use crate::template::base::TmplAny;
use crate::utils::{xerr, xerrf, xfromf, xresf, DtHashMap, TmplID, XResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub(super) struct TmplIndex {
    pub(super) ptr: u32,
    pub(super) len: u32,
}

const _: () = {
    use serde::de::{Deserialize, Deserializer};
    use serde::ser::{Serialize, Serializer};

    impl Serialize for TmplIndex {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let tuple = (self.ptr, self.len);
            tuple.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for TmplIndex {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<TmplIndex, D::Error> {
            let tuple: (u32, u32) = Deserialize::deserialize(deserializer)?;
            Ok(TmplIndex {
                ptr: tuple.0,
                len: tuple.1,
            })
        }
    }
};

pub(super) struct TmplIndexCache {
    indexes: DtHashMap<TmplID, TmplIndex>,
    path: String,
}

impl TmplIndexCache {
    pub(super) const EMPTY: TmplIndexCache = TmplIndexCache {
        indexes: DtHashMap::with_hasher(FxBuildHasher),
        path: String::new(),
    };

    pub(super) fn from_file<P: AsRef<Path>>(path: P) -> XResult<TmplIndexCache> {
        use rkyv::rancor::Failure;
        use rkyv::Archived;

        let path = PathBuf::from(path.as_ref());
        let rkyv_path = path.join("index.rkyv");
        let json_path = path.join("index.json");
        let path = path.to_string_lossy().to_string();

        if fs::exists(&rkyv_path).unwrap_or(false) {
            let buf = fs::read(&rkyv_path).map_err(xfromf!("rkyv_path={:?}", rkyv_path))?;
            let archived = unsafe { rkyv::access_unchecked::<Archived<DtHashMap<TmplID, TmplIndex>>>(&buf) };
            let indexes = rkyv::deserialize::<_, Failure>(archived).map_err(|_| xerr!(Rkyv))?;
            return Ok(TmplIndexCache { indexes, path });
        }

        if fs::exists(&json_path).unwrap_or(false) {
            let buf = fs::read(&json_path).map_err(xfromf!("json_path={:?}", json_path))?;
            let indexes: DtHashMap<TmplID, TmplIndex> =
                serde_json::from_slice(&buf).map_err(xfromf!("json_path={:?}", json_path))?;
            return Ok(TmplIndexCache { indexes, path });
        }

        xresf!(NotFound; "path={}", &path)
    }

    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        self.indexes.is_empty()
    }

    #[inline]
    pub(super) fn path(&self) -> &Path {
        Path::new(&self.path)
    }

    #[inline]
    pub(super) fn find(&self, id: TmplID) -> Option<TmplIndex> {
        self.indexes.get(&id).cloned()
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        self.indexes.len()
    }

    #[inline]
    pub(super) fn iter(&self) -> impl Iterator<Item = (&TmplID, &TmplIndex)> {
        self.indexes.iter()
    }
}

pub(super) fn load_rkyv_into(file: &mut File, id: TmplID, index: TmplIndex, buf: &mut [u8]) -> XResult<()> {
    file.seek(SeekFrom::Start(index.ptr as u64))
        .map_err(xfromf!("id={}", id))?;
    file.read_exact(buf).map_err(xfromf!("id={}", id))?;
    Ok(())
}

pub(super) fn load_json_to_rkyv(file: &mut File, id: TmplID, index: TmplIndex) -> XResult<rkyv::util::AlignedVec> {
    use rkyv::rancor::Failure;

    file.seek(SeekFrom::Start(index.ptr as u64))
        .map_err(xfromf!("id={}", id))?;
    let mut file_buf = Vec::with_capacity(index.len as usize);
    unsafe {
        file_buf.set_len(index.len as usize);
    }
    file.read_exact(&mut file_buf).map_err(xfromf!("id={}", id))?;
    // log::debug!(">>>>>>>>>> {}", str::from_utf8(&file_buf).unwrap_or(""));
    let tmpl: Box<dyn TmplAny> = serde_json::from_slice(&file_buf).map_err(xfromf!("id={}", id))?;

    let rkyv_buf = rkyv::to_bytes::<Failure>(&tmpl).map_err(|_| xerrf!(Unexpected; "id={}", id))?;
    Ok(rkyv_buf)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::utils::id;
    use crate::utils::tests::*;

    #[test]
    fn test_tmpl_index_cache_json() {
        let test_dir = prepare_tmp_dir("tmpl-index-cache-json");
        assert!(TmplIndexCache::from_file(&test_dir).is_err());

        let json_dir = test_dir.join("index.json");
        let mut map = HashMap::new();
        map.insert("Character.Aaa", TmplIndex { ptr: 0, len: 1 });
        map.insert("Character.Aaa^1", TmplIndex { ptr: 2, len: 3 });
        map.insert("Accessory.Bbb.Ccc", TmplIndex { ptr: 4, len: 5 });
        map.insert("Accessory.Bbb.Ccc^0", TmplIndex { ptr: 6, len: 7 });
        write_json(&json_dir, &map);

        let cache = TmplIndexCache::from_file(&test_dir).unwrap();
        assert_eq!(cache.indexes.len(), 4);
        assert_eq!(cache.find(id!("Character.Aaa")).unwrap(), TmplIndex { ptr: 0, len: 1 });
        assert_eq!(cache.find(id!("Character.Aaa^1")).unwrap(), TmplIndex {
            ptr: 2,
            len: 3
        });
        assert_eq!(cache.find(id!("Accessory.Bbb.Ccc")).unwrap(), TmplIndex {
            ptr: 4,
            len: 5
        });
        assert_eq!(cache.find(id!("Accessory.Bbb.Ccc^0")).unwrap(), TmplIndex {
            ptr: 6,
            len: 7
        });
    }

    #[test]
    fn test_tmpl_index_cache_rkyv() {
        let test_dir = prepare_tmp_dir("tmpl-index-cache-rkyv");
        assert!(TmplIndexCache::from_file(&test_dir).is_err());

        let rkyv_dir = test_dir.join("index.rkyv");
        let mut map = DtHashMap::with_hasher(FxBuildHasher);
        map.insert(id!("Character.Aaa"), TmplIndex { ptr: 0, len: 1 });
        map.insert(id!("Character.Aaa^1"), TmplIndex { ptr: 2, len: 3 });
        map.insert(id!("Accessory.Bbb.Ccc"), TmplIndex { ptr: 4, len: 5 });
        map.insert(id!("Accessory.Bbb.Ccc^0"), TmplIndex { ptr: 6, len: 7 });
        write_rkyv(&rkyv_dir, &map);

        let cache = TmplIndexCache::from_file(&test_dir).unwrap();
        assert_eq!(cache.indexes.len(), 4);
        assert_eq!(cache.find(id!("Character.Aaa")).unwrap(), TmplIndex { ptr: 0, len: 1 });
        assert_eq!(cache.find(id!("Character.Aaa^1")).unwrap(), TmplIndex {
            ptr: 2,
            len: 3
        });
        assert_eq!(cache.find(id!("Accessory.Bbb.Ccc")).unwrap(), TmplIndex {
            ptr: 4,
            len: 5
        });
        assert_eq!(cache.find(id!("Accessory.Bbb.Ccc^0")).unwrap(), TmplIndex {
            ptr: 6,
            len: 7
        });
    }
}
