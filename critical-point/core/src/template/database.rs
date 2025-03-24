use std::collections::hash_map::Entry;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};

use crate::template::base::TmplAny;
use crate::utils::{xfrom, xres, IdentityState, StrID, SymbolMap, XResult, Xrc};

#[cfg(not(feature = "server-side"))]
mod client {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::utils::CastPtr;

    #[derive(Debug, Clone)]
    pub struct TmplDatabase(Rc<RefCell<TmplDatabaseInner>>);

    impl TmplDatabase {
        pub fn new<P: AsRef<Path>>(path: P) -> XResult<TmplDatabase> {
            let inner = TmplDatabaseInner::new(path)?;
            Ok(TmplDatabase(Rc::new(RefCell::new(inner))))
        }

        pub fn find(&self, id: &StrID) -> XResult<Xrc<dyn TmplAny>> {
            Ok(self.0.borrow_mut().load(id)?.clone())
        }

        pub fn find_as<T: TmplAny + 'static>(&self, id: &StrID) -> XResult<Xrc<T>> {
            self.0.borrow_mut().load(id)?.cast_to()
        }

        pub fn clean_up(&mut self) -> XResult<()> {
            self.0.borrow_mut().clean_up();
            Ok(())
        }
    }
}

#[cfg(not(feature = "server-side"))]
pub use client::*;

#[cfg(feature = "server-side")]
mod server {
    use std::sync::Arc;

    use super::*;
    use crate::utils::CastPtr;

    #[derive(Debug, Clone)]
    pub struct TmplDatabase(Arc<TmplDatabaseInner>);

    impl TmplDatabase {
        pub fn new<P: AsRef<Path>>(path: P) -> XResult<TmplDatabase> {
            let mut inner = TmplDatabaseInner::new(path)?;
            inner.load_all()?;
            return Ok(TmplDatabase(Arc::new(inner)));
        }

        pub fn find(&self, id: &StrID) -> XResult<Xrc<dyn TmplAny>> {
            return Ok(self.0.find(id)?.clone());
        }

        pub fn find_as<T: TmplAny + 'static>(&self, id: &StrID) -> XResult<Xrc<T>> {
            return Ok(self.0.find(id)?.cast_to()?);
        }

        pub fn clean_up(&mut self) -> XResult<()> {
            return Ok(()); // Not support clean up for server side
        }
    }
}

#[cfg(feature = "server-side")]
pub use server::*;

#[derive(Debug)]
struct TmplDatabaseInner {
    indexes: SymbolMap<TmplDbIndex>,
    data_path: PathBuf,
    data_file: File,
    data_buf: Vec<u8>,
    cache_map: SymbolMap<Xrc<dyn TmplAny>>,
}

#[derive(Debug, Clone)]
struct TmplDbIndex {
    ptr: u32,
    len: u32,
    cache: bool,
}

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
struct RawIndexes(SymbolMap<(u32, u32, u8)>);

impl TmplDatabaseInner {
    #![allow(dead_code)]

    pub fn new<P: AsRef<Path>>(path: P) -> XResult<TmplDatabaseInner> {
        let mut path = PathBuf::from(path.as_ref());
        path.push("index.json");
        let json = fs::read_to_string(&path).map_err(xfrom!())?;

        let raw: RawIndexes = serde_json::from_str(&json).map_err(xfrom!())?;
        let mut indexes: SymbolMap<TmplDbIndex> = SymbolMap::with_capacity_and_hasher(raw.0.len(), IdentityState);
        for (id, (ptr, len, cache)) in raw.0 {
            indexes.insert(
                id,
                TmplDbIndex {
                    ptr,
                    len,
                    cache: cache != 0,
                },
            );
        }

        path.pop();
        path.push("data.json");
        let data_file = OpenOptions::new()
            .read(true)
            .write(false)
            .create_new(false)
            .open(&path)
            .map_err(xfrom!())?;

        Ok(TmplDatabaseInner {
            indexes,
            data_path: path,
            data_file,
            data_buf: Vec::with_capacity(1024 * 10),
            cache_map: SymbolMap::with_capacity_and_hasher(256, IdentityState),
        })
    }

    pub fn load_all(&mut self) -> XResult<()> {
        unimplemented!()
    }

    pub fn find(&self, id: &StrID) -> XResult<&Xrc<dyn TmplAny>> {
        return match self.cache_map.get(id) {
            Some(tmpl) => Ok(tmpl),
            None => xres!(NotFound),
        };
    }

    pub fn load(&mut self, id: &StrID) -> XResult<&Xrc<dyn TmplAny>> {
        match self.cache_map.entry(id.clone()) {
            Entry::Occupied(entry) => return Ok(entry.into_mut()),
            Entry::Vacant(entry) => {
                let index = match self.indexes.get(id) {
                    Some(index) => index,
                    None => return xres!(NotFound),
                };

                self.data_file
                    .seek(SeekFrom::Start(index.ptr as u64))
                    .map_err(xfrom!())?;
                if self.data_buf.capacity() < index.len as usize {
                    self.data_buf.resize(index.len as usize, 0);
                } else {
                    unsafe { self.data_buf.set_len(index.len as usize) };
                }
                self.data_file.read_exact(&mut self.data_buf).map_err(xfrom!())?;

                let tmpl: Xrc<dyn TmplAny> = serde_json::from_slice(&self.data_buf).map_err(xfrom!())?;
                return Ok(entry.insert(tmpl));
            }
        };
    }

    pub fn clean_up(&mut self) {
        self.cache_map.retain(|_, v| {
            if Xrc::strong_count(v) > 1 {
                return true;
            }
            return match self.indexes.get(&v.id()) {
                Some(index) => index.cache,
                None => false,
            };
        });
    }
}
